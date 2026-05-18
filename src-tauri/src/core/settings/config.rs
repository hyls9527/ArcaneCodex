#![allow(missing_docs)]
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub key: String,

    pub value: String,
}
