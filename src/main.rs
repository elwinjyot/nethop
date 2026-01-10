use std::error::Error;
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;

struct Response {
    status: u16,
    headers: String,
    body: String,
}

impl Response {
    fn get_header(&self, k: &str) -> Result<&str, ()> {
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

struct Connection {
    host: String,
    port: u16,
    reader: Option<BufReader<TcpStream>>,
}

struct Request {
    url: String,
    method: String,
    body: String,
    content_type: String,
}

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
            println!("> Body: {}", response.body);
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

fn connect(connection: &mut Connection) -> Result<(), String> {
    println!("Connecting to {}", connection.host);
    let stream = TcpStream::connect(format!("{}:{}", connection.host, connection.port))
        .map_err(|e| format!("Failed to connect to TCP stream: {}", e))?;

    connection.reader = Some(BufReader::new(stream));

    Ok(())
}

fn send_request(connection: &mut Connection, request: &Request) -> Result<Response, String> {
    let body_bytes = request.body.as_bytes();
    let content_len = body_bytes.len();

    let request_str = format!(
        "{method} {url} HTTP/1.1\r\n\
        Host: {host}\r\n\
        User-Agent: NetHop/0.0\r\n\
        Content-Type: {type}\r\n\
        Content-Length: {len}\r\n\
        Connection: keep-alive\r\n\
        \r\n\
        ",
        method = request.method,
        url = request.url,
        host = connection.host,
        len = content_len,
        type = request.content_type
    );

    let reader = connection.reader.as_mut().ok_or("Not Connected")?;
    let mut stream = reader.get_mut();

    // Write headers
    stream
        .write_all(request_str.as_bytes())
        .map_err(|err| format!("Failed to send request: {}", err))?;

    if request.method == "POST" || request.method == "PUT" {
        if request.body.is_empty() {
            return Err(format!("Empty body sent to {} request", request.method));
        };

        stream
            .write_all(body_bytes)
            .map_err(|err| format!("Failed to write body: {}", err))?;
    }

    stream.flush().map_err(|err| err.to_string())?;

    let response = read_body(&mut stream).unwrap_or(String::from(""));
    parse_response(&response)
}

fn read_body(stream: &mut TcpStream) -> Result<String, String> {
    let mut reader = BufReader::new(stream);
    let mut content_length = 0;
    let mut headers = String::new();

    loop {
        let mut line = String::new();
        reader
            .read_line(&mut line)
            .map_err(|_| "Failed to read stream")?;

        if line == "\r\n" || line.is_empty() {
            break;
        }

        if line.to_lowercase().starts_with("content-length:") {
            content_length = line
                .split_once(':')
                .map(|(_, val)| val.trim().parse::<usize>().unwrap_or(0))
                .unwrap_or(0);
        }

        headers.push_str(&line);
    }

    let mut body_bytes = vec![0u8; content_length];
    reader
        .read_exact(&mut body_bytes)
        .map_err(|_| "Failed to read stream")?;
    let body = String::from_utf8_lossy(&body_bytes);

    Ok(format!("{}\r\n\r\n{}", headers, body))
}

fn parse_response(raw: &str) -> Result<Response, String> {
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
