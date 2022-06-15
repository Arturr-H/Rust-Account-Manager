/*- Global allowances -*/
#![allow(
    dead_code,
    unused_variables,
    unused_imports,
    unused_assignments
)]

/*- Imports -*/
use crate::{utils, safe_user::SafeUser};
use crate::dict::DICTIONARY;
use fastserve::ResponseTypeImage;
use serde::{ Serialize, Deserialize };
use chunked_transfer::Encoder;
use serde_json;
use image;
use jsonwebtoken::{ encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey };
use crate::user::{
    User,
    UserClaims,
    AuthorizationStatus,
    get_expiration_time,
    generate_uuid,
    generate_suid,
    authenticate,
    check_email,
};
use std::io::{Read, Write};
use std::{
    ops,
    net::TcpStream,
    collections::HashMap,
    hash::Hash,
    borrow::Borrow,
    default,
    path::Path,
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
        Database, Cursor
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
            suid        : generate_suid(),
        };
    }
    /*- If parsing headers was unsuccessful -*/
    else { return respond(&mut stream, 404, None, None); };

    /*- If the email is invalid -*/
    if !check_email(&user.email) {
        return respond(
            &mut stream,
            400,
            Some((
                ResponseType::Text,
                DICTIONARY.error.invalid.email
            )),
            None
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
            Some((
                ResponseType::Text,
                DICTIONARY.error.in_use.username
            )),
            None
        );
    };
    
    /*- Check if email already exists -*/
    let email_exists = collection.find(doc!{"email": user.email.clone()}, None).unwrap().next().is_some();
    if email_exists {
        return respond(
            &mut stream,
            409,
            Some((
                ResponseType::Text,
                DICTIONARY.error.in_use.email
            )),
            None
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
            Some((
                ResponseType::Text,
                DICTIONARY.error.login
            )),
            None
        );
    };

    /*- Get the user -*/
    let user = collection.find(doc!{"email": email.to_string()}, None).unwrap().next().unwrap().unwrap();

    /*- Check if password is correct -*/
    if &user.password != &utils::hash(&password) {
        return respond(
            &mut stream,
            401,
            Some((
                ResponseType::Text,
                DICTIONARY.error.login
            )),
            None
        );
    };

    /*- Create the token -*/
    let token = User::create__JWT__token(user);

    /*- Respond with a success message -*/
    respond(
        &mut stream,
        200u16,
        Some((
            ResponseType::Json,
    
            /*- Format some JSON -*/
            &format!(
                "{}\"token\":\"{}\"{}",
                "{", &token, "}"
            )
        )),
        None
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
                Some((
                    ResponseType::Text,
                    DICTIONARY.error.unauthorized
                )),
                None
            ),
        AuthorizationStatus::Err =>
            return respond(
                &mut stream,
                401u16,
                Some((
                    ResponseType::Text,
                    DICTIONARY.error.unauthorized
                )),
                None
            )
    }

    /*- Respond -*/
    return respond(&mut stream, 202, None, None);
}

/*- Get other user's profile -*/
pub(crate) fn profile_data(
    mut stream : TcpStream,
        request: String,
        params : HashMap<String, String>
) -> () {
    
    /*- No headers required, the requested users'
        suid is specified in the URL-params -*/
    let request_suid:&str = &params
        .get("suid")
        .unwrap_or(
            &"".to_string()
        ).to_string();

    /*- Establish the mongodb connection -*/
    let collection:Collection<User> = utils::establish_mclient::<User>();

    /*- Check if the user exists -*/
    let user_exists = collection.find(
        doc!{
            "suid": request_suid.to_string()
        }, None
    );

    /*- Get the userdata or respond 404 if not available,
        and convert the user to a SafeUser for safety  -*/
    let user_data:SafeUser = User::to_safe(match user_exists {
        Ok(mut async_cursor) => {
            match async_cursor.next() {
                Some(user_data) => match user_data {
                    Ok(user_data) => user_data,
                    Err(_) => return respond(&mut stream, 404, None, None)
                },
                None => return respond(&mut stream, 404, None, None)
            }
        },
        Err(_) => return respond(&mut stream, 404, None, None)
    });

    /*- Respond with the userdata -*/
    respond(
        &mut stream,
        200u16,
        Some((
            ResponseType::Json,
            &serde_json::to_string(
                &user_data
            ).unwrap()
        )),
        None
    );
}

/*- Respond with a png image -*/
pub(crate) fn profile_image(
    mut stream : TcpStream,
        request: String,
        params : HashMap<String, String>
) -> () {
    
    /*- Get the param named 'profile_image' -*/
    let profile_image:&str = &params
        .get("profile_image")
        .unwrap_or(
            &"".to_string()
        ).to_string();

    /*- Search for the image in the static/ dir -*/
    let image_path:String  = format!("uploads/{}.jpg", profile_image);
    let pfp_not_found:&str = &"static/images/default-user.jpg";

    /*- Buffers -*/
    let mut buf = Vec::new();
    let file = std::fs::File::open(image_path);

    /*- Error handling -*/
    let mut file = match file {
        Ok(file) => file,
        Err(_) => match std::fs::File::open(pfp_not_found) {
            Ok(file) => file,
            Err(_) => return respond(&mut stream, 404u16, None, None)
        }
    };
    
    file.read_to_end(&mut buf).unwrap_or_default();
    
    /*- Encode -*/
    let mut encoded = Vec::new();
    {
        let mut encoder = Encoder::with_chunks_size(&mut encoded, 64);
        encoder.write_all(&buf).unwrap_or_default();
    }

    /*- Create the response -*/
    let headers = [
        "HTTP/1.1 200 OK",
        "Content-type: image/png",
        "Transfer-Encoding: chunked",
        "\r\n"
    ];
    let mut response = headers.join("\r\n")
        .to_string()
        .into_bytes();
        response.extend(encoded);

    /*- Respond with the image -*/
    stream.write(&response).unwrap_or_default();
}
        