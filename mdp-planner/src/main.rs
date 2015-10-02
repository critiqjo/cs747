use std::io;
use std::str::FromStr;
use std::fmt::Debug;

type Vec3D<T> = Vec<Vec<Vec<T>>>;

fn read_line(stdin: &mut io::Stdin) -> String {
    let mut line = String::new();
    match stdin.read_line(&mut line) {
        Err(_) | Ok(0) => panic!("Read error"),
        _ => { line.pop(); line },
    }
}

fn from_str<T, U>(ustr: U) -> T
    where T: FromStr, T::Err: Debug, U: AsRef<str>
{
    T::from_str(ustr.as_ref()).expect("Bad input")
}

fn main() {
    let mut stdin = io::stdin();

    let n: usize = from_str(read_line(&mut stdin)); // # of states
    let k: usize = from_str(read_line(&mut stdin)); // # of actions

    let (r, t) = { // reward and transition functions
        let mut read_f64_3d = |x, y, z| -> Vec3D<f64> {
            let mut v = Vec::with_capacity(x);
            for _ in 0..x {
                let mut v_i = Vec::with_capacity(y);
                for _ in 0..y {
                    let line = read_line(&mut stdin);
                    let v_ij = line.split_whitespace()
                                   .map(|s| from_str::<f64, _>(s))
                                   .collect::<Vec<_>>();
                    if v_ij.len() != z {
                        panic!("Invalid input format!");
                    }
                    v_i.push(v_ij);
                }
                v.push(v_i);
            }
            v
        };
        (read_f64_3d(n, k, n),
         read_f64_3d(n, k, n))
    };

    let g: f64 = from_str(read_line(&mut stdin)); // discount factor

    let q_s_calc = |v: &Vec<f64>, s: usize| -> Vec<f64> {
        r[s].iter()
            .zip(t[s].iter())
            .map(|(r_sa, t_sa)| {
                r_sa.iter().zip(t_sa.iter())
                    .enumerate()
                    .fold(0.0, |acc, (s_, (r_sas_, t_sas_))| {
                        acc + t_sas_ * (r_sas_ + g * v[s_])
                    })
                })
            .collect()
    };

    // value iteration
    let theta = 1e-6;
    let mut v = vec![0f64; n];
    loop {
        let mut delta = 0.0;
        for s in 0..n {
            let v_s = v[s];
            v[s] = q_s_calc(&v, s).into_iter()
                   .fold(None, |acc, q_sa| {
                       if let Some(max) = acc {
                           if max > q_sa {
                               Some(max)
                           } else {
                               Some(q_sa)
                           }
                       } else {
                           Some(q_sa)
                       }
                   }).unwrap();
            let diff = (v[s] - v_s).abs();
            delta = if delta > diff { delta } else { diff };
        }
        if delta < theta { break; }
    }

    let mut pi = Vec::with_capacity(n);
    for s in 0..n {
        pi.push(q_s_calc(&v, s).into_iter()
                .enumerate()
                .fold(None, |acc, (a, q_sa)| {
                    if let Some((a_max, q_sa_max)) = acc {
                        if q_sa > q_sa_max {
                            Some((a, q_sa))
                        } else {
                            Some((a_max, q_sa_max))
                        }
                    } else {
                        Some((a, q_sa))
                    }
                }).unwrap());
    }
    for (a, v_s) in pi {
        println!("{:.5} {}", v_s, a);
    }
}
