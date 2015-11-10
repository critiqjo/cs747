extern crate rand;

use std::fmt;
use rand::{Rand, thread_rng};
use rand::distributions::{IndependentSample, Range};

type Action = Action8;
const N_ACTIONS: usize = 8;
type QASlice = [f64; N_ACTIONS];

const GRID_W: usize = 10;
const GRID_H: usize = 7;
type Q = [[QASlice; GRID_H]; GRID_W];

#[derive(Clone, Copy)]
struct Point {
    x: usize,
    y: usize,
}

fn get_max<'a, T>(iter: T) -> (usize, &'a f64)
        where T : Iterator<Item=&'a f64> {
    iter.enumerate().fold(None, |acc, (i, q_sa)| {
        if let Some((m_i, max)) = acc {
            if q_sa > max {
                Some((i, q_sa))
            } else {
                Some((m_i, max))
            }
        } else if !q_sa.is_nan() {
            Some((i, q_sa))
        } else { None }
    }).unwrap()
}

fn main() {
    let alpha = 0.1;
    let eps = 0.1;

    let goal = Point { x: 7, y: 3 };

    fn q_pos<'a>(q: &'a Q, p: Point) -> &'a QASlice { &q[p.x][p.y] }

    let mut rng = thread_rng();
    let mut next_action_idx = |q: &Q, pos: Point| {
        let q_xy = q_pos(q, pos);
        if f64::rand(&mut rng) > eps {
            let (idx, _) = get_max(q_xy.iter());
            idx
        } else {
            Range::new(0, q_xy.len()).ind_sample(&mut rng)
        }
    };

    let mut q = [[[0.0; N_ACTIONS]; GRID_H]; GRID_W]; // q[x][y][a] is valid
    let mut episodes = 0;
    let mut t = 0;

    'e: loop {
        let (mut grid, mut pos) = GridIfx::new("127.0.0.1:8080");

        let mut prev_pos = pos;
        let mut prev_act_idx = next_action_idx(&q, pos);
        let mut act_idx;

        println!("episode ended, t = {}", t);

        loop {
            pos = grid.try_move(Action::from(prev_act_idx)); // position due to prev action
            let goal_reached = pos.x == goal.x && pos.y == goal.y;
            let r = if goal_reached { 1.0 } else { -1.0 };
            act_idx = next_action_idx(&q, pos); // select the next action
            q[prev_pos.x][prev_pos.y][prev_act_idx] += alpha * (r + q_pos(&q, pos)[act_idx] - q_pos(&q, prev_pos)[prev_act_idx]);
            if goal_reached {
                episodes += 1;
                continue 'e;
            }
            prev_pos = pos;
            prev_act_idx = act_idx;
            if t == 8000 { break 'e; } else { t += 1; }
        }
    }

    let visual_actions = ["->", "\\/", "<-", "/\\", "\\.", "./", "'\\", "/'"];
    for (x, q_x) in q.iter().enumerate() {
        for (y, q_xy) in q_x.iter().enumerate() {
            let (x, y) = (x, y);
            let (idx, _) = get_max(q_xy.iter());
            print!("{} ", if x == goal.x && y == goal.y { "()" }
                          else { visual_actions[idx] });
        }
        print!("\n");
    }
    println!("episodes: {}", episodes);
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

// Action4 {{{
#[derive(Clone, Copy)]
enum Action4 {
    Up,
    Right,
    Down,
    Left,
}

impl Action4 {
    fn to_char(self) -> char {
        match self {
            Action4::Up => 'u',
            Action4::Right => 'r',
            Action4::Down => 'd',
            Action4::Left => 'l',
        }
    }
}

impl From<usize> for Action4 {
    fn from(u: usize) -> Action4 {
        match u {
            0 => Action4::Up,
            1 => Action4::Right,
            2 => Action4::Down,
            3 => Action4::Left,
            _ => panic!("Invalid action index!"),
        }
    }
} // }}}

// Action8 {{{
#[derive(Clone, Copy)]
enum Action8 {
    Up,    UR,
    Right, DR,
    Down,  DL,
    Left,  UL,
}

impl Action8 {
    fn to_char(self) -> char {
        match self {
            Action8::Up => 'u',
            Action8::UR => '1',
            Action8::Right => 'r',
            Action8::DR => '2',
            Action8::Down => 'd',
            Action8::DL => '3',
            Action8::Left => 'l',
            Action8::UL => '4',
        }
    }
}

impl From<usize> for Action8 {
    fn from(u: usize) -> Action8 {
        match u {
            0 => Action8::Up,
            1 => Action8::Right,
            2 => Action8::Down,
            3 => Action8::Left,
            4 => Action8::UR,
            5 => Action8::DR,
            6 => Action8::DL,
            7 => Action8::UL,
            _ => panic!("Invalid action index!"),
        }
    }
} // }}}

// GridIfx {{{
use std::io::{BufRead, BufReader, Lines, Write};
use std::net::TcpStream;
use std::str::FromStr;

struct GridIfx {
    stream: TcpStream,
    lines: Lines<BufReader<TcpStream>>,
}

fn parse_pos(pos_str: String) -> Point {
    let pt_vec: Vec<_> = pos_str.split_whitespace()
                                .map(|s| usize::from_str(s).unwrap())
                                .collect();
    Point {
        x: pt_vec[0],
        y: pt_vec[1],
    }
}

impl GridIfx {
    fn new(address: &str) -> (GridIfx, Point) {
        let stream = match TcpStream::connect(address) {
            Ok(stream) => {
                let _ = stream.set_read_timeout(None);
                stream
            },
            Err(_) => panic!("Could not connect to the grid!"),
        };
        let mut grid_ifx = GridIfx {
            stream: stream.try_clone().unwrap(),
            lines: BufReader::new(stream).lines(),
        };
        if let Some(Ok(pos_str)) = grid_ifx.lines.next() {
            (grid_ifx, parse_pos(pos_str))
        } else {
            panic!("Bad initial reply from the grid!")
        }
    }

    fn try_move(&mut self, a: Action) -> Point {
        let _ = write!(&mut self.stream, "{}\r\n", a.to_char());
        if let Some(Ok(pos_str)) = self.lines.next() {
            parse_pos(pos_str)
        } else {
            panic!("Bad reply from the grid!")
        }
    }
}
// }}}
