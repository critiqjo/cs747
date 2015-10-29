extern crate coio;

use std::io::{Read, Write};
use coio::net::TcpListener;

fn main() {
    coio::Scheduler::new().run(move || {
        let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
        println!("Waiting for connection ...");

        for stream in listener.incoming() {
            let (mut stream, addr) = stream.unwrap();

            println!("Got connection from {:?}", addr);

            coio::spawn(move|| {
                if let Ok(rstream) = stream.try_clone() {
                    for b_res in rstream.bytes() {
                        match b_res.unwrap() {
                            b'u' | b'U' => stream.write(b"up\r\n"),
                            b'r' | b'R' => stream.write(b"right\r\n"),
                            b'd' | b'D' => stream.write(b"down\r\n"),
                            b'l' | b'L' => stream.write(b"left\r\n"),
                            _ => stream.write(b"ignore\r\n"),
                        }.unwrap();
                    }
                } else { println!("Could not clone()"); }

                println!("Client closed");
            });
        }
    }).unwrap();
}
