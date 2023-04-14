use serde::{Deserialize, Serialize};
use sha3::Digest;

pub fn hash_pass(password: String) -> Vec<u8> {
    let mut hasher = sha3::Sha3_224::new();
    hasher.update(password);
    hasher.finalize().as_slice().to_vec()
}

pub type Username = String;
pub type Password = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    username: Username,
    password: Password,
}

impl Credentials {
    pub fn new(username: impl AsRef<str>, pass: impl AsRef<str>) -> Self {
        Self {
            username: username.as_ref().to_string(),
            password: pass.as_ref().to_string(),
        }
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn password(&self) -> &str {
        &self.password
    }

    pub fn consume(self) -> (Username, Password) {
        (self.username, self.password)
    }

    pub fn password_digest(&self) -> Vec<u8> {
        hash_pass(self.password.clone())
    }
}
