/*- Imports -*/
use std::fmt;
use serde::{ Serialize, Deserialize };
use crate::user::User;

/// # SafeUser
/// A struct representing a SafeUser.
/// A SafeUser is a User struct that doesn't contain sensitive information.
/// Like the password, the uid, and the email.
/// SafeUser is used in a variety of places, like when displaying a user's profile to clients.
/// There are functions to convert any user into a SafeUser, like convert_user().
#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct SafeUser {
    pub username    : String,
    pub displayname : String,
    pub suid        : String,
    pub age         : u8,
}

/*- For printing / debugging -*/
impl fmt::Debug for SafeUser {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "User {{ username: {}, displayname: {}, age: {}, suid: {} }}",
            self.username, self.displayname, self.age, self.suid
        )
    }
}

/*- Convert user to SafeUser -*/
pub(crate) fn convert_user(user: User) -> SafeUser {
    SafeUser {
        username    : user.username,
        displayname : user.displayname,
        suid        : user.suid,
        age         : user.age,
    }
}