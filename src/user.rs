/*- Global allowings -*/
#![allow(non_snake_case)]

/*- Imports -*/
use regex;
use uuid::Uuid;
use fastserve::HeaderReturn;
use std::{ time, thread, fmt, collections::HashMap };
use serde::{ Serialize, Deserialize, de::DeserializeOwned };
use jsonwebtoken::{ encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey, TokenData };

use crate::safe_user::SafeUser;

/*- Constants -*/
const SECRET_KEY:&str = "Secret123";

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct User {
    pub username    : String,
    pub displayname : String,
    pub password    : String,
    pub email       : String, 
    pub uid         : String,
    pub suid        : String,
    pub age         : u8,
}

/*- The default users claims -*/
#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct UserClaims {
    pub username: String,
    pub uid     : String,
    pub suid    : String,
    pub exp     : usize,
}

/*- Fcuntion implementations -*/
impl Default for User {
    fn default() -> Self {
        User {
            username    : String::new(),
            displayname : String::new(),
            password    : String::new(),
            email       : String::new(),
            uid         : String::new(),
            suid        : String::new(),
            age         : 0,
        }
    }
}

/*- For printing / debugging -*/
impl fmt::Debug for User {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "User {{ username: {}, displayname : {}, password: {}, email: {}, uid: {}, age: {} }}",
            self.username, self.displayname , self.password, self.email, self.uid, self.age
        )
    }
}

impl User {

    /*- Create a JWT token -*/
    pub fn create__JWT__token(user:User) -> String {
        /*- Get the claims -*/
        let user_claims = UserClaims {
            username: user.username.clone(),
            uid     : user.uid.clone(),
            suid    : user.suid.clone(),
            exp     : get_expiration_time(),
        };

        /*- Encode the claims -*/
        let token = encode(
            &Header::default(),
            &user_claims,
            &EncodingKey::from_secret(SECRET_KEY.as_ref())
        ).expect("Failed to encode token");

        /*- Return the token -*/
        token
    }

    /*- Decode a JWT token -*/
    pub fn decode__JWT__token(token:&str) -> Result<UserClaims, ()> {
        /*- Decode the token -*/
        let token = decode::<UserClaims>(
            &token,
            &DecodingKey::from_secret(
                SECRET_KEY.as_ref()
            ),
            &Validation::default()
        );

        /*- Check token decode status and return the token claims / data -*/
        return match token {
            Ok(token) => Ok(token.claims),
            Err(_) => Err(())
        };
    }

    /*- Convert to SafeUser -*/
    pub fn to_safe(user:User) -> SafeUser {
        return SafeUser {
            username    : user.username,
            displayname : user.displayname,
            suid        : user.suid,
            age         : user.age,
        }
    }
}

/*- Utility functions -*/
pub fn generate_uuid() -> String {
    Uuid::new_v4().as_hyphenated().to_string()
}

/*- Secure user identification -*/
pub fn generate_suid() -> String {
    Uuid::new_v4().as_simple().to_string()
}

/*- If email is valid -*/
pub fn check_email(email:&str) -> bool {

    /*- Check if the email is valid Must contain ["@", "."] -*/
    let email_regex:regex::Regex = regex::Regex::new(r"^[a-zA-Z0-9.!#$%&'*+/=?^_`{|}~-]+@[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(?:\.[a-zA-Z0-9](?:[a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)*$").unwrap();

    /*- Return bool if the email is valid -*/
    email_regex.is_match(email)
}

/*- Get the expiration time -*/
pub fn get_expiration_time() -> usize {

    /*- Get the current time -*/
    let now = time::SystemTime::now();

    /*- Get the expiration time -*/
    let expiration_time = now + time::Duration::from_secs(60*60*24*30);

    /*- Convert the expiration time to unix time -*/
    expiration_time.duration_since(time::UNIX_EPOCH).unwrap().as_secs() as usize

}

/*- Fully check if user is authorized, and
    return a bool dependent on if they are -*/
pub(crate) fn authenticate(headers:HeaderReturn) -> AuthorizationStatus {
    /*- Initialize the user -*/
    let token:String;

    /*- Get the headers -*/
    if let HeaderReturn::Values(headers) = headers {
        /*- Get the values -*/
        token = match headers.get("authorization") {
            Some(token) => token,
            None        => {
                match headers.get("Authorization") {
                    Some(token) => token,
                    None        => { return AuthorizationStatus::Err; }
                }
            }
        }.to_string().replace("Bearer ", "");
    }
    /*- If parsing headers was unsuccessful -*/
    else { return AuthorizationStatus::Err; };

    /*- Decode the token -*/
    let user_claims = User::decode__JWT__token(&token);

    /*- Return -*/
    match user_claims {
        Ok(u)   => return AuthorizationStatus::Authorized(u),
        Err(_)  => return AuthorizationStatus::Unauthorized
    }
}

#[derive(Debug)]
pub(crate) enum AuthorizationStatus{
    Authorized(UserClaims),
    Unauthorized,
    Err,
}
