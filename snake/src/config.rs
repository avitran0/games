use api::serde::{Deserialize, Serialize};

#[derive(Default, Serialize, Deserialize)]
#[serde(crate = "api::serde")]
pub struct Config {
    pub snake: usize,
}
