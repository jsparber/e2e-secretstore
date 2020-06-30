pub use crate::api::contract::AclContract;
pub use crate::api::secretstore::SecretStore;
pub use crate::api::shadow::Shadow;
use crate::Data;
use crate::Password;
use ethcontract::transaction::Account;
use ethcontract::transaction::TransactionResult;
use failure::{Error, SyncFailure};
use primitive_types::{H160, H256, H512};
use serde_derive::{Deserialize, Serialize};
use std::str::FromStr;
use web3::transports::EventLoopHandle;

#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct EncryptedDocumentKey {
    pub common_point: H512,
    pub encrypted_point: H512,
    pub encrypted_key: Data,
}

#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct DecryptionKeys {
    pub common_point: H512,
    pub decrypted_secret: H512,
    pub decrypt_shadows: Vec<Data>,
}

pub struct Blockchain {
    ss_client: SecretStore<jsonrpc_client_http::HttpHandle>,
    shadow_client: Shadow,
    contract: AclContract,
    /* Dropping the event loop will break the web3 connection */
    #[allow(dead_code)]
    eloop: EventLoopHandle,
}

impl Blockchain {
    /* Address and password of the user who creates the initial contract
     * TODO: shouldn't be required since we create the contract on the setup of the network */
    pub fn new(
        address: H160,
        password: &Password,
        shadow: &str,
        jsonrpc: &str,
    ) -> Result<Blockchain, Error> {
        let shadow_client = Shadow::new(shadow);
        let ss_client = SecretStore::create(jsonrpc)?;

        let (eloop, transport) = web3::transports::Http::new(jsonrpc)?;
        let web3 = web3::Web3::new(transport);
        let account = Account::Locked(address, password.into(), None);

        let contract = AclContract::new(account, &web3)?;

        Ok(Blockchain {
            ss_client,
            shadow_client,
            contract,
            eloop,
        })
    }

    /* Todo: make self not mut, but SecretStore requires it */
    pub async fn encrypt(
        &mut self,
        address: H160,
        password: &Password,
        document_id: &str,
        document: &str,
        threshold: u32,
    ) -> Result<Data, Error> {
        // Sign the document key id
        let signed_document_key_id = self
            .ss_client
            .secretstore_signRawHash(address, password, H256::from_str(document_id)?)
            .call()
            .map_err(SyncFailure::new)?;

        let public_server_key = self
            .shadow_client
            .generate_server_key(document_id, &signed_document_key_id, threshold)
            .await?;

        let encrypted_key = self
            .ss_client
            .secretstore_generateDocumentKey(
                address,
                password,
                H512::from_str(&public_server_key.trim_start_matches("0x"))?,
            )
            .call()
            .map_err(SyncFailure::new)?;

        let encrypted_document = self
            .ss_client
            .secretstore_encrypt(
                address,
                password,
                encrypted_key.encrypted_key,
                format!("0x{}", hex::encode(document)),
            )
            .call()
            .map_err(SyncFailure::new)?;

        self.shadow_client
            .store_document_key(
                document_id,
                &signed_document_key_id,
                encrypted_key.common_point,
                encrypted_key.encrypted_point,
            )
            .await?;
        Ok(encrypted_document.into())
    }

    /* Todo: make self not mut, but SecretStore requires it */
    pub async fn decrypt(
        &mut self,
        address: H160,
        password: &Password,
        document_id: &str,
        encrypted_document: &Data,
    ) -> Result<Data, Error> {
        let signed_document_key_id = self
            .ss_client
            .secretstore_signRawHash(address, password, H256::from_str(document_id)?)
            .call()
            .map_err(SyncFailure::new)?;

        let key = self
            .shadow_client
            .get_document_key(document_id, &signed_document_key_id)
            .await?;

        let hashed_document = self
            .ss_client
            .secretstore_shadowDecrypt(
                address,
                password,
                key.decrypted_secret,
                key.common_point,
                key.decrypt_shadows,
                &encrypted_document,
            )
            .call()
            .map_err(SyncFailure::new)?;

        Ok(Data::from_utf8(hex::decode(
            hashed_document.trim_start_matches("0x"),
        )?)?)
    }

    pub async fn allow_access(
        &mut self,
        address: H160,
        password: &Password,
        document_id: &str,
        addresses: &[H160],
    ) -> Result<TransactionResult, Error> {
        let document_id = H256::from_str(document_id).unwrap();
        let account = Account::Locked(address, password.into(), None);
        let result = self
            .contract
            .allow_access(document_id.into(), addresses.into())
            .from(account.clone())
            .into_inner()
            .estimate_gas()
            .await
            .map_err(SyncFailure::new)?;
        println!("Gas price {:?}", result);

        let result = self
            .contract
            .allow_access(document_id.into(), addresses.into())
            .from(account)
            .gas(8000000.into())
            .send()
            .await
            .map_err(SyncFailure::new)?;
        Ok(result)
    }

    pub async fn check_permissions(
        &mut self,
        address: H160,
        document_id: &str,
    ) -> Result<bool, Error> {
        let document_id = H256::from_str(document_id).unwrap();
        let result = self
            .contract
            .check_permissions(address, document_id.into())
            .from(address)
            .call()
            .await
            .map_err(SyncFailure::new)?;
        Ok(result)
    }
}
