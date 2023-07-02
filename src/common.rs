use reywen::structures::channels::message::Message;

// Simple markdown formating
pub fn md_fmt(message: &str, emoji: RE) -> String {
    let emoji = RE::e(emoji);

    format!("{emoji} $\\color{{grey}}\\small\\textsf{{{message}}}$")
}

// Custom emojis
pub enum RE {
    Search,
    Send,
    Rm,
    Json,
    Insert,
}

// Serilizing emojis
impl RE {
    pub fn e(self) -> String {
        match self {
            RE::Search => ":01GQE862YPERANAJC30GNKH625:",
            RE::Send => ":01GQE848SKP794SKZYY8RTCXF1:",
            RE::Rm => ":01GQE86CT9MKAHPTG55HMTG7TR:",
            RE::Json => ":01GQE86K0CG3FWA0D6FRY7JT0R:",
            RE::Insert => ":01GQE86SAYFDXZE2F39YHJMB1F:",
        }
        .to_string()
    }
}
pub fn lte(input: &str) -> String {
    format!("[]({input})")
}
// basic CLI tool for checking content
pub fn crash_condition(input_message: &Message, character: Option<&str>) -> bool {
    if input_message.content.is_none() {
        return true;
    };

    let temp_convec: Vec<&str> = input_message
        .content
        .as_ref()
        .unwrap()
        .split(' ')
        .collect::<Vec<&str>>();

    let mut length = 2;

    if character.is_none() {
        length = 1;
    };

    if temp_convec.len() < length {
        return true;
    };

    if character.is_some() && character != Some(temp_convec[0]) {
        return true;
    };
    false
}
