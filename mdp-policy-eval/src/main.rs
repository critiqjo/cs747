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
    let _: usize = from_str(read_line(&mut stdin).trim()); // # of actions
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

    // TD(1) {{{
    #[derive(Clone, Copy)]
    struct StateStat {
        vs_sum: f64,
        factor: f64,
        count: usize,
    }

    let mut state_stats = vec![StateStat { vs_sum: 0.0, factor: 0.0, count: 0 }; n];

    let mut action_iter = action_history.iter();
    while let Some(&ActionEntry::Acted(s, _, r)) = action_iter.next() {
        for (s_, state_stat) in state_stats.iter_mut().enumerate() {
            if s == s_ {
                state_stat.factor += 1.0;
                state_stat.count += 1;
            }
            state_stat.vs_sum += state_stat.factor * r;
            state_stat.factor *= g;
        }
    }
    let v1: Vec<f64> = state_stats.iter()
                           .map(| &StateStat { vs_sum, factor: _, count } |
                                { vs_sum / count as f64 })
                           .collect();
    // }}}

    // Batch TD(0) {{{
    let mut v2: Vec<f64> = vec![0.0; n];
    for i in 1..256 {
        let mut alpha = 0.25 / i as f64;
        if i > 16 { alpha /= (i as f64).sqrt(); }
        for (cur, next) in action_history.iter().zip(action_history.iter().skip(1)) {
            if let &ActionEntry::Acted(s, _, r) = cur {
                let s_ = match next {
                    &ActionEntry::Acted(s, _, _) => s,
                    &ActionEntry::Stopped(s) => s,
                };
                v2[s] = v2[s] + alpha * (r + g * v2[s_] - v2[s])
            } else { panic!("Invalid action array!"); }
        }
    }
    // }}}

    println!(" {:11}| batch TD(0)", "TD(1)");
    for (v_s, v_s2) in v1.iter().zip(v2.iter()) {
        println!("{:11.6} |{:11.6}", v_s, v_s2);
    }
}
