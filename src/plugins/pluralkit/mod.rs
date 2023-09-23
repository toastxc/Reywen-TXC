pub mod data;
pub mod process;
pub use process::on_message;
pub mod db;
pub const HELP: &str = "## TXC Masquerade Tool

### Profile
`?p profile X`

#### Delete
`?p profile delete ID/NAME`

#### Edit
`?p profile edit ID/NAME`

#### Create
`?p profile create NAME`

#### Send
`?p profile send ID/NAME message`";
pub const NOT_FOUND: &str = "**Requested data could not be found**";
pub const INVALID: &str = "**Invalid imported data**";
pub const TOO_LARGE: &str = "**Message payload too large";
pub const SUCCESS_EMPTY: &str = "**Successfully completed operation**";
pub const DB_ERROR: &str = "**Database error, please contact an administrator**";
pub const NOT_YET_IMPL: &str = "**This feature has not yet been created :/**";

pub const MULTIPLE_NAME: &str = "**Cannot send, multiple entries for given name**";
