use crate::request::{handle_connection, Request};
use chrono::offset::Local;
use std::io::{ErrorKind, Result as IoResult};
use std::net::TcpListener;

#[derive(Debug)]
pub enum BindError {
    PortAlreadyInUse,
    PermissionDenied,
    Unknown(ErrorKind),
}

#[derive(Debug)]
pub struct Server {
    pub port: u16,
    pub listener: TcpListener,
    pub options: ServerOptions,
}

#[derive(Debug)]
pub struct ServerOptions {
    pub log: bool,
}

impl Server {
    pub fn bind_v4(port: u16, options: Option<ServerOptions>) -> Result<Server, BindError> {
        let options = options.unwrap_or(ServerOptions { log: false });

        match TcpListener::bind(format!("0.0.0.0:{}", port)) {
            Ok(listener) => Ok(Server {
                port,
                listener,
                options,
            }),
            Err(e) => Err(match e.kind() {
                ErrorKind::AddrInUse => BindError::PortAlreadyInUse,
                ErrorKind::PermissionDenied => BindError::PermissionDenied,
                kind => BindError::Unknown(kind),
            }),
        }
    }

    pub fn bind_v6(port: u16, options: Option<ServerOptions>) -> Result<Server, BindError> {
        let options = options.unwrap_or(ServerOptions { log: false });

        match TcpListener::bind(format!("[::]:{}", port)) {
            Ok(listener) => Ok(Server {
                port,
                listener,
                options,
            }),
            Err(e) => Err(match e.kind() {
                ErrorKind::AddrInUse => BindError::PortAlreadyInUse,
                ErrorKind::PermissionDenied => BindError::PermissionDenied,
                kind => BindError::Unknown(kind),
            }),
        }
    }

    pub fn next(&self) -> IoResult<Request> {
        match self.listener.accept() {
            Ok((stream, _)) => {
                let req = handle_connection(stream);

                if let Ok(ref req) = req {
                    if self.options.log {
                        println!(
                            "[{:?}] {} {} {}",
                            Local::now(),
                            req.method.to_string(),
                            req.uri.to_string(),
                            req.stream.peer_addr()?
                        );
                    }
                }
                return req;
            }
            Err(e) => {
                println!("Failed to accept connection: {:?}", e);

                return Err(e);
            }
        };
    }

    pub fn requests(&self) -> Requests {
        Requests { server: self }
    }
}

pub struct Requests<'a> {
    pub server: &'a Server,
}

impl<'a> Iterator for Requests<'a> {
    type Item = Option<Request>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.server.next() {
            Ok(req) => Some(Some(req)),
            Err(_) => Some(None),
        }
    }
}
