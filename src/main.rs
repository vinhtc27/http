#![forbid(unsafe_code)]
#![allow(dead_code)]

use std::{
    collections::HashMap,
    env, fmt,
    fs::{read, write},
    io::{BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    str::FromStr,
    sync::Arc,
    thread,
};

use flate2::{write::GzEncoder, Compression};

const CRLF: &str = "\r\n";

#[derive(Debug)]
enum Method {
    Get,
    Head,
    Post,
    Put,
    Delete,
    Connect,
    Options,
    Trace,
    Patch,
}

impl FromStr for Method {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "GET" => Ok(Method::Get),
            "HEAD" => Ok(Method::Head),
            "POST" => Ok(Method::Post),
            "PUT" => Ok(Method::Put),
            "DELETE" => Ok(Method::Delete),
            "CONNECT" => Ok(Method::Connect),
            "OPTIONS" => Ok(Method::Options),
            "TRACE" => Ok(Method::Trace),
            "PATCH" => Ok(Method::Patch),
            _ => Err(()),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum HeaderType {
    Accept,
    AcceptCharset,
    AcceptEncoding,
    AcceptLanguage,
    AccessControlRequestMethod,
    AccessControlRequestHeaders,
    Authorization,
    CacheControl,
    Connection,
    ContentDisposition,
    ContentEncoding,
    ContentLanguage,
    ContentLength,
    ContentType,
    Cookie,
    Date,
    Expect,
    Forwarded,
    From,
    Host,
    IfMatch,
    IfModifiedSince,
    IfNoneMatch,
    IfRange,
    IfUnmodifiedSince,
    MaxForwards,
    Origin,
    Pragma,
    ProxyAuthenticate,
    ProxyAuthorization,
    Range,
    Referer,
    TE,
    Trailer,
    TransferEncoding,
    UserAgent,
    Upgrade,
    Via,
    Warning,
    Custom(String),
}

impl fmt::Display for HeaderType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HeaderType::Accept => write!(f, "Accept"),
            HeaderType::AcceptCharset => write!(f, "Accept-Charset"),
            HeaderType::AcceptEncoding => write!(f, "Accept-Encoding"),
            HeaderType::AcceptLanguage => write!(f, "Accept-Language"),
            HeaderType::AccessControlRequestMethod => write!(f, "Access-Control-Request-Method"),
            HeaderType::AccessControlRequestHeaders => write!(f, "Access-Control-Request-Headers"),
            HeaderType::Authorization => write!(f, "Authorization"),
            HeaderType::CacheControl => write!(f, "Cache-Control"),
            HeaderType::Connection => write!(f, "Connection"),
            HeaderType::ContentDisposition => write!(f, "Content-Disposition"),
            HeaderType::ContentEncoding => write!(f, "Content-Encoding"),
            HeaderType::ContentLanguage => write!(f, "Content-Language"),
            HeaderType::ContentLength => write!(f, "Content-Length"),
            HeaderType::ContentType => write!(f, "Content-Type"),
            HeaderType::Cookie => write!(f, "Cookie"),
            HeaderType::Date => write!(f, "Date"),
            HeaderType::Expect => write!(f, "Expect"),
            HeaderType::Forwarded => write!(f, "Forwarded"),
            HeaderType::From => write!(f, "From"),
            HeaderType::Host => write!(f, "Host"),
            HeaderType::IfMatch => write!(f, "If-Match"),
            HeaderType::IfModifiedSince => write!(f, "If-Modified-Since"),
            HeaderType::IfNoneMatch => write!(f, "If-None-Match"),
            HeaderType::IfRange => write!(f, "If-Range"),
            HeaderType::IfUnmodifiedSince => write!(f, "If-Unmodified-Since"),
            HeaderType::MaxForwards => write!(f, "Max-Forwards"),
            HeaderType::Origin => write!(f, "Origin"),
            HeaderType::Pragma => write!(f, "Pragma"),
            HeaderType::ProxyAuthenticate => write!(f, "Proxy-Authenticate"),
            HeaderType::ProxyAuthorization => write!(f, "Proxy-Authorization"),
            HeaderType::Range => write!(f, "Range"),
            HeaderType::Referer => write!(f, "Referer"),
            HeaderType::TE => write!(f, "TE"),
            HeaderType::Trailer => write!(f, "Trailer"),
            HeaderType::TransferEncoding => write!(f, "Transfer-Encoding"),
            HeaderType::UserAgent => write!(f, "User-Agent"),
            HeaderType::Upgrade => write!(f, "Upgrade"),
            HeaderType::Via => write!(f, "Via"),
            HeaderType::Warning => write!(f, "Warning"),
            HeaderType::Custom(name) => write!(f, "{}", name),
        }
    }
}

