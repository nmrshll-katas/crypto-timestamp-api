use chrono::{Local, NaiveDateTime};
use warp::{reply, Rejection, Reply};
//
use super::middleware::pow_ratelimit;
use crate::models::{ModelErr, NewSignedData};
use crate::utils::db_conn::{self, DbConnErr};

#[derive(Debug, Deserialize)]
pub struct SignDataReq {
    // String since HTTP is text only, and we want the server to accept any bytes as data, only encoded as base64
    pub data_base64: String,
    pub pow_proof_base64: String,
}
impl SignDataReq {
    pub fn data_bytes(&self) -> Result<Vec<u8>, SignDataErr> {
        Ok(base64::decode(&self.data_base64).map_err(SignDataErr::B64DecodeBody)?)
    }
    pub fn hash_data(&self) -> Result<blake3::Hash, SignDataErr> {
        Ok(blake3::hash(&self.data_bytes()?))
    }
    pub fn verify_pow(&self) -> Result<bool, SignDataErr> {
        pow_ratelimit::verify_pow(&self.data_base64, &self.pow_proof_base64)
            .map_err(SignDataErr::from)
    }
}

#[derive(Serialize)]
#[cfg_attr(test, derive(Deserialize))]
pub struct SignDataResp {
    pub fields_signed: FieldsSigned,
    // Why base64 ? FieldsSigned is part of the server response, must be text for HTTP, and we want the field name to be self-documenting for clients
    pub signature_base64: String, // Signature over both data and timestamp
}
#[derive(Serialize, Clone)]
#[cfg_attr(test, derive(Deserialize))]
pub struct FieldsSigned {
    // Why base64 ? FieldsSigned is part of the server response, must be text for HTTP, and we want the field name to be self-documenting for clients
    pub data_hash_base64: String,
    pub timestamp: NaiveDateTime,
}
impl FieldsSigned {
    fn hash(&self) -> Result<[u8; 32], SignDataErr> {
        let json_bytes: Vec<u8> =
            serde_json::to_vec(&self).map_err(SignDataErr::SerializeFieldsSigned)?;
        Ok(*blake3::hash(&json_bytes).as_bytes())
    }
    fn sign(&self) -> Result<[u8; 64], SignDataErr> {
        let sig = crate::config::keypair().sign(&self.hash()?);
        Ok(sig)
    }
}

pub async fn sign_data(sd_req: SignDataReq) -> Result<impl Reply, Rejection> {
    // TODO middleware rate-limit with PoW
    let pow_ok = sd_req.verify_pow()?;
    if !pow_ok {
        return Err(SignDataErr::PowRejected)?;
    }

    // hash data
    let data_hash = sd_req.hash_data()?;
    let data_hash_base64 = base64::encode(&data_hash.as_bytes());

    // serialize {data,timestamp} to bytes, sign serialized bytes
    let fields_signed = FieldsSigned {
        data_hash_base64,
        timestamp: Local::now().naive_local(),
    };
    let signature = fields_signed.sign()?;
    let signature_base64 = base64::encode(&signature);

    // create response
    let resp = SignDataResp {
        fields_signed: fields_signed.clone(),
        signature_base64,
    };

    // insert data_hash into db (to disallow signing the same data a second time)
    let db = db_conn::get().map_err(SignDataErr::DbConn)?;
    let new_signed_data = NewSignedData {
        created_at: Some(fields_signed.timestamp),
        data_hash_b64: &fields_signed.data_hash_base64,
    };
    let _signed_data = new_signed_data.insert(&db).map_err(SignDataErr::Model)?;

    Ok(reply::json(&resp))
}

#[derive(Debug, thiserror::Error)]
pub enum SignDataErr {
    #[error("db conn err: {0}")]
    DbConn(#[from] DbConnErr),
    #[error("model err: {0}")]
    Model(#[from] ModelErr),
    #[error("ser err: {0}")]
    SerializeFieldsSigned(serde_json::Error),
    #[error("ser err: {0}")]
    B64DecodeBody(#[from] base64::DecodeError),
    #[error("PoW proof rejected")]
    PowRejected,
}
use pow_ratelimit::PowVerifErr;
impl From<PowVerifErr> for SignDataErr {
    fn from(e: PowVerifErr) -> Self {
        match e {
            PowVerifErr::B64DecodeBody(e) => SignDataErr::B64DecodeBody(e),
            PowVerifErr::B64DecodePowProof(_) => SignDataErr::PowRejected,
            PowVerifErr::Vec8toVec32 => SignDataErr::PowRejected,
        }
    }
}

impl warp::reject::Reject for SignDataErr {}
impl From<SignDataErr> for Rejection {
    fn from(e: SignDataErr) -> Self {
        warp::reject::custom(e)
    }
}
