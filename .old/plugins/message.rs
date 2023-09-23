use reywen::client::Do;
use serde::{Deserialize, Serialize};

use crate::common::{crash_condition, md_fmt, RE};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MessageConf {
    pub enabled: bool,
}

// main message engine
pub async fn message_main(client: &Do) {
    client
        .member(
            Some("01F80118K1F2EYD9XAMCPQ0BCT"),
            Some("01H0HGAESARSD53KB7WTX7ND9A"),
        )
        .await
        .ban(Some("CP"))
        .await;
}
