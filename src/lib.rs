#![feature(test)]
#![allow(non_snake_case)]
#[macro_use]
extern crate jsonrpc_client_core;
extern crate jsonrpc_client_http;
extern crate rand;
extern crate test;

use primitive_types::H160;
use sha2::{Digest, Sha256};
use std::str::FromStr;
mod api;
mod blockchain;

type Password = str;
type Data = String;

use crate::blockchain::Blockchain;
use ethcontract::transaction::TransactionResult;
use failure::Error;
use tokio::runtime::Runtime;

pub struct CryptoSecretStore {
    blockchain: Blockchain,
    address: H160,
    password: String,
    rt: Runtime,
}

impl CryptoSecretStore {
    pub fn new(addr: &str, password: &str) -> Self {
        let address = H160::from_str(addr).unwrap();
        let password = password.to_string();
        let blockchain = Blockchain::new(
            address,
            &password,
            "http://127.0.0.1:8010",
            "http://127.0.0.1:8545",
        )
        .unwrap();

        let rt = Runtime::new().unwrap();

        CryptoSecretStore {
            blockchain,
            address,
            password,
            rt,
        }
    }

    pub fn generate_id(&mut self, document: &str) -> String {
        format!("{:x}", Sha256::digest(&document.as_bytes()))
    }

    pub fn encrypt(&mut self, id: &str, document: &str, threshold: u32) -> Result<String, Error> {
        let document_id = &format!("{:x}", Sha256::digest(&id.as_bytes()));
        self.rt.block_on(self.blockchain.encrypt(
            self.address,
            &self.password,
            document_id,
            document,
            threshold,
        ))
    }

    pub fn decrypt(&mut self, id: &str, encrypted_document: &str) -> Result<String, Error> {
        let document_id = &format!("{:x}", Sha256::digest(&id.as_bytes()));
        self.rt.block_on(self.blockchain.decrypt(
            self.address,
            &self.password,
            document_id,
            &encrypted_document.to_string(),
        ))
    }

    pub fn allow_access(
        &mut self,
        document_id: &str,
        addresses: &[H160],
    ) -> Result<TransactionResult, Error> {
        let document_id = &format!("{:x}", Sha256::digest(&document_id.as_bytes()));
        self.rt.block_on(self.blockchain.allow_access(
            self.address,
            &self.password,
            document_id,
            addresses,
        ))
    }

