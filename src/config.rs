use anyhow::{Context, Error as AnyErr, Result};
use config::{Config as ConfigLoader, Environment, File};
use std::borrow::Cow;
use std::path::PathBuf;
//
use crate::utils::crypto_sign::KeyPair;

lazy_static::lazy_static! {
    #[derive(Debug)]
    static ref CONFIG: Config<'static> = Config::load().expect("failed loading config");
    static ref PG_DSN: String = CONFIG.pg_dsn().expect("failed loading pg_dsn").to_string();
    static ref KEYPAIR_SIGN: KeyPair = KeyPair::from_file_or_new(&CONFIG.keyfile_path).expect("failed getting keypair for signing");
}

pub fn pg_dsn<'a>() -> &'a str {
    &PG_DSN
}
pub fn port() -> u16 {
    CONFIG.http_port
}
pub fn keypair<'a>() -> &'a KeyPair {
    &KEYPAIR_SIGN
}

#[derive(Deserialize, Serialize, Clone, Debug)]
struct Config<'a> {
    rust_log: String,
    rust_backtrace: String,
    http_port: u16,
    #[serde(borrow, rename = "database_url")]
    pg_dsn: Option<Cow<'a, str>>,
    #[serde(borrow, rename = "postgres_user")]
    pg_user: Option<Cow<'a, str>>,
    #[serde(borrow, rename = "postgres_password")]
    pg_pass: Option<Cow<'a, str>>,
    #[serde(borrow, rename = "postgres_db")]
    pg_db: Option<Cow<'a, str>>,
    #[serde(borrow, rename = "postgres_host")]
    pg_host: Option<Cow<'a, str>>,
    keyfile_path: PathBuf,
}
impl<'a> Config<'a> {
    fn pg_env_vars(&self) -> Result<Option<(&str, &str, &str, &str)>, AnyErr> {
        match (&self.pg_user, &self.pg_pass, &self.pg_host, &self.pg_db) {
            (Some(u), Some(p), Some(h), Some(db)) => Ok(Some((u, p, h, db))),
            (None, None, None, None) => Ok(None),
            _ => return Err(AnyErr::msg("PG env vars: expected only all or none")),
        }
    }
    fn pg_dsn_from_env_vars(&self) -> Option<String> {
        match (&self.pg_host, &self.pg_user, &self.pg_pass, &self.pg_db) {
            (Some(h), Some(u), Some(p), Some(db)) => {
                Some(format!("postgres://{}:{}@{}/{}", u, p, h, db))
            }
            _ => None,
        }
    }
    fn pg_dsn(&self) -> Result<String, AnyErr> {
        // dbg!(&std::env::vars()); // TODO for debugging github action. keep until solved.
        match &self.pg_dsn {
            Some(url) => Ok(url.to_string()),
            None => self
                .pg_dsn_from_env_vars()
                .context("failed loading pg dsn from env vars"),
        }
    }
    fn load() -> Result<Self, AnyErr> {
        let mut s = ConfigLoader::new();
        s.set_default("http_port", 8080)?;
        s.set_default("rust_log", "auth-rs-warp=debug")?;
        s.set_default("rust_backtrace", 1)?;
        s.set_default("keyfile_path", "./.config/keys/keypair_sign")?;
        s.merge(File::with_name("./.config/api_config").required(false))?;
        s.merge(Environment::new())?;

        let config = s.try_into::<Config>().context("failed parsing")?;
        config.validate()?;

        std::env::set_var("RUST_LOG", &config.rust_log);
        std::env::set_var("RUST_BACKTRACE", &config.rust_backtrace);
        Ok(config)
    }
    fn validate(&self) -> Result<(), AnyErr> {
        anyhow::ensure!(self.http_port != 0, "http port can't be 0");
        anyhow::ensure!(self.pg_env_vars().is_ok(), "{}");
        match (self.pg_dsn.as_ref(), self.pg_env_vars()) {
            (Some(dsn), Ok(Some(_))) => {
                anyhow::ensure!(
                    self.pg_dsn_from_env_vars() == Some(dsn.to_string()),
                    "PG_ENV_VARS and DSN must match if both defined"
                );
            }
            _ => {}
        }
        Ok(())
    }
}