impl FromStr for HeaderType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Accept" => Ok(HeaderType::Accept),
            "Accept-Charset" => Ok(HeaderType::AcceptCharset),
            "Accept-Encoding" => Ok(HeaderType::AcceptEncoding),
            "Accept-Language" => Ok(HeaderType::AcceptLanguage),
            "Access-Control-Request-Method" => Ok(HeaderType::AccessControlRequestMethod),
            "Access-Control-Request-Headers" => Ok(HeaderType::AccessControlRequestHeaders),
            "Authorization" => Ok(HeaderType::Authorization),
            "Cache-Control" => Ok(HeaderType::CacheControl),
            "Connection" => Ok(HeaderType::Connection),
            "Content-Disposition" => Ok(HeaderType::ContentDisposition),
            "Content-Encoding" => Ok(HeaderType::ContentEncoding),
            "Content-Language" => Ok(HeaderType::ContentLanguage),
            "Content-Length" => Ok(HeaderType::ContentLength),
            "Content-Type" => Ok(HeaderType::ContentType),
            "Cookie" => Ok(HeaderType::Cookie),
            "Date" => Ok(HeaderType::Date),
            "Expect" => Ok(HeaderType::Expect),
            "Forwarded" => Ok(HeaderType::Forwarded),
            "From" => Ok(HeaderType::From),
            "Host" => Ok(HeaderType::Host),
            "If-Match" => Ok(HeaderType::IfMatch),
            "If-Modified-Since" => Ok(HeaderType::IfModifiedSince),
            "If-None-Match" => Ok(HeaderType::IfNoneMatch),
            "If-Range" => Ok(HeaderType::IfRange),
            "If-Unmodified-Since" => Ok(HeaderType::IfUnmodifiedSince),
            "Max-Forwards" => Ok(HeaderType::MaxForwards),
            "Origin" => Ok(HeaderType::Origin),
            "Pragma" => Ok(HeaderType::Pragma),
            "Proxy-Authenticate" => Ok(HeaderType::ProxyAuthenticate),
            "Proxy-Authorization" => Ok(HeaderType::ProxyAuthorization),
            "Range" => Ok(HeaderType::Range),
            "Referer" => Ok(HeaderType::Referer),
            "TE" => Ok(HeaderType::TE),
            "Trailer" => Ok(HeaderType::Trailer),
            "Transfer-Encoding" => Ok(HeaderType::TransferEncoding),
            "User-Agent" => Ok(HeaderType::UserAgent),
            "Upgrade" => Ok(HeaderType::Upgrade),
            "Via" => Ok(HeaderType::Via),
            "Warning" => Ok(HeaderType::Warning),
            other => Ok(HeaderType::Custom(other.to_string())),
        }
    }
}

#[derive(Debug)]
enum EncodingType {
    Gzip,
    Compress,
    Deflate,
    Brotli,
    Zstd,
}

impl fmt::Display for EncodingType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            EncodingType::Gzip => write!(f, "gzip"),
            EncodingType::Compress => write!(f, "compress"),
            EncodingType::Deflate => write!(f, "deflate"),
            EncodingType::Brotli => write!(f, "br"),
            EncodingType::Zstd => write!(f, "zstd"),
        }
    }
}

impl FromStr for EncodingType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "gzip" => Ok(EncodingType::Gzip),
            "compress" => Ok(EncodingType::Compress),
            "deflate" => Ok(EncodingType::Deflate),
            "br" => Ok(EncodingType::Brotli),
            "zstd" => Ok(EncodingType::Zstd),
            _ => Err(()),
        }
    }
}

