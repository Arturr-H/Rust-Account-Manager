/*- Imports -*/
use lazy_static::lazy_static;

/*- Constants -*/
pub(crate) struct Variables {
    password_min_len:u8,
    password_max_len:u8,
}

lazy_static! {
    static ref VARIABLES:Variables = Variables {
        password_min_len: 8u8,
        password_max_len: 100u8,
    };
}


/*- A dictionary of phrases that are ex
    responded with inside of this project -*/
pub struct Dictionary<'lf> {
    pub error:Error<'lf>
}

/*- (ERR) Error messages -*/
pub struct Error<'lf> {
    pub in_use: InUse<'lf>,
    pub password: Password<'lf>,
    pub invalid: Invalid<'lf>,
}

/*- (ERR) When something with the password has gone wrong -*/
pub struct Password<'lf> {
    pub len_min: &'lf str,
    pub len_max: &'lf str,
}

/*- (ERR) When some parameters are already in use -*/
pub struct InUse<'lf> {
    pub email:&'lf str,
    pub username:&'lf str
}

/*- (ERR) When some parameters are invalid -*/
pub struct Invalid<'lf> {
    pub email:&'lf str,
    pub username:&'lf str
}

/*- Create the dictionary -*/
pub(crate) const DICTIONARY:Dictionary = Dictionary {
    error: Error { 
        in_use: InUse {
            email: "Email is already in use",
            username: "Username is already in use"
        },
        password: Password {
            len_min: "Password must be atleast {} characters long",
            len_max: "Password must be less than {} characters long",
        },
        invalid: Invalid {
            email: "Email is invalid",
            username: "Username is invalid"
        }
    }
};