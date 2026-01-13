use std::io::{BufRead, BufReader, Read};

use serde_json::Value;

use crate::{network::Stream, test_bed::TestCase};

#[derive(Default, Debug)]
pub struct Request {
    pub url: String,
    pub method: String,
    pub body: String,
    pub content_type: String,
    pub test_cases: Vec<TestCase>,
}

pub struct Response {
    pub status: u16,
    pub headers: String,
    pub body: String,
}

impl Response {
    pub fn get_header(&self, k: &str) -> Option<&str> {
        for line in self.headers.split("\r\n") {
            if let Some((key, value)) = line.split_once(": ")
                && key.to_lowercase() == k.to_lowercase()
            {
                return Some(value);
            }
        }

        None
    }
}

pub fn read_body(stream: &mut Stream) -> Result<String, String> {
    let mut reader = BufReader::new(stream);
    let mut content_length = 0;
    let mut headers = String::new();
    let mut is_chunked = false;

    loop {
        let mut line = String::new();
        reader
            .read_line(&mut line)
            .map_err(|_| "Failed to read stream")?;

        if line == "\r\n" || line.is_empty() {
            break;
        }

        let cleaned_line = line.to_lowercase();
        if cleaned_line.starts_with("content-length:") {
            content_length = line
                .split_once(':')
                .map(|(_, val)| val.trim().parse::<usize>().unwrap_or(0))
                .unwrap_or(0);
        } else if cleaned_line.starts_with("transfer-encoding: chunked") {
            is_chunked = true;
        }

        headers.push_str(&line);
    }

    let mut body = Vec::new();

    if is_chunked {
        loop {
            let mut size_line = String::new();
            reader
                .read_line(&mut size_line)
                .map_err(|e| e.to_string())?;

            let chunk_size =
                usize::from_str_radix(size_line.trim(), 16).map_err(|e| e.to_string())?;

            if chunk_size == 0 {
                break;
            }

            let mut chunk_data = vec![0u8; chunk_size];
            reader
                .read_exact(&mut chunk_data)
                .map_err(|e| e.to_string())?;

            let mut crlf = String::new();
            reader.read_line(&mut crlf).map_err(|e| e.to_string())?;
            body.extend(chunk_data);
        }
    } else {
        body = vec![0u8; content_length];
        reader
            .read_exact(&mut body)
            .map_err(|_| "Failed to read stream")?;
    }

    Ok(format!(
        "{}\r\n\r\n{}",
        headers,
        String::from_utf8_lossy(&body)
    ))
}

pub fn parse_response(raw: &str) -> Result<Response, String> {
    let (head, body) = raw.split_once("\r\n\r\n").ok_or("Malformed response!")?;
    let mut response = Response {
        status: 404,
        headers: String::new(),
        body: String::new(),
    };

    let mut head_lines = head.lines();
    let status_line = head_lines.next().ok_or("Empty Response")?;
    let status = status_line
        .split_whitespace()
        .nth(1)
        .ok_or("Invalid status line")?
        .parse::<u16>()
        .map_err(|_| "Invlid status code")?;
    response.status = status;

    let mut headers = String::new();
    for line in head_lines {
        headers.push_str(line);
        headers.push_str("\r\n");
    }
    response.headers = headers;

    let content_type = response
        .get_header("content-type")
        .ok_or("Content type not sent by response".to_string())?;

    let mime_type = content_type.split(";").next().unwrap_or("").trim();
    response.body = match mime_type {
        "application/text" | "text/plain" => body.to_string(),
        "application/json" => {
            let json: Value =
                serde_json::from_str(body).map_err(|e| format!("Failed to parse json: {}", e))?;
            serde_json::to_string_pretty(&json)
                .map_err(|e| format!("Failed to parse json: {}", e))?
        }
        _ => return Err(format!("Unsupported content type found `{}`", mime_type)),
    };

    Ok(response)
}
