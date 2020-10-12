use ed25519_dalek::{
    self as ed25d, Keypair, PublicKey, Signature, SignatureError, Signer, Verifier,
};
use rand::rngs::OsRng;
use std::fs;
use std::path::PathBuf;

#[derive(Debug)]
pub struct KeyPair(ed25519_dalek::Keypair);
impl KeyPair {
    pub fn generate() -> Self {
        Self(Keypair::generate(&mut OsRng {}))
    }
    pub fn pubkey(&self) -> PublicKey {
        self.0.public
    }
    pub fn sign(&self, message: &[u8]) -> [u8; 64] {
        self.0.sign(&message).to_bytes()
    }
    pub fn verify(&self, message: &[u8], sig: impl AsRef<[u8]> + Clone + Sized) -> bool {
        if sig.as_ref().len() != 64 {
            return false;
        }
        use std::convert::TryFrom;
        let sig = match Signature::try_from(sig.as_ref()) {
            Ok(sig) => sig,
            Err(_) => return false,
        };
        self.0.public.verify(message, &sig).is_ok()
    }

    fn to_file(&self, keyfile: &PathBuf) -> Result<&Self, KpErr> {
        let dir = keyfile.parent().ok_or(KpErr::NoParentDir)?;
        fs::create_dir_all(dir)?;
        fs::write(keyfile, self.to_str())?;
        Ok(self)
    }
    fn from_file(keyfile: &PathBuf) -> Result<Self, KpErr> {
        let content_str = fs::read_to_string(keyfile)?;
        Ok(Self::from_str(&content_str)?)
    }
    pub fn from_file_or_new(keyfile: &PathBuf) -> Result<Self, KpErr> {
        match Self::from_file(&keyfile) {
            Ok(keys) => Ok(keys),
            Err(_err) => {
                let new_keys = Self::generate();
                new_keys.to_file(&keyfile)?;
                Ok(new_keys)
            }
        }
    }
}
impl AsBytes for KeyPair {
    type Err = KpErr;
    fn to_bytes(&self) -> [u8; 64] {
        self.0.to_bytes()
    }
    fn from_bytes(bytes: &[u8]) -> Result<Self, KpErr> {
        if bytes.len() != 64 {
            return Err(KpErr::BytesLengthErr {
                expected: 64,
                got: bytes.len(),
            });
        }
        Ok(Self(ed25d::Keypair::from_bytes(bytes)?))
    }
}

#[derive(thiserror::Error, Debug, Clone)]
pub enum KpErr {
    #[error("unexpected bytes length: expected: {expected}, got: {got}")]
    BytesLengthErr { expected: usize, got: usize },
    #[error("signature err: {0}")]
    SignatureErr(String),
    #[error(transparent)]
    B64Err(#[from] B64Err),
    #[error("IO err: {0}")]
    IoErr(String),
    #[error("no parent directory")]
    NoParentDir,
}
impl From<SignatureError> for KpErr {
    // for Clone (SignatureError doesn't implement Clone)
    fn from(e: SignatureError) -> Self {
        Self::SignatureErr(std::error::Error::to_string(&e))
    }
}
impl From<std::io::Error> for KpErr {
    // for Clone (std::io::Error doesn't implement Clone)
    fn from(e: std::io::Error) -> Self {
        Self::IoErr(std::error::Error::to_string(&e))
    }
}

pub trait AsBytes: Sized {
    type Err;
    fn to_bytes(&self) -> [u8; 64];
    fn from_bytes(bytes: &[u8]) -> Result<Self, Self::Err>;
}
pub trait B64: Sized {
    type Err;
    fn to_str(&self) -> String;
    fn from_str(s: &str) -> Result<Self, Self::Err>;
}
#[derive(thiserror::Error, Debug, Clone)]
pub enum B64Err {
    #[error("base64 decode err: {0}")]
    Base64DecodeErr(#[from] base64::DecodeError),
}
impl<T: AsBytes> B64 for T
where
    <Self as AsBytes>::Err: From<B64Err>,
{
    type Err = <Self as AsBytes>::Err;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::from_bytes(
            &base64::decode(s).map_err(B64Err::from)?.to_vec(),
        )?)
    }
    fn to_str(&self) -> String {
        base64::encode(self.to_bytes().to_vec())
    }
}