impl HeaderType {
    fn parse(line: &str) -> Option<(Self, String)> {
        let mut parts = line.splitn(2, ':');
        let key = parts.next()?.trim();
        let value = parts.next()?.trim().to_string();
        let header_type = Self::from_str(key).ok()?;

        Some((header_type, value))
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum StatusCode {
    Continue = 100,
    SwitchingProtocols = 101,
    Processing = 102,

    Ok = 200,
    Created = 201,
    Accepted = 202,
    NonAuthoritativeInfo = 203,
    NoContent = 204,
    ResetContent = 205,
    PartialContent = 206,
    MultiStatus = 207,
    AlreadyReported = 208,
    ImUsed = 226,

    MultipleChoices = 300,
    MovedPermanently = 301,
    Found = 302,
    SeeOther = 303,
    NotModified = 304,
    UseProxy = 305,
    TemporaryRedirect = 307,
    PermanentRedirect = 308,

    BadRequest = 400,
    Unauthorized = 401,
    PaymentRequired = 402,
    Forbidden = 403,
    NotFound = 404,
    MethodNotAllowed = 405,
    NotAcceptable = 406,
    ProxyAuthRequired = 407,
    RequestTimeout = 408,
    Conflict = 409,
    Gone = 410,
    LengthRequired = 411,
    PreconditionFailed = 412,
    PayloadTooLarge = 413,
    UriTooLong = 414,
    UnsupportedMediaType = 415,
    RangeNotSatisfiable = 416,
    ExpectationFailed = 417,
    ImATeapot = 418,
    MisdirectedRequest = 421,
    UnprocessableEntity = 422,
    Locked = 423,
    FailedDependency = 424,
    TooEarly = 425,
    UpgradeRequired = 426,
    PreconditionRequired = 428,
    TooManyRequests = 429,
    RequestHeaderFieldsTooLarge = 431,
    UnavailableForLegalReasons = 451,

    InternalServerError = 500,
    NotImplemented = 501,
    BadGateway = 502,
    ServiceUnavailable = 503,
    GatewayTimeout = 504,
    HttpVersionNotSupported = 505,
    VariantAlsoNegotiates = 506,
    InsufficientStorage = 507,
    LoopDetected = 508,
    NotExtended = 510,
    NetworkAuthenticationRequired = 511,
}

impl fmt::Display for StatusCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                StatusCode::Continue => "100 Continue",
                StatusCode::SwitchingProtocols => "101 Switching Protocols",
                StatusCode::Processing => "102 Processing",
                StatusCode::Ok => "200 OK",
                StatusCode::Created => "201 Created",
                StatusCode::Accepted => "202 Accepted",
                StatusCode::NonAuthoritativeInfo => "203 Non-Authoritative Information",
                StatusCode::NoContent => "204 No Content",
                StatusCode::ResetContent => "205 Reset Content",
                StatusCode::PartialContent => "206 Partial Content",
                StatusCode::MultiStatus => "207 Multi-Status",
                StatusCode::AlreadyReported => "208 Already Reported",
                StatusCode::ImUsed => "226 IM Used",
                StatusCode::MultipleChoices => "300 Multiple Choices",
                StatusCode::MovedPermanently => "301 Moved Permanently",
                StatusCode::Found => "302 Found",
                StatusCode::SeeOther => "303 See Other",
                StatusCode::NotModified => "304 Not Modified",
                StatusCode::UseProxy => "305 Use Proxy",
                StatusCode::TemporaryRedirect => "307 Temporary Redirect",
                StatusCode::PermanentRedirect => "308 Permanent Redirect",
                StatusCode::BadRequest => "400 Bad Request",
                StatusCode::Unauthorized => "401 Unauthorized",
                StatusCode::PaymentRequired => "402 Payment Required",
                StatusCode::Forbidden => "403 Forbidden",
                StatusCode::NotFound => "404 Not Found",
                StatusCode::MethodNotAllowed => "405 Method Not Allowed",
                StatusCode::NotAcceptable => "406 Not Acceptable",
                StatusCode::ProxyAuthRequired => "407 Proxy Authentication Required",
                StatusCode::RequestTimeout => "408 Request Timeout",
                StatusCode::Conflict => "409 Conflict",
                StatusCode::Gone => "410 Gone",
                StatusCode::LengthRequired => "411 Length Required",
                StatusCode::PreconditionFailed => "412 Precondition Failed",
                StatusCode::PayloadTooLarge => "413 Payload Too Large",
                StatusCode::UriTooLong => "414 URI Too Long",
                StatusCode::UnsupportedMediaType => "415 Unsupported Media Type",
                StatusCode::RangeNotSatisfiable => "416 Range Not Satisfiable",
                StatusCode::ExpectationFailed => "417 Expectation Failed",
                StatusCode::ImATeapot => "418 I'm a teapot",
                StatusCode::MisdirectedRequest => "421 Misdirected Request",
                StatusCode::UnprocessableEntity => "422 Unprocessable Entity",
                StatusCode::Locked => "423 Locked",
                StatusCode::FailedDependency => "424 Failed Dependency",
                StatusCode::TooEarly => "425 Too Early",
                StatusCode::UpgradeRequired => "426 Upgrade Required",
                StatusCode::PreconditionRequired => "428 Precondition Required",
                StatusCode::TooManyRequests => "429 Too Many Requests",
                StatusCode::RequestHeaderFieldsTooLarge => "431 Request Header Fields Too Large",
                StatusCode::UnavailableForLegalReasons => "451 Unavailable For Legal Reasons",
                StatusCode::InternalServerError => "500 Internal Server Error",
                StatusCode::NotImplemented => "501 Not Implemented",
                StatusCode::BadGateway => "502 Bad Gateway",
                StatusCode::ServiceUnavailable => "503 Service Unavailable",
                StatusCode::GatewayTimeout => "504 Gateway Timeout",
                StatusCode::HttpVersionNotSupported => "505 HTTP Version Not Supported",
                StatusCode::VariantAlsoNegotiates => "506 Variant Also Negotiates",
                StatusCode::InsufficientStorage => "507 Insufficient Storage",
                StatusCode::LoopDetected => "508 Loop Detected",
                StatusCode::NotExtended => "510 Not Extended",
                StatusCode::NetworkAuthenticationRequired => "511 Network Authentication Required",
            }
        )
    }
}

