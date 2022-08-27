pub mod accept;
pub mod common;
pub mod cookie;
pub mod mime;
pub mod request;
pub mod response;
pub mod search;
pub mod server;

use common::{Cache, Header, Status};
use mime::Mime;
use request::Request;
use response::{BodyEncoding, Response, ResponseBody};
use server::{Server, ServerOptions};
use std::env::args;
use std::io::Error as IoError;
use std::thread::sleep;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = args().collect();

    if args.len() != 2 {
        println!("Usage: {} <port>", args[0]);
        return Ok(());
    }

    let mut port: u16 = match args[1].parse() {
        Ok(port) => port,
        Err(_) => {
            println!("Invalid port: {}", args[1]);
            return Ok(());
        }
    };
    let mut tries = 1;

    let server: Server = loop {
        match Server::bind_v4(port, Some(ServerOptions { log: true })) {
            Ok(listener) => break listener,
            Err(e) => {
                println!("Failed to bind to port {}: {:?} ({}/{})", port, e, tries, 5);
                tries += 1;

                if tries > 5 {
                    port += 1;
                    tries = 1;
                    continue;
                } else {
                    sleep(Duration::from_secs(1));
                }
            }
        }
    };
    println!("Listening on port {}", port);

    for request in server.requests() {
        if let None = request {
            continue;
        }

        if let Some(mut request) = request {
            if let Err(err) = handle_request(&mut request) {
                println!("Error: {}", err);
            }
        }
    }

    return Ok(());
}

fn handle_request(request: &mut Request) -> Result<usize, IoError> {
    let mut response = Response::empty();

    response
        .set_status(Status::Ok)
        .add_header(Header::ContentType(Mime::text("plain")))
        .add_header(Header::Server("JulMan-Http/1.0".to_string()))
        .add_header(Header::CacheControl(vec![Cache::NoStore]))
        .set_body(ResponseBody::Text(format!("{:#?}", request)))
        .set_body_encoding(Some(BodyEncoding::Brotli), None);

    return request.respond(response);
}
