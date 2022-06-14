/*- Imports -*/
use std::fmt;
use uuid::Uuid;
use serde::{ Serialize, Deserialize };
use regex;

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct User {
    pub username    : String,
    pub display_name: String,
    pub password    : String,
    pub email       : String,
    pub bio         : String,
    pub uid         : String,
    pub age         : u8,
}

/*- Fcuntion implementations -*/
impl Default for User {
    fn default() -> Self {
        User {
            username    : String::new(),
            display_name: String::new(),
            password    : String::new(),
            email       : String::new(),
            bio         : String::new(),
            uid         : String::new(),
            age         : 0,
        }
    }
}

/*- For printing / debugging -*/
impl fmt::Debug for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "User {{ username: {}, display_name: {}, password: {}, email: {}, bio: {}, uid: {}, age: {} }}",
            self.username, self.display_name, self.password, self.email, self.bio, self.uid, self.age
        )
    }
}

/*- Utility functions -*/
pub fn generate_uuid() -> String {
    Uuid::new_v4().as_hyphenated().to_string()
}

/*- If email is valid -*/
pub fn check_email(email:&str) -> bool {

    /*-
     *- Check if the email is valid
    -*  Must contain ["@", "."]
    -*/
    let email_regex = regex::Regex::new(r"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$").unwrap();

    /*- Return bool if the email is valid -*/
    email_regex.is_match(email)
}