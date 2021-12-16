/*
 * This file is part of rust-book-multithreaded-web-server.
 * Copyright (c) 2021-2021 Joe Ma <rikkaneko23@gmail.com>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU Lesser General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

use lazy_static::lazy_static;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::fs;
use std::thread;
use std::time::{Duration, Instant};
use regex::Regex;
use rust_book_multithreaded_web_server::ThreadPool;
use chrono::{DateTime, Local};

lazy_static! {
	static ref RESPONSE_TEXT_HOME: String = fs::read_to_string("hello_from_rs.html").unwrap();
	static ref RESPONSE_TEXT_403: String = fs::read_to_string("403.html").unwrap();
	static ref RESPONSE_TEXT: String = fs::read_to_string("reply_page.html").unwrap();
	static ref RE_REQUEST: Regex = Regex::new(r#"(GET|POST|PUT|DELETE) (/.*?)/* HTTP/1\.1"#).unwrap();
	static ref RE_SLASH: Regex = Regex::new(r#"/{2,}"#).unwrap();
}

fn handle_stream(mut stream: TcpStream) {
	let mut buf = [0; 1024];
	stream.read(&mut buf).unwrap();
	let request = String::from_utf8_lossy(&buf);
	let (method, uri) = parse_header(&request);
	let _uri = RE_SLASH.replace_all(uri, "/");
	let uri = _uri.as_ref();
	if method.is_empty() || uri.is_empty() { return }
	
	println!("Request from {}: {} {}", stream.peer_addr().unwrap(), method, uri);
	let response: String;
	if method == "GET" {
		match uri {
			"/" | "/index.html" => response = format!(
					"HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
					RESPONSE_TEXT_HOME.len(), RESPONSE_TEXT_HOME.as_str()),
			
			"/auth" => response = format!(
					"HTTP/1.1 403 Forbidden\r\nContent-Length: {}\r\n\r\n{}",
					RESPONSE_TEXT_403.len(), RESPONSE_TEXT_403.as_str()),
			
			"/now" => {
				let time: String = Local::now().to_rfc3339();
				response = format!(
					"HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}\n",
					time.len(), time);
			}
			
			_ => {
				let content = RESPONSE_TEXT.replace("##METHOD##", method).replace("##URI##", uri);
				response = format!(
					"HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
					content.len(), content.as_str());
			}
		}
	} else if method == "PUT" || method == "POST" {
		match uri {
			"/make_cat" | "/make_dog" => response = format!(
					"HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
					8, "Success\n"),
			
			"/sleep" => {
				thread::sleep(Duration::from_millis(5000));
				response = format!(
					"HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n{}",
					8, "Success\n");
			},
			
			_ => response = format!(
					"HTTP/1.1 403 Forbidden\r\nContent-Length: {}\r\n\r\n{}",
					14, "Access Denied\n")
		}
	} else {
		response = String::from("HTTP/1.1 405 Method Not Allowed\r\n\r\n");
	}
	
	stream.write(response.as_bytes()).unwrap();
	stream.flush().unwrap();
}

fn parse_header(text: &str) -> (&str, &str) {
	let caps = RE_REQUEST.captures(text);
	if let Some(caps) = caps {
		if caps.len() >= 3 {
			return (caps.get(1).unwrap().as_str(), caps.get(2).unwrap().as_str())
		}
	}
	("", "")
}

fn main() {
	let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
	let thread_pool = ThreadPool::new(8);
	for stream in listener.incoming() {
		let stream = stream.unwrap();
		// handle_stream(stream);
		thread_pool.execute(||{
			handle_stream(stream);
		});
	}
}
