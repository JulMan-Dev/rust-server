# rust-server

🗺 A HTTP server written in Rust

> This is not librairy available on [crates.io](https://crates.io)
> 
> It's just a simple hackable project

## Features

- :white_check_mark: Body encoding support
- :white_check_mark: Cookies parsing
- :white_check_mark: Query arguments parsing
- :white_check_mark: Simple API
- :black_square_button: SSL/TLS (HTTPS) support
- :black_square_button: Stream support
- :black_square_button: Websocket support

## Example 

```rust
fn handle_request(request: &mut Request) -> Result<usize, IoError> {
    let mut response = Response::empty();

    response
        .set_status(Status::Ok)
        .add_header(Header::ContentType(Mime::text("plain")))
        .add_header(Header::Server("Custom".to_string()))
        .set_body(ResponseBody::Text("Hello, World".to_string()))
        // Encoding only if "Accept-Encoding" includes it
        .set_body_encoding(
            Some(BodyEncoding::Brotli), 
            None // Optional compression level
        );

    return request.respond(response);
}
```
