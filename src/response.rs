use crate::common::{Header, Method, Status};
use crate::cookie::ResponseCookie;
use crate::mime::Mime;
use crate::request::Request;
use brotli::CompressorReader;
use flate2::write::{DeflateEncoder, GzEncoder};
use flate2::Compression;
use std::io::{Error as IoError, ErrorKind, Read, Write};

#[derive(Debug)]
pub enum ResponseBody {
    Text(String),
    Binary(Vec<u8>),
    None,
}

#[derive(Debug, Clone, Copy)]
pub enum BodyEncoding {
    Gzip,
    Deflate,
    Brotli,
}

impl ToString for BodyEncoding {
    fn to_string(&self) -> String {
        match self {
            BodyEncoding::Gzip => "gzip",
            BodyEncoding::Deflate => "deflate",
            BodyEncoding::Brotli => "br",
        }
        .to_string()
    }
}

#[derive(Debug)]
pub struct Response {
    pub status: Status,
    pub headers: Vec<Header>,
    pub body: ResponseBody,
    pub encoding: (Option<BodyEncoding>, Option<CompressionLevel>),
}

#[derive(Debug, Clone, Copy)]
pub struct CompressionLevel(u32);

impl CompressionLevel {
    pub fn new(level: u32) -> Self {
        CompressionLevel(level)
    }

    pub const fn none() -> CompressionLevel {
        CompressionLevel(0)
    }

    pub const fn fast() -> CompressionLevel {
        CompressionLevel(1)
    }

    pub const fn best() -> CompressionLevel {
        CompressionLevel(9)
    }

    pub fn level(&self) -> u32 {
        self.0
    }
}

fn push_str(vec: &mut Vec<u8>, data: &String) {
    for c in data.chars() {
        vec.push(c as u8);
    }
}

impl Response {
    pub fn new(
        status: Status,
        headers: Vec<Header>,
        body: ResponseBody,
        encoding: (Option<BodyEncoding>, Option<CompressionLevel>),
    ) -> Response {
        Response {
            status,
            headers,
            body,
            encoding,
        }
    }

    pub fn empty() -> Response {
        Response {
            status: Status::Ok,
            headers: Vec::new(),
            body: ResponseBody::None,
            encoding: (None, None),
        }
    }

    pub fn redirect(target: String, status: Option<Status>) -> Response {
        let mut response = Response::empty();

        response
            .set_status(status.unwrap_or(Status::MovedTemporarily))
            .set_body(ResponseBody::Text(format!("Redirecting to {}", &target)))
            .add_header(Header::Location(target))
            .add_header(Header::ContentType(Mime::Text("plain".to_string(), None)));

        response
    }

    pub fn to_vector(&self, request: &Request) -> Vec<u8> {
        let mut headers: Vec<Header> = vec![];

        for header in self.headers.iter() {
            headers.push(header.clone());
        }

        let has_content_length = headers.iter().any(|h| match h {
            Header::ContentLength(_) => true,
            _ => false,
        });

        if !has_content_length {
            match &self.body {
                ResponseBody::Text(ref text) => {
                    headers.push(Header::ContentLength(text.len() as u64))
                }
                ResponseBody::Binary(ref vec) => {
                    headers.push(Header::ContentLength(vec.len() as u64))
                }
                _ => {}
            };
        }

        let support_encoding = if let (Some(encoding), _) = self.encoding {
            let support_encoding = {
                let accept_encodings = request.get_header("accept-encoding");

                match accept_encodings {
                    Some(ref accept_encodings) => match accept_encodings {
                        Header::AcceptEncoding(ref accept_encodings) => {
                            accept_encodings.accept(&encoding)
                        }
                        _ => false,
                    },
                    None => false,
                }
            };

            if support_encoding {
                headers.push(Header::ContentEncoding(vec![encoding]));
            }

            support_encoding
        } else {
            false
        };

        headers.sort_by(|a, b| a.name().cmp(&b.name()));

        let mut response: Vec<u8> = vec![];

        push_str(
            &mut response,
            &format!(
                "{} {}\r\n",
                request.version.to_string(),
                self.status.to_string()
            ),
        );
        for header in &headers {
            push_str(&mut response, &format!("{}", header.to_string()));
        }
        push_str(&mut response, &"\r\n".to_string());
    
        let should_print_body = match (&request.method, &self.status) {
            (Method::Head, _) => false,
            (_, Status::NoContent) => false,
            (_, Status::Unknown(s)) => *s != 204,
            (_, Status::Custom(s, _)) => *s != 204,
            _ => true,
        };

        if should_print_body {
            let mut data = match &self.body {
                ResponseBody::Text(text) => text.chars().map(|c| c as u8).collect::<Vec<_>>(),
                ResponseBody::Binary(vec) => vec.clone(),
                ResponseBody::None => vec![],
            };

            if support_encoding {
                if let (Some(encoding), level) = self.encoding {
                    let res = match (encoding, level) {
                        (BodyEncoding::Gzip, l) => {
                            let level = Compression::new(
                                (match l {
                                    Some(l) => l,
                                    None => CompressionLevel::fast(),
                                })
                                .level(),
                            );

                            let mut encoder: GzEncoder<Vec<u8>> = GzEncoder::new(Vec::new(), level);

                            if let Ok(_) = encoder.write(&data) {
                                encoder.finish()
                            } else {
                                Err(IoError::new(ErrorKind::Other, ""))
                            }
                        }
                        (BodyEncoding::Deflate, l) => {
                            let level = Compression::new(
                                (match l {
                                    Some(l) => l,
                                    None => CompressionLevel::fast(),
                                })
                                .level(),
                            );

                            let mut encoder: DeflateEncoder<Vec<u8>> =
                                DeflateEncoder::new(Vec::new(), level);

                            if let Ok(_) = encoder.write(&data) {
                                encoder.finish()
                            } else {
                                Err(IoError::new(ErrorKind::Other, ""))
                            }
                        }
                        (BodyEncoding::Brotli, l) => {
                            let level = (match l {
                                Some(l) => l,
                                None => CompressionLevel::fast(),
                            })
                            .level();

                            let mut reader =
                                CompressorReader::new(data.as_slice(), data.len(), level, 20);
                            let mut buf = Vec::new();

                            if let Ok(_) = reader.read_to_end(&mut buf) {
                                Ok(buf)
                            } else {
                                Err(IoError::new(ErrorKind::Other, ""))
                            }
                        }
                    };

                    if let Ok(new_data) = res {
                        data.clear();

                        for d in new_data {
                            data.push(d);
                        }
                    }
                }
            }

            for u in data {
                response.push(u);
            }
        }
        response
    }

    pub fn add_cookie(&mut self, cookie: ResponseCookie) -> &mut Self {
        self.headers.push(Header::SetCookie(cookie));

        self
    }

    pub fn add_header(&mut self, header: Header) -> &mut Self {
        self.headers.push(header);

        self
    }

    pub fn set_body(&mut self, body: ResponseBody) -> &mut Self {
        self.body = body;

        self
    }

    pub fn set_status(&mut self, status: Status) -> &mut Self {
        self.status = status;

        self
    }

    pub fn set_content_type(&mut self, content_type: Mime) -> &mut Self {
        self.headers.push(Header::ContentType(content_type));

        self
    }

    /// Set the content encoding for the response.
    /// 
    /// Note: It will only set the encoding if the client 
    /// explicitly say it supports
    pub fn set_body_encoding(
        &mut self,
        encoding: Option<BodyEncoding>,
        level: Option<CompressionLevel>,
    ) -> &mut Self {
        self.encoding = (encoding, level);

        self
    }
}
