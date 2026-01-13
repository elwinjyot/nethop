use std::error::Error;

use crate::{
    http::Request,
    network::Connection,
    test_bed::{get_operator, TestCase},
};

pub fn clean_script(script: &str) -> Result<String, Box<dyn Error>> {
    let mut cleaned_script = String::new();
    for line in script.lines() {
        if line.trim().starts_with("#") || line.is_empty() {
            continue;
        }

        cleaned_script.push_str(line);
        cleaned_script.push('\n');
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
    let mut current_test_case = TestCase::default();
    let mut current_request = Request::default();

    let mut is_query = false;
    let mut is_body = false;
    let mut is_test_case = false;

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
                test_cases: Vec::new(),
            };
            is_query = true;
            continue;
        } else if trimmed == "</query>" {
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
        } else if is_test_case {
            if trimmed == "</assert>" {
                is_test_case = false;
            } else {
                let expression: Vec<&str> = trimmed.splitn(3, " ").collect();
                if expression.len() != 3 {
                    return Err(format!(
                        "Invalid expression in assertion, {}, {}",
                        trimmed,
                        expression.len()
                    ));
                }

                current_test_case.key = String::from(expression[0]);
                current_test_case.operation = get_operator(expression[1])?;
                current_test_case.value = String::from(expression[2]);
                current_request
                    .test_cases
                    .push(std::mem::take(&mut current_test_case));
                current_test_case = TestCase::default();
            }
            continue;
        } else if is_query {
            if trimmed == "<body>" {
                is_body = true;
                continue;
            }

            if trimmed == "<assert>" {
                is_test_case = true;
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                match key.trim() {
                    "url" => current_request.url = value.trim().to_string(),
                    "method" => current_request.method = value.trim().to_uppercase(),
                    "content-type" => current_request.content_type = value.trim().to_lowercase(),
                    _ => return Err(format!("Unknown key: {}", key)),
                }
            }
        }
    }

    Ok(requests)
}
