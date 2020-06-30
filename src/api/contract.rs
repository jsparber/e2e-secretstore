use ethcontract::transaction::Account;
use ethcontract::web3::api::Web3;
use ethcontract::web3::types::*;
use std::fs::File;
use std::io::prelude::*;
use std::str::FromStr;
use web3::transports::Http;

ethcontract::contract!("./SSPermissions.json", contract = AclContract);

impl AclContract {
    pub fn new(account: Account, web3: &Web3<Http>) -> Result<Self, failure::Error> {
        let file = File::open("./contract-address.txt");
        let contract_address = file.and_then(|mut file| {
            let mut contents = String::new();
            let _ = file.read_to_string(&mut contents);
            Ok(contents)
        });

        let contract = if let Ok(address) = contract_address {
            println!("Use contract at address loaded from file 'contract-address.txt'");
            let contract_address =
                H160::from_str(&address).expect("Error in contract-address.txt file");
            /* Load contract from stored address. This doesn't perform any checks */
            AclContract::at(web3, contract_address)
        } else {
            /* TODO: this should be done when creating the network not here
            Deploy new contract if we don't have an address already */
            println!("'contract-address.txt' doens't exsist, deploy new contract");
            futures::executor::block_on(deploy_new_contract(account, web3))?
        };
        println!("Use contract at address {:?}", contract.address());
        Ok(contract)
    }
}

async fn deploy_new_contract(
    account: Account,
    web3: &Web3<Http>,
) -> Result<AclContract, failure::Error> {
    let contract = AclContract::builder(web3).from(account).deploy().await?;
    let contract_address = format!("{:x}", contract.address());
    let file = File::create("./contract-address.txt");
    let _ = file.and_then(|mut file| file.write_all(contract_address.as_bytes()));

    Ok(contract)
}
