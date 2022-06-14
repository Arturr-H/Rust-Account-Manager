/*- Global allowances -*/
#![allow(
    dead_code,
    unused_variables,
    unused_imports,
    unused_assignments
)]

/*- Imports -*/
use crate::utils;
use crate::user::{
    User,
    generate_uuid,
    check_email
};
use std::{
    ops,
    net::TcpStream,
    collections::HashMap,
    hash::Hash,
    borrow::Borrow
};
use crate::dict::DICTIONARY;
use fastserve::{
    respond,
    ResponseType,
    {
        expect_headers,
        HeaderReturn
    }, parse_headers,
};
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

/*- Statics & Constants -*/
pub(crate) const MONGO_DATABASE_NAME:      &'static str = "fastserve_accounts";
pub(crate) const MONGO_CLIENT_URI_STRING:  &'static str = "mongodb://localhost:27017";

/*- All the functions' required headers.
    Accessing these is done via a function
    that lies somewhere in utils.rs -*/
pub(crate) const REQUIRED_HEADERS: &'static [(&'static str, &[&'static str])] = &[
    ("create_account", &["username", "display_name", "password", "email", "bio", "age"]),
];

/*- Functions -*/
pub(super) fn create_account(
    mut stream : TcpStream,
        request: String,
             _ : HashMap<String, String>
) -> () {
    
    /*- Require some headers to be specified -*/
    let required = utils::get_required_headers("create_account");
    let headers  = parse_headers(request, HeaderReturn::All);
    if !expect_headers(&mut stream, &headers, required) { return; };

    /*- Initialize the user -*/
    let user:User;

    /*- Get the headers -*/
    if let HeaderReturn::Values(headers) = headers {
        /*- Get the values -*/
        user = User {
            username    : headers.get("username").unwrap().to_string(),
            display_name: headers.get("display_name").unwrap().to_string(),
            password    : utils::hash(headers.get("password").unwrap()),
            email       : headers.get("email").unwrap().to_string(),
            bio         : headers.get("bio").unwrap().to_string(),
            age         : headers.get("age").unwrap().parse::<u8>().unwrap(),
            uid         : generate_uuid(),
        };
    }
    /*- If parsing headers was unsuccessful -*/
    else { return respond(&mut stream, 404, None, None); };

    /*- If the email is invalid -*/
    if !check_email(&user.email) {
        return respond(
            &mut stream,
            400,
            Some(ResponseType::Text),
            Some(DICTIONARY.error.invalid.email)
        );
    };

    /*- Establish the mongodb connection -*/
    let collection:Collection<User> = utils::establish_mclient::<User>();

    /*- Check if username already exists -*/
    let username_exists = collection.find(doc!{"username": user.username.clone()}, None).unwrap().next().is_some();
    if username_exists {
        return respond(
            &mut stream,
            409,
            Some(ResponseType::Text),
            Some(DICTIONARY.error.in_use.username)
        );
    };
    
    /*- Check if email already exists -*/
    let email_exists = collection.find(doc!{"email": user.email.clone()}, None).unwrap().next().is_some();
    if email_exists {
        return respond(
            &mut stream,
            409,
            Some(ResponseType::Text),
            Some(DICTIONARY.error.in_use.email)
        );
    };
    
    /*- Insert the document -*/
    collection.insert_one(user, None).ok();

    /*- Respond with a success message -*/
    respond(&mut stream, 200u16, None, None);
}

pub(super) fn all_docs(
    mut stream : TcpStream,
        request: String,
        params : HashMap<String, String>
) -> () {

    /*- Establish a connection -*/
    let collection:Collection<Document> = utils::establish_mclient();

    /*- Get all documents -*/
    let docs = collection.find(None, None)
        .expect("Failed to get all documents.");

    /*- Convert the documents to a string -*/
    let mut docs_str = String::new();

    /*- Iterate over the documents -*/
    for doc in docs {
        docs_str += &format!("{:?}", doc);
    };

    /*- Respond with the string -*/
    respond(&mut stream, 200u16, Some(ResponseType::Text), Some(&docs_str));
}
