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
#[path = "resources/dict.rs"] mod dict;
use fastserve::{ *, RouteRoot as RR, RouteValue as RV };
use std::ops;

/*- Statics & Constants -*/

/*- Structs, enums, unions -*/

/*- Startup -*/
fn main() -> () {
    /*- The api routes -*/
    let routes:Vec<RR> = vec![
        RR::Stack("/", vec![
            RR::Endpoint("login",               RV::Function(api::login         )),
            RR::Endpoint("create-account",      RV::Function(api::create_account)),
            RR::Endpoint("authenticate",        RV::Function(api::auth_test     )),
            RR::Endpoint("profile_data/:suid",  RV::Function(api::profile_data  )) 
        ]),
    ];

    /*- Start the server -*/
    fastserve::start(fastserve::ServerOptions {
        routes,
        port: 8000,
        numthreads: 6,
        log_status: true,
        url: "127.0.0.1",
        on_connect: None,
        statics: fastserve::Statics {
            custom404: None,
            dir: "./static",
            serve: false,
        }
    });
}