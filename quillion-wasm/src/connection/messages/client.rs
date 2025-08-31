use serde::Serialize;

#[derive(Serialize, Debug)]
#[serde(tag = "action")]
pub enum ClientMessage<'a> {
    #[serde(rename = "callback")]
    Callback { id: &'a str },
    #[serde(rename = "navigate")]
    Navigate { path: &'a str },
    #[serde(rename = "public_key")]
    PublicKey { key: String },
    #[serde(rename = "encrypted_message")]
    EncryptedMessage { data: String, nonce: String },
    #[serde(rename = "client_error")]
    ClientError { error: String },
}
