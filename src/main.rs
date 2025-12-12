
// ============================================================
//  DAEGONICA SOFTWARE — main.rs
//  Part of the Daegonica Software Rust Ecosystem
// ============================================================

//! # Daegonica Module: Server Main
//!
//! **Purpose:**
//! Entry point for the Daegonica experimental server. Sets up TCP listener and thread pool, and handles incoming HTTP requests.
//!
//! **Context:**
//! - Used as the main executable for the server project.
//!
//! **Responsibilities:**
//! - Accepts incoming TCP connections.
//! - Dispatches requests to worker threads.
//! - Handles basic HTTP GET requests and serves HTML files.
//! - Does NOT handle advanced routing, security, or persistent state.
//!
//! **Author:** Daegonica Software
//! **Version:** 0.1.0
//! **Last Updated:** 2025-12-04
//!
//! ---------------------------------------------------------------
//! This file is part of the Daegonica Software codebase.
//! ---------------------------------------------------------------

use std::{
    fs,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    thread,
    time::Duration,
};

use server::ThreadPool;

/// # main
///
/// **Purpose:**
/// Starts the TCP server, initializes the thread pool, and dispatches incoming connections to worker threads.
///
/// **Parameters:**
/// None.
///
/// **Returns:**
/// None.
///
/// **Errors / Failures:**
/// - Panics if the TCP listener cannot be bound or a stream cannot be unwrapped.
///
/// **Examples:**
/// ```rust
/// // Run with `cargo run` to start the server.
/// main();
/// ```
fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4);
    for stream in listener.incoming().take(2) {
        let stream = stream.unwrap();
        pool.execute(|| {
            handle_connection(stream);
        });
    }
}

/// # handle_connection
///
/// **Purpose:**
/// Processes a single TCP stream, parses the HTTP request, and sends an appropriate HTML response.
///
/// **Parameters:**
/// - `stream`: TCP stream representing the client connection.
///
/// **Returns:**
/// None.
///
/// **Errors / Failures:**
/// - Panics if reading from the stream or writing the response fails.
///
/// **Examples:**
/// ```rust
/// handle_connection(stream);
/// ```
fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();
    let (status_line, filename) = match &request_line[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "html/hello.html"),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "html/hello.html")
        },
        _ => ("HTTP/1.1 404 WHAT THE HELL ARE YOU DOING HERE?!?!", "html/404.html"),
    };
    let contents = fs::read_to_string(filename).unwrap();
    let length = contents.len();
    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
    stream.write_all(response.as_bytes()).unwrap();
}