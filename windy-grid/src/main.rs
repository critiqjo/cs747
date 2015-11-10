extern crate coio;
extern crate rustc_serialize;

use std::io::{stdin, Read, Write};
use std::sync::Arc;
use coio::net::TcpListener;
use rustc_serialize::json;

/// When an agent connects, its initial position will be sent back.
/// The bottom-leftmost cell is the (0, 0) cell.
///
/// The agent may now start sending a sequence of actions.
/// An action is represented by a single ASCII character (case insensitive),
/// `u`, `r`, `d`, and `l`, for up, right, down, and left;
/// and if kings moves allowed,
/// `1`, `2`, `3`, and `4`, for UR, DR, DL, and UL (think quadrants).
/// Anything else is ignored, and an `"err"` is sent back.
///
/// After each move, a reply of the form: `"x y"` is sent back,
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
    start_pos: (isize, isize),
    /// vertical wind strengths (positive => upwards)
    winds: Vec<i8>,
}

fn main() {
    let mut stdin = stdin();
    let mut json_str = String::new();
    let _ = stdin.read_to_string(&mut json_str).unwrap();
    let grid_conf: GridConf = json::decode(&json_str).unwrap();

    if grid_conf.winds.len() != grid_conf.width {
        panic!("Length of winds array and grid width must match.");
    }

    let grid_conf = Arc::new(grid_conf);

    coio::Scheduler::new().run(move || {
        let listener = TcpListener::bind(&grid_conf.listen_addr as &str);
        let listener = match listener {
            Ok(l) => l,
            Err(e) => panic!("Error on binding: {}", e),
        };

        println!("Waiting for agents...");

        for stream in listener.incoming() {
            let (mut stream, addr) = stream.unwrap();
            println!("Agent from {} arrived!", addr);

            let grid_conf = grid_conf.clone();
            coio::spawn(move || {
                let mut pos = grid_conf.start_pos;

                let _ = write!(&mut stream, "{} {}\r\n", pos.0, pos.1);
                loop {
                    let mut bytes = [0u8; 4];
                    let line = match stream.read(&mut bytes) {
                        Ok(n) if n > 0 => {
                            let (s, _) = bytes.split_at(n);
                            String::from_utf8(s.to_vec())
                        },
                        _ => break,
                    };

                    let mut invalid_move = false;
                    let wind = grid_conf.winds[pos.0 as usize];
                    if let Ok(line) = line {
                        match line.trim() {
                            "u" | "U" => pos.1 += 1,
                            "r" | "R" => pos.0 += 1,
                            "d" | "D" => pos.1 -= 1,
                            "l" | "L" => pos.0 -= 1,
                            "1" => { pos.0 += 1; pos.1 += 1 },
                            "2" => { pos.0 += 1; pos.1 -= 1 },
                            "3" => { pos.0 -= 1; pos.1 -= 1 },
                            "4" => { pos.0 -= 1; pos.1 += 1 },
                            _ => invalid_move = true,
                        }
                    } else { break }

                    let _ = if invalid_move {
                        continue
                    } else {
                        fn bound(var: &mut isize, min: usize, max: usize) {
                            if *var < min as isize { *var = min as isize }
                            if *var > max as isize { *var = max as isize }
                        }
                        pos.1 += wind as isize;
                        bound(&mut pos.0, 0, grid_conf.width - 1);
                        bound(&mut pos.1, 0, grid_conf.height - 1);
                        write!(&mut stream, "{} {}\r\n", pos.0, pos.1)
                    };
                }
                println!("Agent from {} left!", addr);
            });
        }
    }).unwrap();
}
