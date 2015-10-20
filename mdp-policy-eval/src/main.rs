#![feature(slice_patterns)]

use std::io;
use std::str::FromStr;
use std::fmt::Debug;

fn read_line(stdin: &mut io::Stdin) -> String {
    let mut line = String::new();
    match stdin.read_line(&mut line) {
        Err(_) | Ok(0) => panic!("Read error"),
        _ => line,
    }
}

fn from_str<T, U>(ustr: U) -> T
    where T: FromStr, T::Err: Debug, U: AsRef<str>
{
    T::from_str(ustr.as_ref()).expect("Bad input")
}

#[derive(Debug)]
enum ActionEntry {
    Acted(usize, usize, f64),
    Stopped(usize),
}

fn main() {
    let mut stdin = io::stdin();

    let n: usize = from_str(read_line(&mut stdin).trim()); // # of states
    let k: usize = from_str(read_line(&mut stdin).trim()); // # of actions
    let g: f64 = from_str(read_line(&mut stdin).trim()); // discount factor

    let mut action_history = Vec::new();
    loop {
        let line = read_line(&mut stdin);
        let tupl: Vec<&str> = line.split_whitespace().collect();
        if let [s, a, r] = &tupl[..] {
            action_history.push(ActionEntry::Acted(from_str(s), from_str(a), from_str(r)));
        } else if let [s_final] = &tupl[..] {
            action_history.push(ActionEntry::Stopped(from_str(s_final)));
            break;
        }
    }

    println!("n: {}, k: {}, g: {}", n, k, g);
    println!("{:?}", action_history);
}
