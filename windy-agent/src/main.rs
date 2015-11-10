extern crate rand;

use std::{fmt,ops};
use rand::{Rand, thread_rng};
use rand::distributions::{IndependentSample, Range};

const N_ACTIONS: usize = 4;
type QASlice = [f64; N_ACTIONS];
type Q = Vec<Vec<QASlice>>;

#[derive(Clone, Copy)]
struct Point {
    x: isize,
    y: isize,
}

#[derive(Clone, Copy)]
enum Action {
    Up,
    Right,
    Down,
    Left,
}

fn geometric_pick(a1u: usize, a2u: usize, pos: Point, goal: Point) -> usize {
    if (pos + Action::from(a1u).to_offset() - goal).mag_sq() <
        (pos + Action::from(a2u).to_offset() - goal).mag_sq() { a1u }
    else { a2u }
}

fn get_max<'a, T>(iter: T, pos: Point, goal: Point) -> (usize, &'a f64)
        where T : Iterator<Item=&'a f64> {
    iter.enumerate().fold(None, |acc, (i, q_sa)| {
        if let Some((m_i, max)) = acc {
            if q_sa > max {
                Some((i, q_sa))
            } else if q_sa == max {
                if geometric_pick(i, m_i, pos, goal) == i {
                    Some((i, q_sa))
                } else {
                    Some((m_i, max))
                }
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

    fn q_pos<'a>(q: &'a Q, p: Point) -> &'a QASlice
    { &q[p.x as usize][p.y as usize] }

    fn q_pos_mut<'a>(q: &'a mut Q, p: Point) -> &'a mut QASlice
    { &mut q[p.x as usize][p.y as usize] }

    let mut rng = thread_rng();
    let mut next_action_idx = |q: &Q, pos: Point| {
        let q_xy = q_pos(q, pos);
        if f64::rand(&mut rng) > eps {
            let (idx, _) = get_max(q_xy.iter(), pos, goal);
            idx
        } else {
            Range::new(0, q_xy.len()).ind_sample(&mut rng)
        }
    };

    let (grid_w, grid_h) = (10, 7);

    let mut q = vec![vec![[0.0; 4]; grid_h]; grid_w]; // so that q[x][y][a] is valid
    let mut episodes = 0;
    let mut t = 0;

    'e: loop {
        let (mut grid, mut pos) = GridIfx::new("127.0.0.1:8080");

        let mut prev_pos = pos;
        let mut prev_act_idx = next_action_idx(&q, pos);
        let mut act_idx;

        println!("start: {}", t);

        loop {
            pos = grid.try_move(Action::from(prev_act_idx)); // position due to prev action
            let goal_reached = pos.x == goal.x && pos.y == goal.y;
            let r = if goal_reached { 1000.0 } else { -1.0 };
            act_idx = next_action_idx(&q, pos); // select the next action
            q_pos_mut(&mut q, prev_pos)[prev_act_idx] += alpha * (r + q_pos(&q, pos)[act_idx] - q_pos(&q, prev_pos)[prev_act_idx]);
            if goal_reached {
                episodes += 1;
                continue 'e;
            }
            prev_pos = pos;
            prev_act_idx = act_idx;
            if t == 8000 { break 'e; } else { t += 1; }
        }
    }

    let visual_actions = ['>', 'v', '<', '^'];
    for (x, q_x) in q.iter().enumerate() {
        for (y, q_xy) in q_x.iter().enumerate() {
            let (x, y) = (x as isize, y as isize);
            let (idx, _) = get_max(q_xy.iter(), Point { x: x, y: y }, goal);
            print!("{} ", if x == goal.x && y == goal.y { 'G' }
                          else { visual_actions[idx] });
        }
        print!("\n");
    }
    println!("episodes: {}", episodes);
}

impl Point {
    fn mag_sq(self) -> usize {
        (self.x * self.x + self.y * self.y) as usize
    }
}

impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

impl ops::Add for Point {
    type Output = Point;

    fn add(self, _rhs: Point) -> Point {
        Point { x: self.x + _rhs.x, y: self.y + _rhs.y }
    }
}

impl ops::Sub for Point {
    type Output = Point;

    fn sub(self, _rhs: Point) -> Point {
        Point { x: self.x - _rhs.x, y: self.y - _rhs.y }
    }
}

impl Action {
    fn to_char(self) -> char {
        match self {
            Action::Up => 'u',
            Action::Right => 'r',
            Action::Down => 'd',
            Action::Left => 'l',
        }
    }

    fn to_offset(self) -> Point {
        match self {
            Action::Up => Point { x: 0, y: 1 },
            Action::Right => Point { x: 1, y: 0 },
            Action::Down => Point { x: 0, y: -1 },
            Action::Left => Point { x: -1, y: 0 },
        }
    }
}

impl From<usize> for Action {
    fn from(u: usize) -> Action {
        match u {
            0 => Action::Up,
            1 => Action::Right,
            2 => Action::Down,
            3 => Action::Left,
            _ => panic!("Invalid action index!"),
        }
    }
}

use std::io::{BufRead, BufReader, Lines, Write};
use std::net::TcpStream;
use std::str::FromStr;

struct GridIfx {
    stream: TcpStream,
    lines: Lines<BufReader<TcpStream>>,
}

fn parse_pos(pos_str: String) -> Point {
    let pt_vec: Vec<_> = pos_str.split_whitespace()
                                .map(|s| isize::from_str(s).unwrap())
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
