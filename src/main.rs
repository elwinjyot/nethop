mod http;
mod network;
mod ui;

use std::error::Error;
use std::fs;

use serde_json::Value;

use crate::{
    http::Request,
    network::{connect, send_request, Connection},
    ui::view_in_less,
};

fn main() -> Result<(), Box<dyn Error>> {
    let query_raw = fs::read_to_string("test.hop")?;
    let mut lines = query_raw.lines();
    let mut conn = Connection {
        host: String::new(),
        port: 80,
        reader: None,
    };

    for line in lines.by_ref().take(2) {
        let cleaned_line: String = line.chars().filter(|c| !c.is_whitespace()).collect();
        let (key, value) = cleaned_line.split_once("=").ok_or("Syntax error")?;

        match key {
            "host" => conn.host = String::from(value),
            "port" => conn.port = value.parse().map_err(|_| "Invalid port value")?,
            _ => return Err("Invalid key found".into()),
        }
    }

    connect(&mut conn)?;

    // TODO: Organise the file handling
    // to a separate module
    let mut is_query = false;
    let mut is_body = false;
    let mut request = Request {
        url: String::new(),
        method: String::new(),
        body: String::new(),
        content_type: String::new(),
    };

    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed == "<query>" {
            is_query = true;
            continue;
        } else if trimmed == "</query>" {
            println!("\n> [{}]: {}{}", request.method, conn.host, request.url);
            let response = send_request(&mut conn, &request)?;
            println!("====[RESPONSE]====");
            println!("> Status: {}", response.status);
            println!("> Date: {}", response.get_header("Date").unwrap_or("--"));

            let json_body: Value = serde_json::from_str(response.body.as_str())?;
            view_in_less(
                format!(
                    "> [{}]: {}{}\n\n{}",
                    request.method,
                    conn.host,
                    request.url,
                    &serde_json::to_string_pretty(&json_body)?
                )
                .as_str(),
            )?;
            is_query = false;
            continue;
        }

        if is_body {
            if trimmed == "</body>" {
                is_body = false;
            } else {
                request.body.push_str(line);
                request.body.push('\n');
            }
            continue;
        }

        if is_query {
            if trimmed == "<body>" {
                is_body = true;
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                match key.trim() {
                    "url" => request.url = value.trim().to_string(),
                    "method" => request.method = value.trim().to_uppercase(),
                    "content-type" => request.content_type = value.trim().to_lowercase(),
                    _ => return Err(format!("Unknown key: {}", key).into()),
                }
            }
        }
    }

    Ok(())
}
