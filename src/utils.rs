/*- Utility-functions -*/
/*- Global allowances -*/
#![allow(
    dead_code,
    unused_variables,
    unused_imports
)]

/*- Constants -*/


/*- Imports -*/
use crate::api::{ MONGO_CLIENT_URI_STRING, REQUIRED_HEADERS };
use fastserve::HeaderReturn;
use crate::user::User;
use sha3::{ Digest, Sha3_256 };
use mongodb::{
    bson::{
        doc,
        Document
    },
    sync::{
        Client,
        Collection,
        Database
    },
};

/*- Quick way of establishing a connection with the mongo client -*/
pub(super) fn establish_mclient<Type__>() -> Collection<Type__> {
    /*- Establish the mongodb connection -*/
    let client:Client = Client::with_uri_str(
        MONGO_CLIENT_URI_STRING
    ).expect("Failed to initialize standalone client.");

    /*- Get the database -*/
    let db:Database = client.database("test");

    /*- Get the collection -*/
    let collection:Collection<Type__> = db.collection::<Type__>("test");

    /*- Return the collection -*/
    collection
}

/*- Most endpoints will require headers, and
    the required headers will be stored in an
    array that might be difficult to search in.
    This function makes that process easier -*/
pub(super) fn get_required_headers(name:&'static str) -> Vec<String> {
    /*- Iterate over all and try find a matching function -*/
    for (key, value) in REQUIRED_HEADERS {
        if key == &name { return value.iter().map(|e| e.to_string()).collect::<Vec<String>>().to_vec(); };
    };

    /*- If no match was found, return an empty array -*/
    return [].to_vec();
}


/*- Hash a string using the SHA-3 algorithm -*/
pub(super) fn hash(value:&str) -> String {
    /*- Hash the string -*/
    let mut hasher = Sha3_256::new();

    /*- Hash the string -*/
    hasher.update(value);

    /*- Return the hash -*/
    return format!("{:x}", hasher.finalize());
}

