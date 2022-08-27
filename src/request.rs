use crate::common::*;
use crate::cookie::RequestCookie;
use crate::mime::Mime;
use crate::response::Response;
use crate::search::SearchParams;
use std::io::{Error as IoError, ErrorKind, Read, Result as IoResult, Write};
use std::net::TcpStream;
use urlencoding::decode;

#[derive(Debug)]
pub struct Request {
    pub method: Method,
    pub version: Version,
    pub uri: Uri,
    pub headers: Vec<Header>,
    pub body: String,
    pub raw: String,
    pub stream: TcpStream,
    pub responded: bool,
}

impl Request {
    pub fn respond(&mut self, response: Response) -> IoResult<usize> {
        if self.responded {
            return Err(IoError::new(ErrorKind::Other, "Request already responded"));
        }

        let response = response.to_vector(&self);
        let size = self.stream.write(&response)?;
        self.responded = true;

        return Ok(size);
    }

    pub fn get_header(&self, name: &str) -> Option<&Header> {
        for header in self.headers.iter() {
            if header.name().to_lowercase() == name.to_lowercase() {
                return Some(header);
            }
        }

        return None;
    }

    pub fn get_cookie(&self, name: &str) -> Option<&RequestCookie> {
        let cookies = if let Header::Cookie(cookies) = self.get_header("Cookie")? {
            cookies
        } else {
            return None;
        };

        for cookie in cookies.iter() {
            if cookie.name().to_lowercase() == name.to_lowercase() {
                return Some(cookie);
            }
        }

        return None;
    }
}

