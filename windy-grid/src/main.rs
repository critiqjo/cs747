extern crate coio;
extern crate rustc_serialize;

use std::io::{stdin, Read, Write, BufReader, BufRead};
use coio::net::TcpListener;
use rustc_serialize::json;
use std::ops::Deref;

/// When an agent connects, its initial position will be sent back.
/// The bottom-leftmost cell is the (0, 0) cell.
///
/// The agent may now start sending a sequence of actions.
/// An action is represented by a single ASCII character (case insensitive),
/// `u`, `r`, `d`, and `l`, for up, right, down, and left;
/// and if kings moves allowed,
/// `1`, `2`, `3`, and `4`, for UR, DR, DL, and UL (think quadrants).
/// Any other character is simply ignored.
///
/// After each move, a reply of the form: `"x y\r\n"` is sent back,
/// indicating the agent's position after the move.

#[derive(RustcDecodable, RustcEncodable)]
struct GridConf {
    /// of the form `host:port`
    listen_addr: String,
    /// width of the grid
    width: usize,
    /// height of the grid
    height: usize,
    /// starting position of agent
    start_pos: (usize, usize),
}

fn main() {
    let mut stdin = stdin();
    let mut json_str = String::new();
    let _ = stdin.read_to_string(&mut json_str).unwrap();
    let grid_conf: GridConf = json::decode(&json_str).unwrap();
    coio::Scheduler::new().run(move || {
        let listener = TcpListener::bind(grid_conf.listen_addr.deref()).unwrap();
        println!("Waiting for connection ...");

        for stream in listener.incoming() {
            let (mut stream, addr) = stream.unwrap();
            println!("Got connection from {:?}", addr);

            let start_pos = grid_conf.start_pos.clone();
            coio::spawn(move || {
                let mut pos = start_pos;
                let reader = BufReader::new(stream.try_clone().unwrap());
                let mut write = |p: &(usize, usize)|
                    write!(&mut stream, "{} {}\r\n", p.0, p.1);

                let _ = write(&pos);
                for line in reader.lines() {
                    if let Ok(line) = line {
                        match line.trim() {
                            "u" | "U" => pos.1 += 1,
                            "r" | "R" => pos.0 += 1,
                            "d" | "D" => pos.1 -= 1,
                            "l" | "L" => pos.0 -= 1,
                            _ => (),
                        }
                    } else { break }

                    let _ = write(&pos);
                }
                println!("Client closed");
            });
        }
    }).unwrap();
}
