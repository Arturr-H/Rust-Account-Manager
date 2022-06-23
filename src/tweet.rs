/*- Imports -*/
use serde::{ Serialize, Deserialize, de::DeserializeOwned };
use std::default;
use crate::utils;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct Tweet {
    pub owner:String,
    pub content:String,
    pub id:String,
    pub likes:Vec<String>,
    pub unix:u64,
    pub hashtags:Vec<String>,
}

/*- For unwrap-defaulting -*/
impl default::Default for Tweet {
    fn default() -> Self {
        Tweet {
            owner: String::new(),
            content: String::new(),
            id: String::new(),
            likes: vec![],
            unix: utils::get_unix_epoch_time(),
            hashtags: vec![],
        }
    }
}