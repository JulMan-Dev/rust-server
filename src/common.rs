use crate::accept::AcceptEncodings;
use crate::cookie::{RequestCookie, ResponseCookie};
use crate::mime::Mime;
use crate::response::BodyEncoding;
use crate::search::SearchParams;
use std::ops::Add;
use urlencoding::decode;

#[derive(Debug)]
pub enum Method {
    Get,
    Post,
    Put,
    Delete,
    Head,
    Patch,
    Options,
    Connect,
    Trace,
    Unknown(String),
}

impl ToString for Method {
    fn to_string(&self) -> String {
        match self {
            Method::Get => "GET".to_string(),
            Method::Post => "POST".to_string(),
            Method::Put => "PUT".to_string(),
            Method::Delete => "DELETE".to_string(),
            Method::Head => "HEAD".to_string(),
            Method::Patch => "PATCH".to_string(),
            Method::Options => "OPTIONS".to_string(),
            Method::Connect => "CONNECT".to_string(),
            Method::Trace => "TRACE".to_string(),
            Method::Unknown(raw_method) => raw_method.clone(),
        }
    }
}

#[derive(Debug)]
pub enum Version {
    Http10,
    Http11,
    Http20,
    Unknown(String),
}

impl ToString for Version {
    fn to_string(&self) -> String {
        match self {
            Version::Http10 => "HTTP/1.0".to_string(),
            Version::Http11 => "HTTP/1.1".to_string(),
            Version::Http20 => "HTTP/2.0".to_string(),
            Version::Unknown(raw_version) => raw_version.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Connection {
    KeepAlive,
    Close,
    Upgrade,
    Unknown(String),
}

impl ToString for Connection {
    fn to_string(&self) -> String {
        match self {
            Connection::KeepAlive => "keep-alive".to_string(),
            Connection::Close => "close".to_string(),
            Connection::Upgrade => "upgrade".to_string(),
            Connection::Unknown(raw_connection) => raw_connection.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Dnt {
    PrefersAllowTrack,
    PrefersNoTrack,
    NotSpecified,
}

impl ToString for Dnt {
    fn to_string(&self) -> String {
        match self {
            Dnt::PrefersAllowTrack => "1".to_string(),
            Dnt::PrefersNoTrack => "0".to_string(),
            Dnt::NotSpecified => "null".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Cache {
    MaxAge(u32),
    NoCache,
    MustRevalidate,
    ProxyRevalidate,
    NoStore,
    Private,
    Public,
    MustUnderstand,
    NoTransform,
    Immutable,
    StaleWhileRevalidate(u32),
    StaleIfError(u32),
    MaxStale(u32),
    MinFresh(u32),
    OnlyIfCached,
}

impl ToString for Cache {
    fn to_string(&self) -> String {
        match self {
            Cache::MaxAge(max_age) => format!("max-age={}", max_age),
            Cache::NoCache => "no-cache".to_string(),
            Cache::MustRevalidate => "must-revalidate".to_string(),
            Cache::ProxyRevalidate => "proxy-revalidate".to_string(),
            Cache::NoStore => "no-store".to_string(),
            Cache::Private => "private".to_string(),
            Cache::Public => "public".to_string(),
            Cache::MustUnderstand => "must-understand".to_string(),
            Cache::NoTransform => "no-transform".to_string(),
            Cache::Immutable => "immutable".to_string(),
            Cache::StaleWhileRevalidate(age) => format!("stale-while-revalidate={}", age),
            Cache::StaleIfError(age) => format!("stale-if-error={}", age),
            Cache::MaxStale(age) => format!("max-stale={}", age),
            Cache::MinFresh(age) => format!("min-fresh={}", age),
            Cache::OnlyIfCached => "only-if-cached".to_string(),
        }
    }
}

impl Cache {
    pub fn parse_once(raw_cache: &String) -> Option<Cache> {
        if raw_cache.contains("=") {
            let (key, value) = raw_cache.split_at(raw_cache.find('=')?);

            return match key {
                "max-age" => Some(Cache::MaxAge(value.parse().ok()?)),
                "stale-while-revalidate" => Some(Cache::StaleWhileRevalidate(value.parse().ok()?)),
                "stale-if-error" => Some(Cache::StaleIfError(value.parse().ok()?)),
                "max-stale" => Some(Cache::MaxStale(value.parse().ok()?)),
                "min-fresh" => Some(Cache::MinFresh(value.parse().ok()?)),
                _ => None,
            };
        }

        match raw_cache.as_str() {
            "no-cache" => Some(Cache::NoCache),
            "must-revalidate" => Some(Cache::MustRevalidate),
            "proxy-revalidate" => Some(Cache::ProxyRevalidate),
            "no-store" => Some(Cache::NoStore),
            "private" => Some(Cache::Private),
            "public" => Some(Cache::Public),
            "must-understand" => Some(Cache::MustUnderstand),
            "no-transform" => Some(Cache::NoTransform),
            "immutable" => Some(Cache::Immutable),
            "only-if-cached" => Some(Cache::OnlyIfCached),
            _ => None,
        }
    }

    pub fn parse(raw_cache: &String) -> Vec<Cache> {
        raw_cache
            .split(',')
            .map(|raw_cache| {
                let mut d = String::from(raw_cache);

                while d.starts_with(" ") {
                    d.remove(0);
                }

                while d.ends_with(" ") {
                    d.remove(d.len() - 1);
                }

                Cache::parse_once(&d)
            })
            .filter_map(|cache| cache)
            .collect()
    }

    pub fn format(vec: &Vec<Cache>) -> String {
        let mut val = vec.iter().fold(String::new(), |acc, current| {
            acc.add(&current.to_string()).add(", ")
        });

        while val.ends_with(" ") || val.ends_with(",") {
            val.remove(val.len() - 1);
        }

        val
    }
}

#[derive(Debug, Clone)]
pub enum Header {
    Connection(Connection),
    ContentLength(u64),
    ContentType(Mime),
    Host(String),
    UserAgent(String),
    Accept(String),
    AcceptEncoding(AcceptEncodings),
    AcceptLanguage(String),
    AcceptCharset(String),
    AcceptDatetime(String),
    AcceptRanges(String),
    CacheControl(Vec<Cache>),
    Cookie(Vec<RequestCookie>),
    Date(String),
    Pragma(Cache),
    Trailer(String),
    TransferEncoding(String),
    Upgrade(String),
    ProxyConnection(Connection),
    Server(String),
    Origin(String),
    Dnt(Dnt),
    SetCookie(ResponseCookie),
    Location(String),
    ContentEncoding(Vec<BodyEncoding>),
    Unknown(String, String),
}

impl ToString for Header {
    fn to_string(&self) -> String {
        match self {
            Header::Connection(connection) => format!("Connection: {}\r\n", connection.to_string()),
            Header::ContentLength(content_length) => {
                format!("Content-Length: {}\r\n", content_length)
            }
            Header::ContentType(content_type) => {
                format!("Content-Type: {}\r\n", content_type.to_string())
            }
            Header::Host(host) => format!("Host: {}\r\n", host),
            Header::UserAgent(user_agent) => format!("User-Agent: {}\r\n", user_agent),
            Header::Accept(accept) => format!("Accept: {}\r\n", accept),
            Header::AcceptEncoding(accept_encoding) => {
                format!("Accept-Encoding: {}\r\n", accept_encoding.to_string())
            }
            Header::AcceptLanguage(accept_language) => {
                format!("Accept-Language: {}\r\n", accept_language)
            }
            Header::AcceptCharset(accept_charset) => {
                format!("Accept-Charset: {}\r\n", accept_charset)
            }
            Header::AcceptDatetime(accept_datetime) => {
                format!("Accept-Datetime: {}\r\n", accept_datetime)
            }
            Header::AcceptRanges(accept_ranges) => format!("Accept-Ranges: {}\r\n", accept_ranges),
            Header::CacheControl(cache_control) => {
                format!("Cache-Control: {}\r\n", Cache::format(cache_control))
            }
            Header::Cookie(cookie) => {
                let mut out = "Cookie: ".to_string();

                for cookie in cookie {
                    out += &format!("{}={}; ", cookie.name(), cookie.value());
                }

                out += "\r\n";

                out
            }
            Header::Date(date) => format!("Date: {}\r\n", date),
            Header::Pragma(pragma) => format!("Pragma: {}\r\n", pragma.to_string()),
            Header::Trailer(trailer) => format!("Trailer: {}\r\n", trailer),
            Header::TransferEncoding(transfer_encoding) => {
                format!("Transfer-Encoding: {}\r\n", transfer_encoding)
            }
            Header::Upgrade(upgrade) => format!("Upgrade: {}\r\n", upgrade),
            Header::ProxyConnection(proxy_connection) => {
                format!("Proxy-Connection: {}\r\n", proxy_connection.to_string())
            }
            Header::Server(server) => format!("Server: {}\r\n", server),
            Header::Origin(origin) => format!("Origin: {}\r\n", origin),
            Header::Dnt(dnt) => format!(
                "DNT: {}",
                match dnt {
                    Dnt::PrefersAllowTrack => "0",
                    Dnt::PrefersNoTrack => "1",
                    Dnt::NotSpecified => "null",
                }
            ),
            Header::SetCookie(set_cookie) => format!("Set-Cookie: {}\r\n", set_cookie.to_string()),
            Header::Location(location) => format!("Location: {}\r\n", location),
            Header::ContentEncoding(content_encoding) => {
                let mut out = "Content-Encoding: ".to_string();

                for encoding in content_encoding {
                    out += &format!("{}, ", encoding.to_string());
                }

                out.pop();
                out.pop();

                out += "\r\n";

                out
            }
            Header::Unknown(name, value) => format!("{}: {}\r\n", name, value),
        }
    }
}

impl Header {
    pub fn name(&self) -> String {
        match self {
            Header::Connection(_) => "Connection",
            Header::ContentLength(_) => "Content-Length",
            Header::ContentType(_) => "Content-Type",
            Header::Host(_) => "Host",
            Header::UserAgent(_) => "User-Agent",
            Header::Accept(_) => "Accept",
            Header::AcceptEncoding(_) => "Accept-Encoding",
            Header::AcceptLanguage(_) => "Accept-Language",
            Header::AcceptCharset(_) => "Accept-Charset",
            Header::AcceptDatetime(_) => "Accept-Datetime",
            Header::AcceptRanges(_) => "Accept-Ranges",
            Header::CacheControl(_) => "Cache-Control",
            Header::Cookie(_) => "Cookie",
            Header::Date(_) => "Date",
            Header::Pragma(_) => "Pragma",
            Header::Trailer(_) => "Trailer",
            Header::TransferEncoding(_) => "Transfer-Encoding",
            Header::Upgrade(_) => "Upgrade",
            Header::ProxyConnection(_) => "Proxy-Connection",
            Header::Server(_) => "Server",
            Header::Origin(_) => "Origin",
            Header::Dnt(_) => "DNT",
            Header::SetCookie(_) => "Set-Cookie",
            Header::Location(_) => "Location",
            Header::ContentEncoding(_) => "Content-Encoding",
            Header::Unknown(ref a, _) => a.as_str(),
        }
        .to_string()
    }
}

#[derive(Debug)]
pub struct Uri {
    pub scheme: String,
    pub host: String,
    pub path: String,
    pub search: SearchParams,
}

impl Uri {
    pub fn absolute(host: String, mut path: String) -> Uri {
        let search = if path.contains('?') {
            let index = path.match_indices('?').next().unwrap().0;

            let search_raw = String::from(&path[index..]);

            path = decode(&path[..index]).unwrap().into_owned();

            match SearchParams::parse(search_raw) {
                Ok(v) => v,
                Err(_) => SearchParams::empty(),
            }
        } else {
            SearchParams::empty()
        };

        Uri {
            scheme: "http".to_string(),
            host,
            path,
            search,
        }
    }
}

impl ToString for Uri {
    fn to_string(&self) -> String {
        format!(
            "{}://{}{}{}",
            self.scheme,
            self.host,
            self.path,
            self.search.to_string()
        )
    }
}

#[derive(Debug, Clone)]
pub enum Status {
    SwitchingProtocols,
    Ok,
    Created,
    Accepted,
    NoContent,
    ResetContent,
    PartialContent,
    MultipleChoices,
    MovedPermanently,
    MovedTemporarily,
    NotModified,
    BadRequest,
    Unauthorized,
    Forbidden,
    NotFound,
    MethodNotAllowed,
    NotAcceptable,
    ProxyAuthenticationRequired,
    RequestTimeout,
    Conflict,
    Gone,
    LengthRequired,
    PreconditionFailed,
    RequestEntityTooLarge,
    RequestUriTooLong,
    UnsupportedMediaType,
    RequestedRangeNotSatisfiable,
    ExpectationFailed,
    InternalServerError,
    NotImplemented,
    BadGateway,
    ServiceUnavailable,
    GatewayTimeout,
    HttpVersionNotSupported,
    Unknown(u16),
    Custom(u16, String),
}

impl ToString for Status {
    fn to_string(&self) -> String {
        match self {
            Status::SwitchingProtocols => "HTTP/1.1 101 Switching Protocols\r\n".to_string(),
            Status::Ok => "200 OK".to_string(),
            Status::Created => "201 Created".to_string(),
            Status::Accepted => "202 Accepted".to_string(),
            Status::NoContent => "204 No Content".to_string(),
            Status::ResetContent => "205 Reset Content".to_string(),
            Status::PartialContent => "206 Partial Content".to_string(),
            Status::MultipleChoices => "300 Multiple Choices".to_string(),
            Status::MovedPermanently => "301 Moved Permanently".to_string(),
            Status::MovedTemporarily => "302 Moved Temporarily".to_string(),
            Status::NotModified => "304 Not Modified".to_string(),
            Status::BadRequest => "400 Bad Request".to_string(),
            Status::Unauthorized => "401 Unauthorized".to_string(),
            Status::Forbidden => "403 Forbidden".to_string(),
            Status::NotFound => "404 Not Found".to_string(),
            Status::MethodNotAllowed => "405 Method Not Allowed".to_string(),
            Status::NotAcceptable => "406 Not Acceptable".to_string(),
            Status::ProxyAuthenticationRequired => "407 Proxy Authentication Required".to_string(),
            Status::RequestTimeout => "408 Request Timeout".to_string(),
            Status::Conflict => "409 Conflict".to_string(),
            Status::Gone => "410 Gone".to_string(),
            Status::LengthRequired => "411 Length Required".to_string(),
            Status::PreconditionFailed => "412 Precondition Failed".to_string(),
            Status::RequestEntityTooLarge => "413 Request Entity Too Large".to_string(),
            Status::RequestUriTooLong => "414 Request-URI Too Long".to_string(),
            Status::UnsupportedMediaType => "415 Unsupported Media Type".to_string(),
            Status::RequestedRangeNotSatisfiable => {
                "416 Requested Range Not Satisfiable".to_string()
            }
            Status::ExpectationFailed => "417 Expectation Failed".to_string(),
            Status::InternalServerError => "500 Internal Server Error".to_string(),
            Status::NotImplemented => "501 Not Implemented".to_string(),
            Status::BadGateway => "502 Bad Gateway".to_string(),
            Status::ServiceUnavailable => "503 Service Unavailable".to_string(),
            Status::GatewayTimeout => "504 Gateway Timeout".to_string(),
            Status::HttpVersionNotSupported => "505 HTTP Version Not Supported".to_string(),
            Status::Unknown(code) => format!("{} Unknown", code),
            Status::Custom(code, reason) => format!("{} {}", code, reason),
        }
    }
}

impl Status {
    pub fn from_code(code: u16) -> Status {
        match code {
            101 => Status::SwitchingProtocols,
            200 => Status::Ok,
            201 => Status::Created,
            202 => Status::Accepted,
            204 => Status::NoContent,
            205 => Status::ResetContent,
            206 => Status::PartialContent,
            300 => Status::MultipleChoices,
            301 => Status::MovedPermanently,
            302 => Status::MovedTemporarily,
            304 => Status::NotModified,
            400 => Status::BadRequest,
            401 => Status::Unauthorized,
            403 => Status::Forbidden,
            404 => Status::NotFound,
            405 => Status::MethodNotAllowed,
            406 => Status::NotAcceptable,
            407 => Status::ProxyAuthenticationRequired,
            408 => Status::RequestTimeout,
            409 => Status::Conflict,
            410 => Status::Gone,
            411 => Status::LengthRequired,
            412 => Status::PreconditionFailed,
            413 => Status::RequestEntityTooLarge,
            414 => Status::RequestUriTooLong,
            415 => Status::UnsupportedMediaType,
            416 => Status::RequestedRangeNotSatisfiable,
            500 => Status::InternalServerError,
            501 => Status::NotImplemented,
            502 => Status::BadGateway,
            503 => Status::ServiceUnavailable,
            504 => Status::GatewayTimeout,
            505 => Status::HttpVersionNotSupported,
            _ => Status::Unknown(code),
        }
    }
}