#[derive(Debug)]
struct HttpRequest {
    method: Method,
    path: String,
    version: String,
    headers: HashMap<HeaderType, String>,
    body: Vec<u8>,
}

impl From<&TcpStream> for HttpRequest {
    fn from(connection: &TcpStream) -> Self {
        let mut reader = BufReader::new(connection);

        let mut request_line = String::new();
        reader.read_line(&mut request_line).unwrap();
        let parts: Vec<_> = request_line.split_whitespace().collect();
        let method = parts[0].parse().unwrap();
        let path = parts[1].to_string();
        let version = parts[2].to_string();

        let mut headers = HashMap::new();
        loop {
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            if line.trim().is_empty() {
                break;
            }
            if line == CRLF {
                reader.read_line(&mut line).unwrap();
                break;
            }
            if let Some((header_type, value)) = HeaderType::parse(&line) {
                headers.insert(header_type, value);
            }
        }

        let mut body = Vec::new();
        if let Some(content_length_str) = headers.get(&HeaderType::ContentLength) {
            let content_length: usize = content_length_str.parse().unwrap();
            body.resize(content_length, 0);
            reader.read_exact(&mut body).unwrap();
        }

        Self {
            method,
            path,
            version,
            headers,
            body,
        }
    }
}

#[derive(Debug)]
struct HttpResponse {
    version: String,
    status_code: StatusCode,
    headers: HashMap<HeaderType, String>,
    body: Vec<u8>,
}

