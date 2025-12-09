use crate::Result;
use crate::error::Error;
use dotenvy::dotenv;
use std::env;
use std::str::FromStr;
use std::sync::OnceLock;

pub fn config() -> &'static Config {
    static INSTANCE: OnceLock<Config> = OnceLock::new();

    INSTANCE.get_or_init(|| {
        Config::load_from_env().unwrap_or_else(|err| {
            panic!("FATAL - WHILE LOADING Config -cause: {:?}", err);
        })
    })
}

#[allow(dead_code)]
#[allow(non_snake_case)]
pub struct Config {
    pub FUNNEL: i64,
    // --TG
    pub ADMIN_ID: i64,
    pub TG_GROUP_ID: i64,
    // -- DB
    pub DB_URL: String,
    // -- AmoCRM
    pub AMO_CITY_ACCOUNT: String,
    pub AMO_CITY_TOKEN: String,
    pub AMO_FORMAT_ACCOUNT: String,
    pub AMO_FORMAT_TOKEN: String,
    // -- Profitbase
    pub PROF_CITY_ACCOUNT: String,
    pub PROF_CITY_API_KEY: String,
    pub PROF_FORMAT_ACCOUNT: String,
    pub PROF_FORMAT_API_KEY: String,
    // -- Schedule for workers
    pub SCHEDULE: String,
    pub DEADLINE_SCHEDULE: String,
    // -- Mailer
    pub SMTP_SERVER: String,
    pub SMTP_PORT: u16,
    pub FROM: String,
    pub LOGIN: String,
    pub PASSWORD: String,
    pub RECEIVERS: String,
}

impl Config {
    fn load_from_env() -> Result<Config> {
        dotenv().expect("dotenv init failed");
        Ok(Config {
            FUNNEL: get_env_as_parse("FUNNEL")?,
            ADMIN_ID: get_env_as_parse("TG_HANMASTER_ID")?,
            TG_GROUP_ID: get_env_as_parse("TG_GROUP_ID")?,
            DB_URL: get_env("DB_URL")?,
            AMO_CITY_ACCOUNT: get_env("AMO_CITY_ACCOUNT")?,
            AMO_CITY_TOKEN: get_env("AMO_CITY_TOKEN")?,
            AMO_FORMAT_ACCOUNT: get_env("AMO_FORMAT_ACCOUNT")?,
            AMO_FORMAT_TOKEN: get_env("AMO_FORMAT_TOKEN")?,
            PROF_CITY_ACCOUNT: get_env("PROF_CITY_ACCOUNT")?,
            PROF_CITY_API_KEY: get_env("PROF_CITY_API_KEY")?,
            PROF_FORMAT_ACCOUNT: get_env("PROF_FORMAT_ACCOUNT")?,
            PROF_FORMAT_API_KEY: get_env("PROF_FORMAT_API_KEY")?,
            SCHEDULE: get_env("SCHEDULE")?,
            DEADLINE_SCHEDULE: get_env("DEADLINE_SCHEDULE")?,
            SMTP_SERVER: get_env("SMTP_SERVER")?,
            SMTP_PORT: get_env_as_parse("SMTP_PORT")?,
            FROM: get_env("FROM")?,
            LOGIN: get_env("LOGIN")?,
            PASSWORD: get_env("PASSWORD")?,
            RECEIVERS: get_env("RECEIVERS")?,
        })
    }
}

fn get_env(name: &'static str) -> Result<String> {
    env::var(name).map_err(|_| Error::ConfigMissingEnv(name))
}

fn get_env_as_parse<T: FromStr>(name: &'static str) -> Result<T> {
    let val = get_env(name)?;
    val.parse::<T>().map_err(|_| Error::ConfigWrongFormat(name))
}
