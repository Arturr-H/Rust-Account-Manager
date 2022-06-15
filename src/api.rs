/*- Global allowances -*/
#![allow(
    dead_code,
    unused_variables,
    unused_imports,
    unused_assignments
)]

/*- Imports -*/
use crate::utils;
use crate::dict::DICTIONARY;
use serde::{ Serialize, Deserialize };
use jsonwebtoken::{ encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey };
use crate::user::{
    User,
    UserClaims,
    AuthorizationStatus,
    get_expiration_time,
    generate_uuid,
    authenticate,
    check_email,
};
use std::{
    ops,
    net::TcpStream,
    collections::HashMap,
    hash::Hash,
    borrow::Borrow
};
use fastserve::{
    respond,
    ResponseType,
    {
        expect_headers,
        parse_headers,
        HeaderReturn
    },
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
    ("create_account",  &["username", "display_name", "password", "email", "bio", "age"]),
    ("login",           &["email", "password"]),
    ("auth_test",       &["token"]),
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

/*- Login accounts -*/
pub(super) fn login(
    mut stream : TcpStream,
        request: String,
        params : HashMap<String, String>
) -> () {

    /*- Require some headers to be specified -*/
    let required = utils::get_required_headers("login");
    let headers  = parse_headers(request, HeaderReturn::All);
    if !expect_headers(&mut stream, &headers, required) { return; };

    /*- Initialize the user -*/
    let password:String;
    let email:String;

    /*- Get the headers -*/
    if let HeaderReturn::Values(headers) = headers {
        /*- Get the values -*/
        password = headers.get("password").unwrap().to_string();
        email    = headers.get("email").unwrap().to_string();
    }
    /*- If parsing headers was unsuccessful -*/
    else { return respond(&mut stream, 404, None, None); };

    /*- Establish the mongodb connection -*/
    let collection:Collection<User> = utils::establish_mclient::<User>();

    /*- Check if email exists -*/
    let email_exists = collection.find(doc!{"email": email.to_string()}, None).unwrap().next().is_some();
    if !email_exists {
        return respond(
            &mut stream,
            404,
            Some(ResponseType::Text),
            Some(DICTIONARY.error.login)
        );
    };

    /*- Get the user -*/
    let user = collection.find(doc!{"email": email.to_string()}, None).unwrap().next().unwrap().unwrap();

    /*- Check if password is correct -*/
    if &user.password != &utils::hash(&password) {
        return respond(
            &mut stream,
            401,
            Some(ResponseType::Text),
            Some(DICTIONARY.error.login)
        );
    };

    /*- Create the token -*/
    let token = User::create__JWT__token(user);

    /*- Respond with a success message -*/
    respond(
        &mut stream,
        200u16,
        Some(ResponseType::Json),

        /*- Format some JSON -*/
        Some(
            &format!(
                "{}\"token\":\"{}\"{}",
                "{", &token, "}"
            )
        )
    );
}

/*- Authenticate using just a token as header -*/
pub(super) fn auth_test(
    mut stream : TcpStream,
        request: String,
        params : HashMap<String, String>
) -> () {

    /*- Require some headers to be specified -*/
    let required = utils::get_required_headers("auth_test");
    let headers  = parse_headers(request, HeaderReturn::All);
    if !expect_headers(&mut stream, &headers, required) { return; };

    /*- Check the auth availability -*/
    let     authentication_status:AuthorizationStatus = authenticate(headers);
    match   authentication_status {
        AuthorizationStatus::Authorized => 
            (),
        AuthorizationStatus::Unauthorized => 
            return respond(
                &mut stream,
                401u16,
                Some(ResponseType::Text),
                Some(DICTIONARY.error.unauthorized)
            ),
        AuthorizationStatus::Err =>
            return respond(
                &mut stream,
                401u16,
                Some(ResponseType::Text),
                Some(DICTIONARY.error.unauthorized)
            )
    }

    /*- Respond -*/
    return respond(&mut stream, 202, None, None);
}