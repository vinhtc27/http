use std::{
    error::Error,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
    str::FromStr,
};

#[derive(Debug)]
enum Method {
    GET,
    HEAD,
    POST,
    PUT,
    DELETE,
    CONNECT,
    OPTIONS,
    TRACE,
    PATCH,
}

impl FromStr for Method {
    type Err = MethodParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GET" => Ok(Method::GET),
            "HEAD" => Ok(Method::HEAD),
            "POST" => Ok(Method::POST),
            "PUT" => Ok(Method::PUT),
            "DELETE" => Ok(Method::DELETE),
            "CONNECT" => Ok(Method::CONNECT),
            "OPTIONS" => Ok(Method::OPTIONS),
            "TRACE" => Ok(Method::TRACE),
            "PATCH" => Ok(Method::PATCH),
            _ => Err(MethodParseError),
        }
    }
}

#[derive(Debug)]
struct MethodParseError;

impl std::fmt::Display for MethodParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid HTTP method")
    }
}

#[derive(Debug)]
struct HttpRequest {
    method: Method,
    path: String,
    version: String,
    host: String,
    user_agent: String,
    accept: String,
}

impl From<&TcpStream> for HttpRequest {
    fn from(stream: &TcpStream) -> Self {
        let mut lines = BufReader::new(stream).lines();

        // Read the first line (request line)
        let request_line = lines.next().unwrap().unwrap();
        let parts: Vec<_> = request_line.split_whitespace().collect();
        let method = parts[0].parse().unwrap();
        let path = parts[1].to_string();
        let version = parts[2].to_string();

        // Read the subsequent header lines
        let mut host = String::new();
        let mut user_agent = String::new();
        let mut accept = String::new();

        for line in lines {
            let line = line.unwrap();
            if line.starts_with("Host:") {
                host = line[6..].trim().to_string();
            } else if line.starts_with("User-Agent:") {
                user_agent = line[12..].trim().to_string();
            } else if line.starts_with("Accept:") {
                accept = line[8..].trim().to_string();
            }
            // Check for an empty line, which indicates the end of headers
            if line.is_empty() {
                break;
            }
        }

        Self {
            method,
            path,
            version,
            host,
            user_agent,
            accept,
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let request = HttpRequest::from(&stream);

                if request.path == "/" {
                    stream.write(b"HTTP/1.1 200 OK\r\n\r\n")?;
                } else {
                    stream.write_all(b"HTTP/1.1 404 Not Found\r\n\r\n")?;
                }
            }
            Err(e) => {
                eprintln!("error: {}", e);
            }
        }
    }

    Ok(())
}
