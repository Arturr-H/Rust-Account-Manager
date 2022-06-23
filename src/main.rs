/*- Global allowances -*/
#![allow(
    dead_code,
    unused_variables,
    unused_imports
)]

/*- Imports -*/
mod api;
mod utils;
mod user;
mod safe_user;
mod tweet;
#[path = "resources/dict.rs"] mod dict;
use fastserve::{ *, RouteRoot as RR, RouteValue as RV, Method };
use std::ops;

/*- Statics & Constants -*/

/*- Structs, enums, unions -*/
const POST:Method = Method::Post;
const GET:Method =  Method::Get;

/*- Startup -*/
fn main() -> () {
    /*- The api routes -*/
    let routes:Vec<RR> = vec![
        RR::Stack("/", vec![
            RR::Endpoint("feed",                            RV::Function((GET,  api::feed          ))),
            RR::Endpoint("like",                            RV::Function((GET,  api::like          ))),
            RR::Endpoint("login",                           RV::Function((GET,  api::login         ))),
            RR::Endpoint("tweet",                           RV::Function((POST, api::tweet         ))),
            RR::Endpoint("comment",                         RV::Function((POST, api::comment       ))),
            RR::Endpoint("hashtag",                         RV::Function((GET,  api::hashtag       ))),
            RR::Endpoint("get_comments",                    RV::Function((GET,  api::get_comments  ))),
            RR::Endpoint("create-account",                  RV::Function((GET,  api::create_account))),
            RR::Endpoint("profile_data/:suid",              RV::Function((GET,  api::profile_data  ))),
            RR::Endpoint("profile_image/:profile_image",    RV::Function((GET,  api::profile_image ))) 
        ]),
    ];

    /*- Start the server -*/
    fastserve::start(fastserve::ServerOptions {
        routes,
        port      : 8000,
        numthreads: 6,
        log_status: true,
        url       : "127.0.0.1",
        on_connect: None,
        statics   : fastserve::Statics {
            custom404: None,
            dir      : "./static",
            serve    : false,
        }
    });
}