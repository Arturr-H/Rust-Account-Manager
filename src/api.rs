/*- Global allowances -*/
#![allow(
    dead_code,
    unused_variables,
    unused_imports,
    unused_assignments
)]

/*- Imports -*/
use crate::{ utils, safe_user::SafeUser, tweet::Tweet };
use crate::dict::{ DICTIONARY, get_error_code };
use fastserve::ResponseTypeImage;

use serde::{ Serialize, Deserialize };
use serde_json;
use chunked_transfer::{ Encoder, Decoder };
use image;
use jsonwebtoken::{ encode, decode, Header, Algorithm, Validation, EncodingKey, DecodingKey };
use regex::Regex;
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
use std::{
    io::{
        Read,
        Write
    },
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
pub(crate) const MONGO_CLIENT_URI_STRING:  &'static str = "mongodb://mongo:27017";

/*- All the functions' required headers.
    Accessing these is done via a function
    that lies somewhere in utils.rs -*/
pub(crate) const REQUIRED_HEADERS: &'static [(&'static str, &[&'static str])] = &[
    ("create_account",  &["username", "displayname", "password", "email"]),
    ("login",           &["email", "password"]),
    ("auth_test",       &["Authorization"]),
    ("tweet",           &["Authorization", "content"]),
    ("like",            &["Authorization", "tweet"]),
];

/*- Functions -*/
pub(super) fn create_account(
    mut stream : TcpStream,
        request: String,
             _ : HashMap<String, String>
) -> () {
    
    /*- Require some headers to be specified -*/
    let required = utils::get_required_headers("create_account");
    println!("required: {:?}", required);
    println!("request: {}", request);
    let headers  = parse_headers(request, HeaderReturn::All);
    println!("headers: {:?}", headers);
    if !expect_headers(&mut stream, &headers, required) { return; };

    /*- Initialize the user -*/
    let user:User;

    println!("Hej");

    /*- Get the headers -*/
    if let HeaderReturn::Values(headers) = headers {
        /*- Get the values -*/
        user = User {
            username    : headers.get("username").unwrap().to_string(),
            displayname : headers.get("displayname").unwrap().to_string(),
            password    : utils::hash(headers.get("password").unwrap()),
            email       : headers.get("email").unwrap().to_string(),
            age         : 0,
            uid         : generate_uuid(),
            suid        : generate_suid(),
        };
    }
    /*- If parsing headers was unsuccessful -*/
    else { return respond(&mut stream, 404, None, None); };
    println!("{:?}", user);


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
    let collection:Collection<User> = utils::establish_mclient::<User>("test");

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
    let collection:Collection<User> = utils::establish_mclient::<User>("test");

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
    let token = User::create__JWT__token(user.clone());

    /*- Respond with a success message -*/
    respond(
        &mut stream,
        200u16,
        Some((
            ResponseType::Json,
    
            /*- Format some JSON -*/
            &format!(
                "{}\"token\":\"{}\",\"suid\":\"{}\"{}",
                "{", &token, &user.suid, "}"
            )
        )),
        None
    );
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
    let collection:Collection<User> = utils::establish_mclient::<User>("test");

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

/*- Get a users profile image -*/
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

/*- Get a feed -*/
pub(crate) fn feed(
    mut stream : TcpStream,
        request: String,
        params : HashMap<String, String>
) -> () {

    /*- Establish the mongodb connection -*/
    let collection:Collection<Tweet> = utils::establish_mclient::<Tweet>("tweets");
    
    /*- Get all -*/
    let mut all_tweets = match collection.find(None, None) {
        Ok(tweets) => tweets,
        _ => return respond(&mut stream, 404u16, None, None),
    }.into_iter()
        .map(|e| e.unwrap_or_default())
        .collect::<Vec<_>>();

    /*- Sort them based on the one with the most suid:s in the like vector -*/
    all_tweets.sort_by(|a, b| {
        let a_likes = a.likes.len();
        let b_likes = b.likes.len();
        b_likes.cmp(&a_likes)
    });

    /*- What we'll send back -*/
    let response_json:&str = &serde_json::to_string(&all_tweets).unwrap();

    /*- Respond -*/
    respond(
        &mut stream,
        200u16,
        Some((
            ResponseType::Json,
            response_json
        )),
        None
    )
}

/*- Create a tweet -*/
pub(crate) fn tweet(
    mut stream : TcpStream,
        request: String,
        params : HashMap<String, String>

) -> () {

    /*- Require some headers to be specified -*/
    let required = utils::get_required_headers("tweet");
    let headers  = parse_headers(request.clone(), HeaderReturn::All);

    println!("{:?}", required);
    println!("{:?}", headers);

    if !expect_headers(&mut stream, &headers, required) { return; };

    /*- Check the auth availability -*/
    let authentication_status:AuthorizationStatus = authenticate(headers.clone());
    let user_claims:UserClaims = match authentication_status {
        AuthorizationStatus::Authorized(v) => v,
        AuthorizationStatus::Unauthorized => 
            return respond(&mut stream, 401u16, Some((ResponseType::Text, DICTIONARY.error.unauthorized)), None),
        AuthorizationStatus::Err =>
            return respond(&mut stream, 401u16, Some((ResponseType::Text, DICTIONARY.error.unauthorized)), None)
    };

    /*- Get the "content" header -*/
    let content:String;

    /*- Get the headers -*/
    if let HeaderReturn::Values(headers) = headers {
        /*- Get the values -*/
        content = headers.get("content").unwrap().to_string();
    }else {
        /*- Return an error -*/
        return respond(&mut stream, 400u16, None, None);
    }

    /*- Content will be a string of ascii numbers separated by commas (utf16) -*/
    let content:String = String::from_utf16(
        &content.split(",")
        .map(|e| e.parse::<u16>()
        .unwrap_or_default()
    ).collect::<Vec<_>>())
        .unwrap_or_default();

    /*- Get the hashtags from the tweet -*/
    let hashtags:Vec<String> = Regex::new(r"#\w+")
        .unwrap()
        .captures_iter(&content)
        .map(|e| e.get(0).unwrap().as_str()[1..].to_string())
        .collect::<Vec<_>>();

    /*- Establish the mongodb connection -*/
    let collection:Collection<Tweet> = utils::establish_mclient::<Tweet>("tweets");

    /*- Create the tweet -*/
    let tweet:Tweet = Tweet {
        content,
        owner: user_claims.suid,
        id   : generate_suid(),
        likes: vec![],
        unix : utils::get_unix_epoch_time(),
        hashtags,
    };

    /*- Insert the tweet -*/
    match collection.insert_one(tweet, None) {
        /*- Respond -*/
        Ok(_) => respond(&mut stream, 200u16, None, None),

        /*- Throw the docker-mongo bridge err -*/
        Err(e) => respond(&mut stream, 500u16, Some((
            ResponseType::Text,
            &get_error_code(103)
        )), None)
    };
}

/*- Like a tweet -*/
pub(crate) fn like(
    mut stream : TcpStream,
        request: String,
        params : HashMap<String, String>
) -> () {

    /*- Require some headers to be specified -*/
    let required = utils::get_required_headers("like");
    println!("{:?}", required);
    println!("{:?}", request);
    let headers  = parse_headers(request.clone(), HeaderReturn::All);
    println!("{:?}", headers);
    if !expect_headers(&mut stream, &headers, required) { return; };

    /*- Check the auth availability -*/
    let authentication_status:AuthorizationStatus = authenticate(headers.clone());
    let user_claims:UserClaims = match authentication_status {
        AuthorizationStatus::Authorized(v) => v,
        AuthorizationStatus::Unauthorized => 
            return respond(&mut stream, 401u16, Some((ResponseType::Text, DICTIONARY.error.unauthorized)), None),
        AuthorizationStatus::Err =>
            return respond(&mut stream, 401u16, Some((ResponseType::Text, DICTIONARY.error.unauthorized)), None)
    };

    /*- Get the "tweet_id" header -*/
    let tweet_id:String;

    /*- Get the headers -*/
    if let HeaderReturn::Values(headers) = headers {
        /*- Get the values -*/
        tweet_id = headers.get("tweet").unwrap().to_string();
    }else {
        /*- Return an error -*/
        return respond(&mut stream, 400u16, None, None);
    }

    /*- Establish the mongodb connection -*/
    let collection:Collection<Tweet> = utils::establish_mclient::<Tweet>("tweets");

    /*- Get the tweet -*/
    let tweet:Tweet = match collection.find_one(Some(doc!{"id": tweet_id.clone()}), None) {
        Ok(tweet) => tweet.unwrap_or_default(),
        _ => return respond(&mut stream, 404u16, None, None),
    };

    /*- Check if the user has already liked the tweet -*/
    if tweet.likes.contains(&user_claims.suid) {
        /*- Remove the user from the likes -*/
        let update_result = collection.update_one(
            doc!{ "id": tweet_id.clone() },
            doc!{ "$pull": doc!{"likes": user_claims.suid} },
            None
        );

        /*- Respond -*/
        respond(
            &mut stream,
            200u16,
            None,
            None
        );
    }else {
        /*- Add the user to the likes -*/
        let mut likes = tweet.likes.clone();
        likes.push(user_claims.suid);

        /*- Update the tweet -*/
        let update_result = collection.update_one(
            doc!{ "id": tweet_id },
            doc!{ "$set": {"likes": likes} },
            None
        );
        /*- Respond -*/
        respond(
            &mut stream,
            200u16,
            None,
            None
        );
    };
}

/*- Get all tweets containing hashtag -*/
pub(crate) fn hashtag(
    mut stream : TcpStream,
        request: String,
        params : HashMap<String, String>
) -> () {
    
    /*- Require some headers to be specified -*/
    let required = utils::get_required_headers("hashtag");
    let headers  = parse_headers(request.clone(), HeaderReturn::All);
    if !expect_headers(&mut stream, &headers, required) { return; };

    /*- Check the auth availability -*/
    let authentication_status:AuthorizationStatus = authenticate(headers.clone());

    /*- Get the "hashtag" header -*/
    let hashtag:String;

    /*- Get the headers -*/
    if let HeaderReturn::Values(headers) = headers {
        /*- Get the values -*/
        hashtag = headers.get("hashtag").unwrap().to_string();
    }else {
        /*- Return an error -*/
        return respond(&mut stream, 400u16, None, None);
    };

    /*- Establish the mongodb connection -*/
    let collection:Collection<Tweet> = utils::establish_mclient::<Tweet>("tweets");

    /*- Get the tweets -*/
    let tweets:Vec<Tweet> = match collection.find(Some(doc!{"hashtags": hashtag.clone()}), None) {
        Ok(tweets) => tweets,
        _ => return respond(&mut stream, 404u16, None, None),
    }.into_iter()
        .map(|e| e.unwrap_or_default())
        .collect::<Vec<_>>();

    /*- Respond -*/
    respond(
        &mut stream,
        200u16,
        Some((ResponseType::Json, &serde_json::to_string(&tweets).unwrap_or("{}".to_string()))),
        None
    );
}