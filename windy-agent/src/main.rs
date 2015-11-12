extern crate rand;

use rand::{Rand, thread_rng};

const STRATEGY: Strategy = Strategy::DiscourageIdling;
const HORIZON: usize = 16000;

type Action = Action8;
const N_ACTIONS: usize = 8;
type QASlice = [f64; N_ACTIONS];
const GRID_W: usize = 10;
const GRID_H: usize = 7;
type Q = [[QASlice; GRID_H]; GRID_W];

#[allow(dead_code)]
enum Strategy {
    Simple,
    UnitGoalReward,
    HugeGoalReward,
    DiscourageIdling,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct Point {
    x: usize,
    y: usize,
}

fn get_max<'a, T>(iter: T) -> (Vec<usize>, f64)
        where T : Iterator<Item=&'a f64> {
    let mut max_ids = vec![];
    let mut max_val = None;
    for (i, v) in iter.enumerate() {
        if let Some(m_v) = max_val {
            if *v > m_v {
                max_ids.truncate(1);
                max_ids[0] = i;
                max_val = Some(*v);
            } else if *v == m_v {
                max_ids.push(i);
            }
        } else if !v.is_nan() {
            max_ids.push(i);
            max_val = Some(*v);
        }
    }
    (max_ids, max_val.unwrap())
}

fn main() {
    let alpha = 0.1;
    let eps = 0.1;

    let goal = Point { x: 7, y: 3 };

    // get "Q action-value list" of a point
    fn q_pos<'a>(q: &'a Q, p: Point) -> &'a QASlice { &q[p.x][p.y] }

    let mut rng = thread_rng();
    let mut next_action_idx = |q: &Q, pos: Point| {
        let q_xy = q_pos(q, pos);
        if f64::rand(&mut rng) > eps {
            let (ids, _) = get_max(q_xy.iter());
            ids[usize::rand(&mut rng) % ids.len()]
        } else {
            usize::rand(&mut rng) % q_xy.len()
        }
    };

    let mut q = [[[0.0; N_ACTIONS]; GRID_H]; GRID_W]; // q[x][y][a] is valid
    let mut t = 0;

    'e: loop {
        let (mut grid, mut pos) = GridIfx::new("127.0.0.1:8080");

        let mut prev_pos = pos;
        let mut prev_act_idx = next_action_idx(&q, pos);
        let mut act_idx;

        println!("episode start; time step {}", t);

        loop {
            pos = grid.try_move(Action::from(prev_act_idx)); // position due to prev action
            let goal_reached = pos == goal;
            let r = match STRATEGY {
                Strategy::Simple => if goal_reached { 0.0 } else { -1.0 },
                Strategy::UnitGoalReward => if goal_reached { 1.0 } else { -1.0 },
                Strategy::HugeGoalReward => if goal_reached { 1000.0 } else { -1.0 },
                Strategy::DiscourageIdling => if pos == prev_pos { -2.0 } else if goal_reached { 1.0 } else { -1.0 },
            };
            act_idx = next_action_idx(&q, pos); // select the next action
            q[prev_pos.x][prev_pos.y][prev_act_idx] += alpha * (r + q_pos(&q, pos)[act_idx] - q_pos(&q, prev_pos)[prev_act_idx]);
            if t == HORIZON { break 'e; } else { t += 1; }
            if goal_reached { continue 'e; }
            prev_pos = pos;
            prev_act_idx = act_idx;
        }
    }

    return; // comment this out to visualize optimal actions

    // print the "optimal" path found
    let visual_actions = ["->", "\\/", "<-", "/\\", "\\.", "./", "'\\", "/'"];
    for (x, q_x) in q.iter().enumerate() {
        for (y, q_xy) in q_x.iter().enumerate() {
            let (x, y) = (x, y);
            let (ids, _) = get_max(q_xy.iter());
            print!("{} ", if x == goal.x && y == goal.y { "()" }
                          else { visual_actions[ids[0]] });
        }
        print!("\n");
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
    #[allow(dead_code)]
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
    #[allow(dead_code)]
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
