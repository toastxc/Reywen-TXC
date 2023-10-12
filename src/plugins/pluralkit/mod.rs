pub mod data;
pub mod process;
pub use process::on_message;
pub mod db;

pub fn help() -> String {
    format!("{HELP_PROFILES}\n{HELP_ALIASES}")
}

pub const HELP_PROFILES: &str = "
### Profile
Profiles contain masquerade data as well as a unique ID
#### Make
##### Syntax
```text
?p profile create NAME
```
##### Example
```text
?p profile create fox
```
#### Use
##### Syntax
```text
?p profile send NAME MESSAGE
```
##### Example
```text
?p profile send fox hello! ^w^
```
#### Update
##### Syntax
```text
?p PROFILE skip/none/NAME skip/none/AVATAR skip/none/color
```
##### Example - change name
```text
?p profile edit fox other_name
```
##### Example - change avatar
```text
?p profile edit fox skip https://rube.kipiouq.com
```
##### Example - remove avatar
```text
?p profile edit fox skip none
```";

pub const HELP_ALIASES: &str = "
### Aliases
Aliases are shortened commands used to call predefined profiles
#### Make
##### Syntax
```text
?p alias create NAME/ID ALIAS
```
##### Example
```text
?p alias create fox f
```
#### Use
##### Syntax
```text
?p ALIAS MESSAGE
```
##### Example
```text
?p f hello! ^w^
```
#### Remove
##### Syntax
```text
?p alias remove ALIAS
```
##### Example
```text
?p alias remove f
```";

pub const NOT_FOUND: &str = "**Requested data could not be found**";
pub const INVALID: &str = "**Invalid imported data**";
pub const TOO_LARGE: &str = "**Message payload too large";
pub const SUCCESS_EMPTY: &str = "**Successfully completed operation**";
pub const DB_ERROR: &str = "**Database error, please contact an administrator**";
pub const NOT_YET_IMPL: &str = "**This feature has not yet been created :/**";

pub const MULTIPLE_NAME: &str = "**Cannot send, multiple entries for given name**";







