use std::io::{BufRead, BufReader, Read};

use crate::network::Stream;

#[derive(Default, Debug)]
pub struct Request {
    pub url: String,
    pub method: String,
    pub body: String,
    pub content_type: String,
}

pub struct Response {
    pub status: u16,
    pub headers: String,
    pub body: String,
}

impl Response {
    pub fn get_header(&self, k: &str) -> Result<&str, ()> {
        for line in self.headers.split("\r\n") {
            if let Some((key, value)) = line.split_once(": ") {
                if key == k {
                    return Ok(value);
                }
            }
        }

        Err(())
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
                usize::from_str_radix(&size_line.trim(), 16).map_err(|e| e.to_string())?;

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

    let mut head_lines = head.lines();
    let status_line = head_lines.next().ok_or("Empty Response")?;
    let status = status_line
        .split_whitespace()
        .nth(1)
        .ok_or("Invalid status line")?
        .parse::<u16>()
        .map_err(|_| "Invlid status code")?;

    let mut headers = String::new();
    for line in head_lines {
        headers.push_str(line);
        headers.push_str("\r\n");
    }

    Ok(Response {
        status,
        headers,
        body: String::from(body),
    })
}
