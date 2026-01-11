use std::error::Error;

use crate::{http::Request, network::Connection};

pub fn clean_script(script: &str) -> Result<String, Box<dyn Error>> {
    let mut cleaned_script = String::new();
    for line in script.lines() {
        if line.trim().starts_with("#") || line.is_empty() {
            continue;
        }

        cleaned_script.push_str(line);
        cleaned_script.push_str("\n");
    }

    Ok(cleaned_script)
}

pub fn fetch_connection_header(script: &str) -> Result<Connection, String> {
    if script.matches("<connect>").count() > 1 {
        return Err("Multiple connect headers found!".to_string());
    }
    let mut lines = script.lines();

    if lines.next() != Some("<connect>") {
        return find_connection_header(script);
    }

    let mut conn = Connection {
        host: String::new(),
        port: 443,
        is_safe: true,
        reader: None,
    };

    for line in lines.take_while(|l| l.trim() != "</connect>") {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        if let Some((key, value)) = line.split_once("=") {
            match key.trim() {
                "host" => conn.host = value.trim().to_string(),
                "port" => conn.port = value.trim().parse().map_err(|_| "Invalid PORT passed")?,
                _ => return Err(format!("Invalid parameter: {}", key)),
            }
        } else {
            match line {
                "unsafe" => {
                    conn.port = 80;
                    conn.is_safe = false;
                }
                _ => return Err(format!("Invalid option: {}", line)),
            }
        }
    }

    if conn.host.is_empty() {
        return Err("Connection host not specified".into());
    }

    Ok(conn)
}

pub fn find_connection_header(_script: &str) -> Result<Connection, String> {
    todo!("Search for connection headers from the whole file")
}

pub fn fetch_requests(script: &str) -> Result<Vec<Request>, String> {
    let lines = script.lines();
    let estimated_size = script.matches("<query>").count();
    let mut requests: Vec<Request> = Vec::with_capacity(estimated_size);
    let mut current_request = Request {
        url: String::new(),
        method: String::from("GET"),
        body: String::new(),
        content_type: String::from("application/text"),
    };

    let mut is_query = false;
    let mut is_body = false;

    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if trimmed == "<query>" {
            current_request = Request {
                url: String::new(),
                method: String::from("GET"),
                body: String::new(),
                content_type: String::new(),
            };
            is_query = true;
            continue;
        } else if trimmed == "</query>" {
            // println!("\n> [{}]: {}{}", request.method, conn.host, request.url);
            // let response = send_request(&mut conn, &request)?;
            // println!("====[RESPONSE]====");
            // println!("> Status: {}", response.status);
            // println!("> Date: {}", response.get_header("Date").unwrap_or("--"));
            //
            // let json_body: Value = serde_json::from_str(response.body.as_str())?;
            // view_in_less(
            //     format!(
            //         "> [{}]: {}{}\n\n{}",
            //         request.method,
            //         conn.host,
            //         request.url,
            //         &serde_json::to_string_pretty(&json_body)?
            //     )
            //     .as_str(),
            // )?;
            requests.push(std::mem::take(&mut current_request));
            is_query = false;
            continue;
        }

        if is_body {
            if trimmed == "</body>" {
                is_body = false;
            } else {
                current_request.body.push_str(line);
                current_request.body.push('\n');
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
                    "url" => current_request.url = value.trim().to_string(),
                    "method" => current_request.method = value.trim().to_uppercase(),
                    "content-type" => current_request.content_type = value.trim().to_lowercase(),
                    _ => return Err(format!("Unknown key: {}", key).into()),
                }
            }
        }
    }

    Ok(requests)
}
