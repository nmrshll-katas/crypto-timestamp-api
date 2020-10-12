use diesel::{r2d2::ConnectionManager, PgConnection};
use thiserror::Error;

// auto-connect to DB, keep pool global
lazy_static::lazy_static! {
    pub static ref DB_CONN_POOL: Pool = connect_DB();
}

pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;
pub type PooledConnection =
    r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>;

// METHODS
pub fn get() -> Result<PooledConnection, DbConnErr> {
    Ok(DB_CONN_POOL.get()?)
}

fn connect_DB() -> Pool {
    let manager = ConnectionManager::<PgConnection>::new(crate::config::pg_dsn());
    let pool: Pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    return pool;
}

#[derive(Error, Debug)]
pub enum DbConnErr {
    #[error("r2d2 err: {0}")]
    R2d2(#[from] r2d2::Error),
}
