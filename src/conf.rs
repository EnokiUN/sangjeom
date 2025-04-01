use std::{collections::HashMap, env, fs};

use anyhow::Context;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Conf {
    pub tokens: HashMap<String, String>,
}

impl Conf {
    pub fn new_from_env() -> anyhow::Result<Self> {
        let conf = fs::read_to_string(
            env::var("CONFIG_PATH").unwrap_or_else(|_| "Sangjeom.toml".to_string()),
        )?;
        toml::from_str(&conf).context("Could not deserialize toml file")
    }
}
