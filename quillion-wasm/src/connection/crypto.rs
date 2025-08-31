use crate::utils::log;
use hkdf::Hkdf;
use rand::RngCore;
use rand::rngs::OsRng;
use sha2::Sha256;
use std::cell::RefCell;
use std::rc::Rc;

use crate::error::AppError;
use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit, Payload},
};
use base64::{Engine as _, engine::general_purpose};
use x25519_dalek::{PublicKey, StaticSecret};

#[derive(Clone)]
pub struct Crypto {
    x25519_secret: StaticSecret,
    x25519_public: PublicKey,
    pub aes_cipher: Rc<RefCell<Option<Aes256Gcm>>>,
}

impl Crypto {
    pub fn new() -> Self {
        let mut rng = OsRng;
        let x25519_secret = StaticSecret::random_from_rng(&mut rng);
        let x25519_public = PublicKey::from(&x25519_secret);

        Self {
            x25519_secret,
            x25519_public,
            aes_cipher: Rc::new(RefCell::new(None)),
        }
    }

    pub fn public_key_b64(&self) -> String {
        let public_key_bytes = self.x25519_public.as_bytes();
        general_purpose::STANDARD.encode(public_key_bytes)
    }

    pub fn derive_shared_secret(&self, server_public_key_b64: &str) -> Result<(), AppError> {
        match general_purpose::STANDARD.decode(server_public_key_b64) {
            Ok(server_public_key_bytes) => {
                if server_public_key_bytes.len() == 32 {
                    let mut server_public_key_array = [0u8; 32];
                    server_public_key_array.copy_from_slice(&server_public_key_bytes);

                    let server_public = PublicKey::from(server_public_key_array);
                    let shared_secret = self.x25519_secret.diffie_hellman(&server_public);
                    let hk = Hkdf::<Sha256>::new(None, shared_secret.as_bytes());
                    let mut aes_key = [0u8; 32];
                    hk.expand(b"quillion-aes-key", &mut aes_key)
                        .expect("hkdf expansion - fail");

                    let cipher =
                        Aes256Gcm::new_from_slice(&aes_key).expect("aes key derivation - failed");

                    *self.aes_cipher.borrow_mut() = Some(cipher);
                    Ok(())
                } else {
                    Err(AppError::CryptoError(format!(
                        "Invalid public key, length: {}",
                        server_public_key_bytes.len()
                    )))
                }
            }
            Err(e) => Err(AppError::CryptoError(format!(
                "Base64 decode error: {:?}",
                e
            ))),
        }
    }

    pub fn encrypt(&self, message: &str) -> Option<(String, String)> {
        if let Some(cipher) = self.aes_cipher.borrow().as_ref() {
            let mut rng = OsRng;
            let mut nonce_bytes = [0u8; 12];
            rng.fill_bytes(&mut nonce_bytes);
            let nonce = Nonce::from_slice(&nonce_bytes);

            let payload = Payload {
                msg: message.as_bytes(),
                aad: &[],
            };

            match cipher.encrypt(nonce, payload) {
                Ok(ciphertext) => Some((
                    general_purpose::STANDARD.encode(&ciphertext),
                    general_purpose::STANDARD.encode(&nonce_bytes),
                )),
                Err(e) => {
                    log(&format!("Encryption error: {:?}", e));
                    None
                }
            }
        } else {
            None
        }
    }

    pub fn decrypt(&self, encrypted_payload_b64: &str, nonce_b64: &str) -> Option<String> {
        if let Some(cipher) = self.aes_cipher.borrow().as_ref() {
            match (
                general_purpose::STANDARD.decode(encrypted_payload_b64),
                general_purpose::STANDARD.decode(nonce_b64),
            ) {
                (Ok(encrypted_bytes), Ok(nonce_bytes)) => {
                    if nonce_bytes.len() == 12 {
                        let nonce = Nonce::from_slice(&nonce_bytes);
                        let payload = Payload {
                            msg: &encrypted_bytes,
                            aad: &[],
                        };

                        match cipher.decrypt(nonce, payload) {
                            Ok(decrypted_bytes) => String::from_utf8(decrypted_bytes).ok(),
                            Err(e) => {
                                log(&format!("Decryption failed: {:?}", e));
                                None
                            }
                        }
                    } else {
                        log(&format!(
                            "Invalid nonce length: {} bytes",
                            nonce_bytes.len()
                        ));
                        None
                    }
                }
                (Err(e), _) => {
                    log(&format!("Base64 decode error for payload: {:?}", e));
                    None
                }
                (_, Err(e)) => {
                    log(&format!("Base64 decode error for nonce: {:?}", e));
                    None
                }
            }
        } else {
            log(&"AES cipher not initialized");
            None
        }
    }
}