    pub fn check_permissions(&mut self, address: H160, document_id: &str) -> Result<bool, Error> {
        let document_id = &format!("{:x}", Sha256::digest(&document_id.as_bytes()));
        self.rt
            .block_on(self.blockchain.check_permissions(address, document_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use ethcontract::U256;
    use primitive_types::H160;
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};
    use std::cell::RefCell;
    use std::convert::TryInto;
    use std::fs::File;
    use std::fs::OpenOptions;
    use std::io::prelude::*;
    use std::io::Write;
    use std::rc::Rc;
    use std::time;
    use std::time::Duration;
    use test::Bencher;

    struct Message {
        id: String,
        cleartext: String,
        ciphertext: String,
    }

    #[test]
    fn setup1() {
        let address = "27d39a0fe767025e7ea0f78dccd4665929e3a8f2";
        let password = "alicepwd";
        let store = Rc::new(RefCell::new(CryptoSecretStore::new(address, password)));
    }

    #[test]
    fn setup_encrypt() {
        let address = "27d39a0fe767025e7ea0f78dccd4665929e3a8f2";
        let password = "alicepwd";
        let store = Rc::new(RefCell::new(CryptoSecretStore::new(address, password)));
        encrypt(store, 30, 1);
    }

    #[test]
    fn time_encrypt_decrypt() {
        let address = "27d39a0fe767025e7ea0f78dccd4665929e3a8f2";
        let password = "alicepwd";
        let store = Rc::new(RefCell::new(CryptoSecretStore::new(address, password)));
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open("./stats.txt")
            .unwrap();

        let _ = writeln!(file, "[");

        let limit = 25;
        for threshold in 0..limit {
            let now = time::Instant::now();
            for _i in 0..5 {
                let message = encrypt(store.clone(), 30, threshold);
                {
                    let address = H160::from_str(address).unwrap();
                    let addresses = vec![address];
                    let mut store = store.borrow_mut();
                    let result = store.allow_access(&message.id, &addresses).unwrap();
                    assert_eq!(result.is_receipt(), true);
                }
                decrypt(store.clone(), message);
            }
            let durtation = now.elapsed();
            println!("Threshold: {}, Elapsed time {:?}", threshold, durtation / 5);
            if threshold == limit - 1 {
                let _ = writeln!(file, "{}", (durtation / 5).as_millis());
            } else {
                let _ = writeln!(file, "{},", (durtation / 5).as_millis());
            }
        }
        let _ = writeln!(file, "]");
    }

    /*
     * Not sure if this makes sense
    #[test]
    fn time_encrypt_decrypt_var_room_members() {
        let address = "27d39a0fe767025e7ea0f78dccd4665929e3a8f2";
        let password = "alicepwd";
        let store = Rc::new(RefCell::new(CryptoSecretStore::new(address, password)));
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open("./stats.txt")
            .unwrap();

        let _ = writeln!(file, "[");
        let limit = 1000;
        let step: u32 = 10;
        for threshold in (step..limit).step_by(step.try_into().unwrap()) {
            let mut messages: Vec<Message> = vec![];
            let test_addresses = (1..11).map(|x| x*x).collect();
            let test_addr = H160::random();
            let now = time::Instant::now();
            for _i in 0..5 {
                let msg = encrypt(store.clone(), 30, 1);
                {
                    let address = H160::from_str(address).unwrap();
                    let addresses = vec![address];
                    let mut store = store.borrow_mut();
                    let result = store.allow_access(&msg.id, &addresses).unwrap();
                    assert_eq!(result, true);
                }
                messages.push(msg);
            }
            let durtation_encrypt = now.elapsed()/5;
            let now = time::Instant::now();

            for _i in 0..5 {
                let msg = messages.pop().unwrap();
                decrypt(store.clone(), msg);
            }
            let durtation_decrypt = now.elapsed()/5;
            println!("Threshold: {}, Elapsed time encryption {:?}, decryption {:?}",
                     threshold,
                     durtation_encrypt,
                     durtation_decrypt);
            let _ = writeln!(file, "{{\"encryption\" : {}, \"decryption\": {}, \"threshold\" : {}}},",
                             (durtation_encrypt).as_millis(), durtation_decrypt.as_millis(), threshold);
        }
        let _ = writeln!(file, "]");
        let _ = writeln!(file, "#Range {}..{} with step {}", step, limit, step);
        }
        */
    #[test]
    fn time_encrypt_decrypt_var_threshold() {
        let address = "27d39a0fe767025e7ea0f78dccd4665929e3a8f2";
        let password = "alicepwd";
        let store = Rc::new(RefCell::new(CryptoSecretStore::new(address, password)));
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open("./stats.txt")
            .unwrap();

        let _ = writeln!(file, "[");
        let limit = 25;
        let step: u32 = 1;
        for threshold in (step..limit) {
            let mut messages: Vec<Message> = vec![];
            let now = time::Instant::now();
            for _i in 0..5 {
                let msg = encrypt(store.clone(), 30, threshold);
                messages.push(msg);
            }
            let durtation_encrypt = now.elapsed() / 5;

            let now = time::Instant::now();
            for i in 0..5 {
                let address = H160::from_str(address).unwrap();
                let addresses = vec![address];
                let mut store = store.borrow_mut();
                let result = store.allow_access(&messages[i].id, &addresses).unwrap();
                assert_eq!(result.is_receipt(), true);
            }

            let durtation_set_access = now.elapsed() / 5;

            let now = time::Instant::now();

            for _i in 0..5 {
                let msg = messages.pop().unwrap();
                decrypt(store.clone(), msg);
            }
            let durtation_decrypt = now.elapsed() / 5;
            println!(
                "Threshold: {}, Elapsed time encryption {:?}, decryption {:?}",
                threshold, durtation_encrypt, durtation_decrypt
            );
            let _ = writeln!(
                file,
                "{{\"encryption\" : {}, \"decryption\": {}, \"threshold\" : {}, \"set_access\" : {}}},",
                (durtation_encrypt).as_millis(),
                durtation_decrypt.as_millis(),
                threshold,
                durtation_set_access.as_millis()
            );
        }
        let _ = writeln!(file, "]");
        let _ = writeln!(file, "#Range {}..{} with step {}", step, limit, step);
    }

    #[test]
    fn time_encrypt_decrypt_var_message_size() {
        let address = "27d39a0fe767025e7ea0f78dccd4665929e3a8f2";
        let password = "alicepwd";
        let store = Rc::new(RefCell::new(CryptoSecretStore::new(address, password)));
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open("./stats.txt")
            .unwrap();

        let _ = writeln!(file, "[");
        let limit = 10000;
        let step: u32 = 100;
        for size in (step..limit).step_by(step.try_into().unwrap()) {
            let mut messages: Vec<Message> = vec![];
            let now = time::Instant::now();
            for _i in 0..5 {
                let msg = encrypt(store.clone(), size, 1);
                messages.push(msg);
            }
            let durtation_encrypt = now.elapsed() / 5;

            let now = time::Instant::now();
            for i in 0..5 {
                let address = H160::from_str(address).unwrap();
                let addresses = vec![address];
                let mut store = store.borrow_mut();
                let result = store.allow_access(&messages[i].id, &addresses).unwrap();
                assert_eq!(result.is_receipt(), true);
            }
            let durtation_set_access = now.elapsed() / 5;

            let now = time::Instant::now();

            for _i in 0..5 {
                let msg = messages.pop().unwrap();
                decrypt(store.clone(), msg);
            }
            let durtation_decrypt = now.elapsed() / 5;
            println!(
                "Message Size: {}, Elapsed time encryption {:?}, decryption {:?}",
                size, durtation_encrypt, durtation_decrypt
            );
            let _ = writeln!(
                file,
                "{{\"encryption\" : {}, \"decryption\": {}, \"set_access\" : {}, \"length\": {}}},",
                (durtation_encrypt).as_millis(),
                durtation_decrypt.as_millis(),
                durtation_set_access.as_millis(),
                size
            );
        }
        let _ = writeln!(file, "]");
        let _ = writeln!(file, "#Range {}..{} with step {}", step, limit, step);
    }

    #[test]
    fn setup_encrypt_decrypt() {
        let address = "27d39a0fe767025e7ea0f78dccd4665929e3a8f2";
        let password = "alicepwd";
        let store = Rc::new(RefCell::new(CryptoSecretStore::new(address, password)));
        let message = encrypt(store.clone(), 30, 1);
        {
            let address = H160::from_str(address).unwrap();
            let addresses = vec![address];
            let mut store = store.borrow_mut();
            let result = store.check_permissions(address, &message.id).unwrap();
            assert_eq!(result, false);
            let result = store.allow_access(&message.id, &addresses).unwrap();
            assert_eq!(result.is_receipt(), true);
            let result = store.check_permissions(address, &message.id).unwrap();
            assert_eq!(result, true);
        }
        decrypt(store, message);
    }

    #[test]
    fn access_controll() {
        let address = "27d39a0fe767025e7ea0f78dccd4665929e3a8f2";
        let password = "alicepwd";
        let store = Rc::new(RefCell::new(CryptoSecretStore::new(address, password)));
        let mut store = store.borrow_mut();

        let document: String = thread_rng().sample_iter(&Alphanumeric).take(30).collect();
        let id = store.generate_id(&document);
        let test_addr = H160::random();
        let test_addr1 = H160::random();
        let addresses = vec![test_addr, test_addr1];
        let result = store.check_permissions(test_addr, &id).unwrap();
        assert_eq!(result, false);
        let result = store.check_permissions(test_addr1, &id).unwrap();
        assert_eq!(result, false);
        let result = store.allow_access(&id, &addresses).unwrap();
        assert_eq!(result.is_receipt(), true);
        let result = store.check_permissions(test_addr, &id).unwrap();
        assert_eq!(result, true);
        let result = store.check_permissions(test_addr1, &id).unwrap();
        assert_eq!(result, true);
    }

    #[test]
    fn time_access_controll() {
        let address = "27d39a0fe767025e7ea0f78dccd4665929e3a8f2";
        let password = "alicepwd";
        let mut store = CryptoSecretStore::new(address, password);
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open("./stats.txt")
            .unwrap();

        let _ = writeln!(file, "[");
        let limit = 1000;
        let step: u32 = 10;
        for size in (step..limit).step_by(step.try_into().unwrap()) {
            let now = time::Instant::now();
            let test_addresses: Vec<H160> = (0..size).map(|_| H160::random()).collect();
            let test_ids: Vec<String> = (0..5)
                .map(|_| {
                    let document: String =
                        thread_rng().sample_iter(&Alphanumeric).take(30).collect();
                    let id = store.generate_id(&document);
                    id
                })
                .collect();

            let mut used_gas: U256 = 0.into();
            for i in 0..5 {
                let result = store.allow_access(&test_ids[i], &test_addresses).unwrap();
                assert_eq!(result.is_receipt(), true);
                used_gas += result.as_receipt().unwrap().gas_used.unwrap();
            }
            let durtation_set_access = now.elapsed() / 5;
            let used_gas = used_gas / 5;

            let now = time::Instant::now();
            for i in 0..5 {
                for addr in &test_addresses {
                    let result = store.check_permissions(addr.clone(), &test_ids[i]).unwrap();
                    assert_eq!(result, true);
                }
            }
            let durtation_get_access = now.elapsed() / 5;
            println!("Number of addresses: {}, Elapsed time to set permission {:?}, to check permissions{:?}",
                     size,
                     durtation_set_access,
                     durtation_get_access);
            let _ = writeln!(
                file,
                "{{\"set\" : {}, \"get\": {}, \"no_members\": {}, \"used_gas\": {}}},",
                (durtation_get_access).as_millis(),
                durtation_set_access.as_millis(),
                size,
                used_gas
            );
        }
        let _ = writeln!(file, "]");
        let _ = writeln!(
            file,
            "#Permission time Range {}..{} with step {}",
            step, limit, step
        );
    }

    fn encrypt(store: Rc<RefCell<CryptoSecretStore>>, size: u32, threshold: u32) -> Message {
        let mut store = store.borrow_mut();
        let document: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(size.try_into().unwrap())
            .collect();
        let id = store.generate_id(&document);
        let ciphertext = store.encrypt(&id, &document, threshold);
        assert_eq!(ciphertext.is_ok(), true);
        Message {
            id: id,
            ciphertext: ciphertext.unwrap(),
            cleartext: document,
        }
    }

    fn decrypt(store: Rc<RefCell<CryptoSecretStore>>, message: Message) {
        let mut store = store.borrow_mut();
        let cleartext = store.decrypt(&message.id, &message.ciphertext);
        println!("Whats now the error {:?}", cleartext);
        let cleartext = if !cleartext.is_ok() {
            store.decrypt(&message.id, &message.ciphertext)
        } else {
            cleartext
        };
        assert_eq!(cleartext.is_ok(), true);
        assert_eq!(cleartext.unwrap(), message.cleartext);
    }

    /*
    #[bench]
    fn bench_encrypt(b: &mut Bencher) {
        let address = "27d39a0fe767025e7ea0f78dccd4665929e3a8f2";
        let password = "alicepwd";
        let store = Rc::new(RefCell::new(CryptoSecretStore::new(address, password)));
        //let messages: Rc<RefCell<Vec<Message>>> = Rc::new(RefCell::new(vec![]));

        for i in 0..25 {
            let threshold = i;
            let result = b.bench(|b| b.iter(|| {
                 encrypt(store.clone(), threshold)
            }));
            if let Some(result) = result {
                let stats = format!("{:?}", result);
                let mut f = File::create(format!("./stats{}.txt", threshold)).unwrap();
                let _ = f.write_all(stats.as_bytes());
            }
        }
    }
    */

    /*
    #[bench]
    fn bench_encrypt_decrypt(b: &mut Bencher) {
        let address = "27d39a0fe767025e7ea0f78dccd4665929e3a8f2";
        let password = "alicepwd";
        let store = Rc::new(RefCell::new(CryptoSecretStore::new(address, password)));
        //let messages: Rc<RefCell<Vec<Message>>> = Rc::new(RefCell::new(vec![]));

        for i in 0..25 {
            let threshold = i;
            let result = b.bench(|b| b.iter(|| {
                let message = encrypt(store.clone(), threshold);
                {
                    let address = H160::from_str(address).unwrap();
                let addresses = vec![address];
                    let mut store = store.borrow_mut();
                    let result = store.allow_access(&message.id, &addresses).unwrap();
                    assert_eq!(result, true);
                }
                decrypt(store.clone(), message);
            }));
            if let Some(result) = result {
                let stats = format!("{:?}", result);
                let mut f = File::create(format!("./stats{}.txt", threshold)).unwrap();
                let _ = f.write_all(stats.as_bytes());
            }
        }
    }
    */

    /*
    #[bench]
    fn bench_decrypt(b: &mut Bencher) {
        let address = "27d39a0fe767025e7ea0f78dccd4665929e3a8f2";
        let password = "alicepwd";
        for threshold in 0..5 {
            let result = b
                .bench(|b| {
                    let store = Rc::new(RefCell::new(CryptoSecretStore::new(
                        address, password, threshold,
                    )));
                    b.iter(|| encrypt(store.clone()))
                })
                .unwrap();
            let stats = format!("{:?}", result);
            let mut f = File::create(format!("./stats{}.txt", threshold)).unwrap();
            f.write_all(stats.as_bytes());
        }
    }
    */

    /*
    #[bench]
    fn bench_encrypt_2(b: &mut Bencher) {
        let address = "27d39a0fe767025e7ea0f78dccd4665929e3a8f2";
        let password = "alicepwd";
        let threshold = 8;
        //for (threshold = 0; threshold < 9; threshold++) {
            let store = Rc::new(RefCell::new(CryptoSecretStore::new(address, password, threshold)));
            b.iter(|| encrypt(store.clone()));
        //}
    }
    */

    //futures::executor::block_on(store.blockchain.access_controll(address));
    /*
       #[tokio::test]
       async fn test() {
       let address = H160::from_str("1879f9419e6ffb56bff78c76a2576d15a3af0641").unwrap();
    let password = "alicepwd";
    let document = "Hello World";
    let document_id = &format!("{:x}", Sha256::digest(&document.as_bytes()));

    let mut blockchain =
    Blockchain::new("http://127.0.0.1:8010", "http://127.0.0.1:8545", 1).unwrap();

    println!("Input document: {}", document);
    println!("With ID: {}", document_id);

    let encrypted_document = blockchain
    .encrypt(address, password, document_id, document)
    .await
    .unwrap();

    println!("Encrypted document: {}", encrypted_document);

    let address = H160::from_str("32a93089dc00e6b8379c3f3c28ac1df19a575e5f").unwrap();
    let password = "bobpwd";
    let decrypted_document = blockchain
    .decrypt(address, password, document_id, &encrypted_document)
    .await
    .unwrap();

    assert_eq!(document, decrypted_document);
    println!("Successfully decrypted the document");
    }
    */
}
