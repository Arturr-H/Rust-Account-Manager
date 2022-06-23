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

/*- Startup -*/
fn main() -> () {
    /*- The api routes -*/
    let routes:Vec<RR> = vec![
        RR::Stack("/", vec![
            RR::Endpoint("feed",                            RV::Function((Method::Get, api::feed          ))),
            RR::Endpoint("like",                            RV::Function((Method::Get, api::like          ))),
            RR::Endpoint("tweet",                           RV::Function((Method::Get, api::tweet         ))),
            RR::Endpoint("login",                           RV::Function((Method::Get, api::login         ))),
            RR::Endpoint("hashtag",                         RV::Function((Method::Get, api::hashtag       ))),
            RR::Endpoint("create-account",                  RV::Function((Method::Get, api::create_account))),
            RR::Endpoint("profile_data/:suid",              RV::Function((Method::Get, api::profile_data  ))),
            RR::Endpoint("profile_image/:profile_image",    RV::Function((Method::Get, api::profile_image ))) 
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