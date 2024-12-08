pub mod config;
pub mod download;
pub mod load;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct ModelData {
    map: HashMap<String, String>,
}