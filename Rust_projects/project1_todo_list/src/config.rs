use std::env;

pub struct Config {
    #[allow(dead_code)]
    pub bot_token: String,
}

impl Config {
    pub fn from_env() -> Result<Self, env::VarError> {
        Ok(Self {
            bot_token: env::var("TELOXIDE_TOKEN")?,
        })
    }
}