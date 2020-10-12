#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
extern crate openssl; // for musl (if using alpine). Before diesel macro imports
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate log;
use std::convert::Infallible;
use std::net::SocketAddr;
use warp::{body, get, path, post, Filter, Reply};
//
mod config;
mod errors;
mod models;
mod routes;
mod utils;
use utils::db_conn;

#[cfg(test)]
mod tests {
    mod routes;
}

pub fn router(
) -> impl Filter<Extract = impl Reply, Error = Infallible> + Clone + Send + Sync + 'static {
    (get().and(path::end()).and_then(routes::getRoot))
        .or(get().and(path("health")).and_then(routes::getRoot))
        .or(get().and(path("pubkey")).and_then(routes::pubkey))
        .or(post().and(path("sign_data").and(body::json().and_then(routes::sign_data))))
        .recover(errors::handle_rejection)
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    // auto-loaded: config, logger, db-conn-pool
    pretty_env_logger::init();
    lazy_static::initialize(&db_conn::DB_CONN_POOL);

    let addr: SocketAddr = ([0, 0, 0, 0], config::port()).into();
    info!("Listening on http://{}", addr);
    warp::serve(router()).run(addr).await;
    Ok(())
}
