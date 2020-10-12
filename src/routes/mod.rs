pub mod pubkey;
pub mod sign_data;
pub use pubkey::{pubkey, PubkeyResp};
pub use sign_data::{sign_data, SignDataErr, SignDataReq, SignDataResp};
pub mod middleware {
    pub mod pow_ratelimit;
}

pub async fn getRoot() -> Result<impl warp::Reply, warp::Rejection> {
    Ok("Hello world !")
}
