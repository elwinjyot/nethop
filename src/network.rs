use std::{
    error::Error,
    io::{BufReader, Read, Write},
    net::TcpStream,
};

use native_tls::{TlsConnector, TlsStream};

use crate::{
    http::{parse_response, read_body, Request, Response},
    test_bed::test_case,
    ui::view_in_less,
};

pub enum Stream {
    Http(TcpStream),
    Https(TlsStream<TcpStream>),
}

impl Read for Stream {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        match self {
            Stream::Http(stream) => stream.read(buf),
            Stream::Https(stream) => stream.read(buf),
        }
    }
}

impl Write for Stream {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match self {
            Stream::Http(s) => s.write(buf),
            Stream::Https(s) => s.write(buf),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        match self {
            Stream::Http(s) => s.flush(),
            Stream::Https(s) => s.flush(),
        }
    }
}

pub struct Connection {
    pub host: String,
    pub port: u16,
    pub is_safe: bool,
    pub reader: Option<BufReader<Stream>>,
}

pub fn connect(connection: &mut Connection) -> Result<(), Box<dyn Error>> {
    println!(
        "Connecting to {} on port {}",
        connection.host, connection.port
    );
    let address = format!("{}:{}", connection.host, connection.port);
    let tcp_stream = TcpStream::connect(address)?;

    let stream = if connection.is_safe {
        let connector = TlsConnector::new()?;
        let tls_stream = connector
            .connect(&connection.host, tcp_stream)
            .map_err(|e| format!("TLS Handshake failed: {}", e))?;
        Stream::Https(tls_stream)
    } else {
        Stream::Http(tcp_stream)
    };

    connection.reader = Some(BufReader::new(stream));
    Ok(())
}

pub fn send_request(connection: &mut Connection, request: &Request) -> Result<Response, String> {
    let body_bytes = request.body.as_bytes();
    let content_len = body_bytes.len();

    let request_str = format!(
    "{method} {url} HTTP/1.1\r\n\
    Host: {host}\r\n\
    User-Agent: NetHop/0.0\r\n\
    Content-Type: {type}\r\n\
    Content-Length: {len}\r\n\
    Accept: application/json\r\n\
    Accept-Encoding: identity\r\n\
    Connection: keep-alive\r\n\
    \r\n",
        method = request.method,
        url = request.url,
        host = connection.host,
        len = content_len,
        type = request.content_type
    );

    let reader = connection.reader.as_mut().ok_or("Not Connected")?;
    let stream = reader.get_mut();

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

    let response = read_body(stream).unwrap_or(String::from(""));
    parse_response(&response)
}

pub fn execute_batch_requests(
    requests: Vec<Request>,
    conn: &mut Connection,
) -> Result<(), Box<dyn Error>> {
    for request in requests {
        execute_request(&request, conn)?;
    }

    Ok(())
}

fn execute_request(request: &Request, conn: &mut Connection) -> Result<(), Box<dyn Error>> {
    println!("\n[{}: {}]", request.method, request.url);
    let response = send_request(conn, request)?;

    if request.test_cases.len() > 0 {
        let mut passed = 0;
        let mut failed = 0;
        println!("Running {} test(s)\n", request.test_cases.len());
        for case in &request.test_cases {
            if test_case(&response, &case) {
                println!(" -> Passed");
                passed += 1;
            } else {
                println!(" -> Failed");
                failed += 1;
            }
        }

        println!("\nReport:");
        println!(
            "Ran {} test(s), {} test(s) passed, {} test(s) failed",
            passed + failed,
            passed,
            failed
        );
    } else {
        println!("> Status: {}", response.status);
        println!("> Date: {}", response.get_header("Date").unwrap_or("--"));
        view_in_less(&format!(
            "[{}: {}{}]\n\n{}",
            request.method, conn.host, request.url, response.body
        ))?;
    }

    Ok(())
}
