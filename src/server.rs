use crate::http::{ParseError, Request, Response, StatusCode};
use std::convert::TryFrom;
use std::convert::TryInto;
use std::io::{Read, Write};
use std::net::TcpListener;

pub trait Handler {
    fn handle_request(&mut self, request: &Request) -> Response;

    fn handle_bad_request(&mut self, e: &ParseError) -> Response {
        println!("Failed to parse request: {}", e);
        Response::new(StatusCode::BadRequest, None)
    }
}

pub struct Server {
    addr: String,
}

impl Server {
    // In every struct, there is `Self` where `Self` + `Server` are interchangeable
    pub fn new(addr: String) -> Self {
        Server { addr }
    }

    // Method always has `self` as first param (aka: `this`)
    // -    Takes ownership of entire struct
    pub fn run(self, mut handler: impl Handler) {
        println!("Listening on {}", self.addr);

        // `.bind` returns a result and we decide how to handle it (either success || err)
        // -    e.g, fail to bind socket, stop server
        // -    `.unwrap()` will terminate program if err on a result
        let listener = TcpListener::bind(&self.addr).unwrap();

        // loop == infinite loop
        // check for new connections
        loop {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    let mut buffer = [0; 1024];

                    match stream.read(&mut buffer) {
                        Ok(_) => {
                            // `_lossy` never fails, but replaces with `?` symbol
                            println!("Received a request: {}", String::from_utf8_lossy(&buffer));

                            // Match on result from request
                            let response = match Request::try_from(&buffer[..]) {
                                Ok(request) => handler.handle_request(&request),
                                Err(e) => handler.handle_bad_request(&e),
                            };

                            // Send response to TcpStream
                            if let Err(e) = response.send(&mut stream) {
                                println!("Failed to send response: {}", e)
                            }
                        }
                        Err(e) => println!("Failed to read from connection: {}", e),
                    }

                    println!("OK")
                }
                Err(e) => println!("Failed to establish a connection: {}", e),
            }
        }
    }
}