pub fn handle_connection(mut stream: TcpStream) -> IoResult<Request> {
    let mut buffer = [0; 2048];
    let mut parsed_bytes = 0;

    match stream.read(&mut buffer) {
        Ok(mut bytes_read) => {
            bytes_read += parsed_bytes;

            let method = {
                let mut raw_method = String::new();

                for byte in buffer[parsed_bytes..bytes_read].iter() {
                    parsed_bytes += 1;
                    if *byte == b' ' {
                        break;
                    }
                    raw_method.push(*byte as char);
                }

                match raw_method.to_uppercase().as_str() {
                    "GET" => Method::Get,
                    "POST" => Method::Post,
                    "PUT" => Method::Put,
                    "DELETE" => Method::Delete,
                    "HEAD" => Method::Head,
                    "PATCH" => Method::Patch,
                    "OPTIONS" => Method::Options,
                    "CONNECT" => Method::Connect,
                    "TRACE" => Method::Trace,
                    _ => Method::Unknown(raw_method),
                }
            };
            let path = {
                let mut raw_path = String::new();

                for byte in buffer[parsed_bytes..bytes_read].iter() {
                    parsed_bytes += 1;
                    if *byte == b' ' {
                        break;
                    }
                    raw_path.push(*byte as char);
                }
                raw_path
            };
            let version = {
                let mut raw_version = String::new();

                for byte in buffer[parsed_bytes..bytes_read].iter() {
                    parsed_bytes += 1;
                    if *byte == b'\r' {
                        break;
                    }
                    raw_version.push(*byte as char);
                }
                match raw_version.as_str() {
                    "HTTP/1.0" => Version::Http10,
                    "HTTP/1.1" => Version::Http11,
                    "HTTP/2.0" => Version::Http20,
                    _ => Version::Unknown(raw_version),
                }
            };

            let mut host = String::new();

            let (headers, body) = {
                let raw = String::from_utf8_lossy(&buffer[(parsed_bytes + 1)..bytes_read]);
                let mut split = raw.split("\r\n\r\n");

                let raw_headers = split.next().unwrap();

                let rest = split.collect::<Vec<&str>>().join("\r\n");

                let split = raw_headers.split("\r\n");

                let mut headers = Vec::new();

                for header in split {
                    let mut split = header.split(": ");

                    let (name, value) = (
                        split.next().unwrap().to_string().to_lowercase(),
                        split.next().unwrap().to_string(),
                    );
                    headers.push(match name.as_str() {
                        "connection" => Header::Connection(match value.to_lowercase().as_str() {
                            "keep-alive" => Connection::KeepAlive,
                            "close" => Connection::Close,
                            "upgrade" => Connection::Upgrade,
                            _ => Connection::Unknown(value),
                        }),
                        "content-length" => Header::ContentLength(value.parse().unwrap()),
                        "content-type" => Header::ContentType(match Mime::parse(&value) {
                            Ok(mime) => mime,
                            Err(_) => {
                                return Err(IoError::new(
                                    ErrorKind::Other,
                                    format!("Invalid content type: {}", &value),
                                ))
                            }
                        }),
                        "host" => {
                            host = String::from(value.as_str());
                            Header::Host(value)
                        }
                        "user-agent" => Header::UserAgent(value),
                        "accept" => Header::Accept(value),
                        "accept-encoding" => Header::AcceptEncoding(match value.parse() {
                            Ok(encoding) => encoding,
                            Err(_) => {
                                return Err(IoError::new(
                                    ErrorKind::Other,
                                    format!("Invalid accept encoding: {}", &value),
                                ))
                            }
                        }),
                        "accept-language" => Header::AcceptLanguage(value),
                        "accept-charset" => Header::AcceptCharset(value),
                        "accept-datetime" => Header::AcceptDatetime(value),
                        "accept-ranges" => Header::AcceptRanges(value),
                        "cache-control" => Header::CacheControl(Cache::parse(&value)),
                        "cookie" => match RequestCookie::parse(value) {
                            Ok(cookie) => Header::Cookie(cookie),
                            Err(_) => {
                                return Err(IoError::new(ErrorKind::InvalidInput, "Invalid cookie"))
                            }
                        },
                        "date" => Header::Date(value),
                        "pragma" => match Cache::parse_once(&value) {
                            Some(cache) => Header::Pragma(cache),
                            None => {
                                return Err(IoError::new(ErrorKind::InvalidInput, "Invalid pragma"))
                            }
                        },
                        "trailer" => Header::Trailer(value),
                        "transfer-encoding" => Header::TransferEncoding(value),
                        "upgrade" => Header::Upgrade(value),
                        "proxy-connection" => {
                            Header::ProxyConnection(match value.to_lowercase().as_str() {
                                "keep-alive" => Connection::KeepAlive,
                                "close" => Connection::Close,
                                "upgrade" => Connection::Upgrade,
                                _ => Connection::Unknown(value),
                            })
                        }
                        "server" => Header::Server(value),
                        "origin" => Header::Origin(value),
                        "dnt" => Header::Dnt(match value.to_lowercase().as_str() {
                            "0" => Dnt::PrefersAllowTrack,
                            "1" => Dnt::PrefersNoTrack,
                            "null" => Dnt::NotSpecified,
                            _ => {
                                return Err(IoError::new(
                                    ErrorKind::InvalidInput,
                                    "Invalid DNT value",
                                ))
                            }
                        }),
                        _ => Header::Unknown(name, value),
                    });
                }

                (headers, rest)
            };

            let uri = if path.starts_with("http://") || path.starts_with("https://") {
                let mut split = path.split("//");

                let scheme = {
                    let raw = split.next().unwrap();

                    raw[..raw.len() - 1].to_string()
                };

                let (host, mut path) = {
                    let mut split = split.next().unwrap().split("/");

                    let host = split.next().unwrap();
                    let path = format!("/{}", split.next().unwrap_or(""));

                    (host.to_string(), path)
                };

                let search = if path.contains('?') {
                    let index = path.match_indices('?').next().unwrap().0;
                    let search_raw = String::from(&path[index..]);
                    path = decode(&path[..index]).unwrap().into_owned();

                    match SearchParams::parse(search_raw) {
                        Ok(v) => v,
                        Err(_) => return Err(IoError::new(ErrorKind::Other, "")),
                    }
                } else {
                    SearchParams::empty()
                };

                Uri {
                    scheme,
                    host,
                    path,
                    search,
                }
            } else {
                Uri::absolute(host.to_string(), path.to_string())
            };

            return Ok(Request {
                method,
                uri,
                version,
                headers,
                body,
                raw: String::from_utf8_lossy(&buffer[..bytes_read]).to_string(),
                stream,
                responded: false,
            });
        }
        Err(err) => {
            println!("Error: {}", err);

            return Err(err);
        }
    }
}
