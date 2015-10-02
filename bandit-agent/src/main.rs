#![feature(convert)]

// The crates and modules {{{
use std::process::exit;
use std::str::FromStr;

extern crate clap;
use clap::{App, Arg};

mod server_ifx;
use server_ifx::ServerIfx;
// -- }}}

// The helper functions {{{
fn err_exit(message: &str) -> ! {
    println!("error: {}", message);
    println!("See --help for usage information.");
    exit(1)
}

fn make_arg<'a, 'g, 'p, 'r>(name: &'a str, long: &'a str, help: &'a str)
        -> Arg<'a, 'a, 'a, 'g, 'p, 'r> {
    Arg::with_name(name).long(long)
                        .required(true)
                        .takes_value(true)
                        .help(help)
}
// -- }}}

// The Arm {{{
#[derive(Clone, Debug)]
struct Arm {
    cum_reward: usize,
    pull_count: usize,
    mean: f64,
}

impl Arm {
    fn pulled_once(reward: u8) -> Arm {
        Arm {
            cum_reward: reward as usize,
            pull_count: 1,
            mean: reward as f64,
        }
    }

    fn pulled(&mut self, reward: u8) {
        self.cum_reward += reward as usize;
        self.pull_count += 1;
        self.mean = self.cum_reward as f64 / self.pull_count as f64;
    }

    // tolerance (< 0): log of acceptable error
    fn get_ucb(&self, error: f64) -> f64 {
        self.mean + (-error.ln() / 2.0 / self.pull_count as f64).sqrt()
    }
}
// -- }}}

fn main() {
    // The parsing of arguments {{{
    let arg_matches = App::new("agent").version("v0.0.1")
                          .arg(make_arg("address", "server",
                               "Server address given as host:port"))
                          .arg(make_arg("num-arms", "arms",
                               "Number of arms in the bandit"))
                          .arg(make_arg("horizon", "horizon",
                               "Time horizon"))
                          .get_matches();

    let address = arg_matches.value_of("address").unwrap();
    let num_arms = usize::from_str(arg_matches.value_of("num-arms").unwrap())
                       .unwrap_or_else(|_| err_exit("Invalid number of arms!"));
    let horizon = usize::from_str(arg_matches.value_of("horizon").unwrap())
                      .unwrap_or_else(|_| err_exit("Invalid horizon!"));
    // -- args parse }}}

    let mut server = ServerIfx::new(&address);
    let mut arms = Vec::new();

    // The intelligent loop {{{
    for arm_id in 0..std::cmp::min(num_arms, horizon) {
        let reward = server.pull_arm(arm_id);
        arms.push(Arm::pulled_once(reward));
    }

    let mut active_arms: Vec<_> = (0..num_arms).collect();
    for time_step in num_arms..horizon {
        let error = 1.0 / time_step as f64;
        let arm2pull = active_arms.iter()
            .fold(None, |id_ucb_pair, &arm_id| {
                let arm_ucb = arms[arm_id].get_ucb(error);
                if let Some((arm2pull, max_ucb)) = id_ucb_pair {
                    if arm_ucb > max_ucb {
                        Some((arm_id, arm_ucb))
                    } else {
                        Some((arm2pull, max_ucb))
                    }
                } else {
                    Some((arm_id, arm_ucb))
                }
            }).unwrap().0;
        let reward = server.pull_arm(arm2pull);
        arms[arm2pull].pulled(reward);

        if active_arms.len() == 1 { continue; }

        let max_pull_count = active_arms.iter()
            .fold(0, |max, &arm_id| {
                std::cmp::max(max, arms[arm_id].pull_count)
            }) as f64;
        let threshold = max_pull_count * (1. - 1./(time_step as f64).ln())
                                       / active_arms.len() as f64;
        if (arms[arm2pull].pull_count as f64) < threshold {
            if let Ok(index) = active_arms.binary_search(&arm2pull) {
                active_arms.remove(index);
            }
        }
    }
    // -- intelli loop }}}

    // The final output! {{{
    let mut total_reward = 0;
    for arm_id in 0..num_arms {
        let arm = &arms[arm_id];
        println!("Arm {} has mean reward {:.4}, having pulled {:>5$}{} times ({:.1}%)",
                 arm_id, arm.mean, arm.pull_count,
                 if let Ok(_) = active_arms.binary_search(&arm_id) { "*" } else { " " },
                 100. * arm.pull_count as f64 / horizon as f64,
                 (horizon as f64).log10() as usize + 1);
        total_reward += arm.cum_reward;
    }
    println!("Total reward: {}", total_reward)
    // -- }}}
}
