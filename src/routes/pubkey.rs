use ed25519_dalek::PublicKey;
use warp::{reply, Rejection, Reply};

#[derive(Serialize)]
#[cfg_attr(test, derive(Deserialize, Debug))]
pub struct PubkeyResp {
    pub pubkey: PublicKey,
}

pub async fn pubkey() -> Result<impl Reply, Rejection> {
    let pubkey = crate::config::keypair().pubkey();
    let resp = PubkeyResp { pubkey };

    Ok(reply::json(&resp))
}
