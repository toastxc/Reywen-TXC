use reywen::client::Do;
use serde::{Deserialize, Serialize};
use std::process::Command;

use crate::{crash_condition, md_fmt, RE};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ShellConf {
    pub enabled: bool,
    pub whitelist_sudoers: bool,
    pub channel_only: bool,
    pub channel: String,
}

pub async fn shell_main(client: &Do) {
    let help = format!("{} {}", md_fmt("?/", RE::Send), "executes a bash command");

    // import config
    let shell: ShellConf = serde_json::from_str(
        &(String::from_utf8(
            std::fs::read("config/shell.json").expect("failed to read config/shell.json\n{e}"),
        )
        .unwrap()),
    )
    .expect("Failed to deser plural.json");

    if !shell.enabled {
        return;
    };

    if crash_condition(&client.input_message, Some("?/")) {
        return;
    };

    if client.input().convec()[1] == "help" {
        client.message().sender(&help).await;
        return;
    };

    // due to how dangerous shell commands are, there needs to be security checks

    if shell.channel_only && client.input().channel_is(&shell.channel) {
        return;
    };
    client.auth().is_sudoer(&client.input().author());

    if shell.whitelist_sudoers && !client.auth().is_sudoer(&client.input().author()) {
        client.message().sender("**Only sudoers allowed**").await;
        return;
    };

    let convec = client.input().convec();

    let mut content_min1 = String::new();

    for x in 0..convec.len() - 1 {
        content_min1 += &format!("{} ", convec[x + 1])
    }

    bash_exec(client, convec).await;
}

pub async fn bash_exec(client: &Do, mut convec: Vec<String>) {
    // shell

    let mut com = Command::new("bash");
    com.arg("-c");

    convec.remove(0);

    let mut newstr = String::new();

    for x in convec.iter() {
        newstr += &(String::from(" ") + x)
    }

    println!("{newstr}");

    com.arg(newstr);

    if let Err(e) = com.output() {
        client.message().sender(&e.to_string()).await;
        return;
    };

    let out = com.output().unwrap();

    let out = String::from_utf8_lossy(&out.stdout) + String::from_utf8_lossy(&out.stderr);

    if out.chars().count() <= 1000 {
        client.message().sender(&format!("```text\n{out}")).await;
    } else {
        bash_big_msg(out.to_string(), client).await;
    };
}

// this code is terrible im sorry
pub async fn bash_big_msg(out: String, client: &Do) {
    let vec: Vec<char> = out.chars().collect();

    let (a, b, c) = convert(vec.len() as i32);

    // iterator
    // payload
    // remainder

    let mut current = String::new();
    let mut iter = 0;

    for _ in 0..a {
        for _ in 0..b {
            current += &vec[(iter) as usize].to_string();
            iter += 1;
        }

        client
            .clone()
            .message()
            .sender(&format!("```\\n\\n{current}"))
            .await;

        current = String::new();
    }

    if c > 0 {
        for _ in 0..c {
            current += &vec[(iter - 1) as usize].to_string();
            iter += 1;
        }

        current = format!("```\\n\\n{current}");

        client.message().sender(&current).await;
    };
    println!();
}

pub fn convert(a: i32) -> (i32, i32, i32) {
    if a < 1000 {
        return (1, a, 0);
    };

    (a / 1000, 1000, a % 1000)
}
