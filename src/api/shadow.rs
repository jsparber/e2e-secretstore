use crate::blockchain::DecryptionKeys;
use crate::Data;
use primitive_types::H512;
use reqwest::{Client, IntoUrl, Url};
use serde_json;

#[derive(Debug)]
pub enum Error {
    KeyAlreadyGenerated,
    KeyAlreadyStored,
    KeyNotFound,
    Unknown(String),
    Reqwest(reqwest::Error),
    UrlParse(url::ParseError),
    HashParse(serde_json::Error),
}

impl std::error::Error for Error {}

impl From<url::ParseError> for Error {
    fn from(error: url::ParseError) -> Self {
        Error::UrlParse(error)
    }
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Error::Reqwest(error)
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Error::HashParse(error)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::KeyAlreadyGenerated => {
                write!(f, "Server key with this ID is already generated.")
            }
            Error::KeyNotFound => write!(f, "Server key with this ID is not found."),
            Error::KeyAlreadyStored => write!(f, "Document key with this ID is already stored."),
            Error::Unknown(text) => write!(f, "UnknownError: {}", text),
            Error::Reqwest(error) => write!(f, "Reqwest: {}", error),
            Error::UrlParse(error) => write!(f, "UrlParse: {}", error),
            Error::HashParse(error) => write!(f, "HashParse: {}", error),
        }
    }
}

pub struct Shadow {
    url: Url,
    client: Client,
}

const CHARS_TO_TRIM: &[char] = &['"', '\\'];

impl Shadow {
    pub fn new<T: IntoUrl>(url: T) -> Self {
        let url: Url = url.into_url().unwrap().join("shadow/").unwrap();
        let client = Client::new();
        Shadow { url, client }
    }

    pub async fn generate_server_key(
        &self,
        document_key_id: &str,
        signed_document_key_id: &Data,
        threshold: u32,
    ) -> Result<Data, Error> {
        let signed_document_key_id = signed_document_key_id
            .trim_matches(CHARS_TO_TRIM)
            .trim_start_matches("0x");
        let url = self.url.join(&format!(
            "{}/{}/{}",
            document_key_id, signed_document_key_id, threshold
        ))?;
        parse_response(
            self.client
                .post(url)
                .send()
                .await
                .map_err(|err| Error::Reqwest(err))?,
        )
        .await
    }

    pub async fn store_document_key(
        &self,
        document_key_id: &str,
        signed_document_key_id: &Data,
        common_point: H512,
        encryption_point: H512,
    ) -> Result<(), Error> {
        let query = format!(
            "{}/{}/{}/{}",
            document_key_id,
            signed_document_key_id
                .trim_matches(CHARS_TO_TRIM)
                .trim_start_matches("0x"),
            serde_json::to_string_pretty(&common_point)?
                .trim_matches(CHARS_TO_TRIM)
                .trim_start_matches("0x"),
            serde_json::to_string_pretty(&encryption_point)?
                .trim_matches(CHARS_TO_TRIM)
                .trim_start_matches("0x")
        );

        let url = self.url.join(&query)?;
        let result = parse_response(
            self.client
                .post(url)
                .send()
                .await
                .map_err(|err| Error::Reqwest(err))?,
        )
        .await?;
        if result == "" {
            Ok(())
        } else {
            Err(Error::Unknown(result))
        }
    }

    pub async fn get_document_key(
        &self,
        document_key_id: &str,
        signed_document_key_id: &Data,
    ) -> Result<DecryptionKeys, Error> {
        let query = format!(
            "{}/{}",
            document_key_id,
            signed_document_key_id
                .trim_matches(CHARS_TO_TRIM)
                .trim_start_matches("0x")
        );

        let url = self.url.join(&query)?;
        let result = parse_response(
            self.client
                .get(url)
                .send()
                .await
                .map_err(|err| Error::Reqwest(err))?,
        )
        .await?;
        Ok(serde_json::from_str(&result)?)
    }
}

async fn parse_response(response: reqwest::Response) -> Result<Data, Error> {
    if !response.status().is_success() {
        let body = response.text().await?;

        if body.contains("Server key with this ID is already generated") {
            return Err(Error::KeyAlreadyGenerated.into());
        } else if body.contains("Document key with this ID is already stored") {
            return Err(Error::KeyAlreadyStored.into());
        } else if body.contains("Server key with this ID is not found") {
            return Err(Error::KeyNotFound.into());
        } else {
            return Err(Error::Unknown(body).into());
        }
    }

    let text = response.text().await?;
    Ok(text.trim_matches(CHARS_TO_TRIM).into())
}
