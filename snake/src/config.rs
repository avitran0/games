use api::{
    SaveName,
    serde::{Deserialize, Serialize},
};

use crate::snakes::Snake;

#[derive(Default, Serialize, Deserialize)]
#[serde(crate = "api::serde")]
pub struct Config {
    pub snake: Snake,
}

pub const CONFIG_NAME: SaveName = api::save_name("snake");