impl HttpResponse {
    fn write_to<W: Write>(&self, writer: &mut W) -> Result<(), Error> {
        write!(writer, "{} {}{CRLF}", self.version, self.status_code)?;

        for (key, value) in &self.headers {
            write!(writer, "{}: {}{CRLF}", key, value)?;
        }

        write!(writer, "{CRLF}")?;
        writer.write_all(&self.body)?;
        writer.flush()?;

        Ok(())
    }
}

fn connection_handler(mut conn: TcpStream, dir: Arc<String>) -> Result<(), Error> {
    let request = HttpRequest::from(&conn);
    let mut response = HttpResponse {
        version: request.version,
        status_code: StatusCode::Ok,
        headers: HashMap::new(),
        body: String::new().into(),
    };

    if let Some(values) = request.headers.get(&HeaderType::AcceptEncoding) {
        let values: Vec<&str> = if values.contains(", ") {
            values.split(", ").collect()
        } else {
            vec![values]
        };
        for value in values {
            if let Ok(encoding_type) = EncodingType::from_str(value) {
                if let Some(encoding_types) = response.headers.get_mut(&HeaderType::ContentEncoding)
                {
                    let new_encoding_types = format!("{}, {}", encoding_types, encoding_type);
                    response
                        .headers
                        .insert(HeaderType::ContentEncoding, new_encoding_types);
                } else {
                    response
                        .headers
                        .insert(HeaderType::ContentEncoding, encoding_type.to_string());
                }
            }
        }
    }

    if request.path.starts_with("/echo") {
        let parts: Vec<_> = request.path.split(|s| s == '/').collect();
        let str = parts[2].to_string();
        response
            .headers
            .insert(HeaderType::ContentType, "text/plain".to_owned());

        if let Some(value) = response.headers.get(&HeaderType::ContentEncoding) {
            if value == "gzip" {
                let mut encoder = GzEncoder::new(vec![], Compression::default());
                encoder.write_all(str.as_bytes())?;
                response.body = encoder.finish()?;
            } else {
                response.body = str.into();
            }
        } else {
            response.body = str.into();
        }
    } else if request.path.starts_with("/files") {
        let parts: Vec<_> = request.path.split(|s| s == '/').collect();
        let file_name = parts[2].to_string();
        let file_path = format!("{}/{}", dir, file_name);

        match request.method {
            Method::Get => match read(file_path) {
                Ok(file) => {
                    response.headers.insert(
                        HeaderType::ContentType,
                        "application/octet-stream".to_owned(),
                    );
                    response.body = file;
                }
                Err(_) => response.status_code = StatusCode::NotFound,
            },
            Method::Post => match write(file_path, request.body) {
                Ok(_) => response.status_code = StatusCode::Created,
                Err(err) => {
                    response.status_code = StatusCode::InternalServerError;
                    response.body = err.to_string().as_bytes().to_vec();
                }
            },
            _ => response.status_code = StatusCode::MethodNotAllowed,
        }
    } else if request.path == "/user-agent" {
        let user_agent = request
            .headers
            .get(&HeaderType::UserAgent)
            .unwrap()
            .to_string();
        response
            .headers
            .insert(HeaderType::ContentType, "text/plain".to_owned());

        response.body = user_agent.into();
    } else if request.path == "/" {
    } else {
        response.status_code = StatusCode::NotFound;
    }

    response
        .headers
        .insert(HeaderType::ContentLength, response.body.len().to_string());

    response.write_to(&mut conn)
}

fn main() -> Result<(), Error> {
    let mut directory = String::new();
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        if arg == "--directory" {
            if let Some(dir) = args.next() {
                directory = dir;
            }
        }
    }
    let directory = if !directory.is_empty() {
        Arc::new(directory)
    } else {
        Arc::new("./".to_owned())
    };

    let listener = TcpListener::bind("127.0.0.1:4221")?;

    for connection in listener.incoming() {
        match connection {
            Ok(conn) => {
                let dir = directory.clone();
                thread::spawn(move || {
                    if let Err(err) = connection_handler(conn, dir) {
                        eprintln!("Connection handler error: {}", err);
                    }
                });
            }
            Err(err) => {
                eprintln!("Failed to accept connection: {}", err);
            }
        }
    }

    Ok(())
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
