#![allow(non_snake_case)]
use crate::blockchain::EncryptedDocumentKey;
use crate::Data;
use crate::Password;
use failure::{Error, SyncFailure};
use jsonrpc_client_http::HttpTransport;
use primitive_types::{H160, H256, H512};

jsonrpc_client!(pub struct SecretStore {
    pub fn secretstore_generateDocumentKey(&mut self, address: H160, password: &Password, server_key_public: H512) -> RpcRequest<EncryptedDocumentKey>;
    pub fn secretstore_encrypt(&mut self, address: H160, password: &Password, key: Data, data: Data) -> RpcRequest<Data>;
    pub fn secretstore_shadowDecrypt(&mut self, address: H160, password: &Password, decrypted_secret: H512, common_point: H512, decrypt_shadows: Vec<Data>, data: &Data) -> RpcRequest<Data>;
    pub fn secretstore_signRawHash(&mut self, address: H160, password: &Password, raw_hash: H256) -> RpcRequest<Data>;
});

impl SecretStore<jsonrpc_client_http::HttpHandle> {
    pub fn create(url: &str) -> Result<SecretStore<jsonrpc_client_http::HttpHandle>, Error> {
        /* TODO: Use shared tokio runtime:
         * possible soluiton https://github.com/mullvad/jsonrpc-client-rs/compare/add-newhttp-transport */
        let transport = HttpTransport::new()
            .standalone()
            .map_err(SyncFailure::new)?;
        let transport_handle = transport.handle(url).map_err(SyncFailure::new)?;
        Ok(SecretStore::new(transport_handle))
    }
}
