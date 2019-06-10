#[macro_use(lazy_static)]
extern crate lazy_static;

use std::env;
use std::fs;

mod ltl;
use ltl::*;

mod rules {

use chrono::DateTime;
use chrono::Duration;
use chrono::FixedOffset;
use regex::Regex;

use crate::ltl::*;

fn parse_time(text: &str) -> Option<DateTime<FixedOffset>> {
    lazy_static! {
        static ref RE: Regex =
            Regex::new(r" node-([0-9]+): \|([-0-9]+ [:0-9]+\.[0-9]+) UTC\|").unwrap();
    }
    let caps = RE.captures(text)?;
    DateTime::parse_from_str(&caps[1], "%F %T.%q").ok()
}

fn transactions_always_handled(text: &str) -> Formula<&str> {
    lazy_static! {
        static ref RE: Regex =
            Regex::new("node-0:.*?node/restapi: submitted tx: ([0-9a-f]{64})").unwrap();
    }
    if let Some(begin) = parse_time(text) {
        if let Some(caps) = RE.captures(text) {
            fn look_for_handled(
                begin: DateTime<FixedOffset>,
                tx: String,
            ) -> Box<dyn Fn(&str) -> Formula<&str>> {
                let inner: Regex = Regex::new(
                    &("node-0:.*?consensus: finalized round [0-9]+, .*\\[.*".to_string()
                        + &tx
                        + ".*\\]"),
                )
                .unwrap();
                Box::new(move |text: &str| {
                    if let Some(now) = parse_time(text) {
                        if now.signed_duration_since(begin) > Duration::seconds(30) {
                            bottom("Failed to finalize transaction".to_string())
                        } else {
                            if inner.is_match(text) {
                                top()
                            } else {
                                accept(look_for_handled(begin, tx.to_string()))
                            }
                        }
                    } else {
                        top()
                    }
                })
            }
            let tx = &caps[0];
            accept(look_for_handled(begin, tx.to_string()))
        } else {
            top()
        }
    } else {
        top()
    }
}

fn transactions_always_valid(text: &str) -> Formula<&str> {
    lazy_static! {
        static ref RE: Regex = Regex::new("tx not valid for any round").unwrap();
    }
    if RE.is_match(text) {
        bottom("encountered \"tx not valid for any round\"".to_string())
    } else {
        top()
    }
}

fn confirm_round_finalization(text: &str) -> Formula<&str> {
    lazy_static! {
        static ref RE: Regex =
            Regex::new("node-([0-9]+):.*?consensus: starting new round: ([0-9]+),").unwrap();
    }
    if let Some(begin) = parse_time(text) {
        if let Some(caps) = RE.captures(text) {
            fn look_for_finalized(
                begin: DateTime<FixedOffset>,
                node: String,
                rnd: String,
            ) -> Box<dyn Fn(&str) -> Formula<&str>> {
                let inner: Regex = Regex::new(
                    &("node-".to_string() + &node + ":.*?consensus: finalized round ([0-9]+),"),
                )
                .unwrap();
                Box::new(move |text: &str| {
                    if let Some(now) = parse_time(text) {
                        if now.signed_duration_since(begin) > Duration::seconds(60) {
                            bottom("Failed to complete round".to_string())
                        } else {
                            if let Some(caps) = inner.captures(text) {
                                let rnd2 = (&caps[0]).to_string();
                                if rnd <= rnd2 {
                                    top()
                                } else {
                                    bottom("truth".to_string())
                                }
                            } else {
                                accept(look_for_finalized(begin, node.to_string(), rnd.to_string()))
                            }
                        }
                    } else {
                        top()
                    }
                })
            }
            let node = &caps[0];
            let rnd = &caps[1];
            accept(look_for_finalized(begin, node.to_string(), rnd.to_string()))
        } else {
            top()
        }
    } else {
        top()
    }
}

pub fn analysis_rules<'a>() -> Formula<&'a str> {
    and(
        with(&transactions_always_handled),
        and(
            with(&transactions_always_valid),
            with(&confirm_round_finalization),
        ),
    )
}

}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() <= 1 {
        println!("usage: logscan <FILES...>");
    }

    let path = &args[1];
    let contents = fs::read_to_string(path).expect("Could not read file");

    let formula = rules::analysis_rules();
    let mut st = Result::Continue(formula);

    for line in contents.lines() {
        match ltl::step(st, line) {
            Result::Failure(f) => {
                println!("Analysis FAILED: {}", f);
                std::process::exit(1)
            }
            Result::Success => {
                println!("Analysis completed successfully.");
                std::process::exit(0)
            }
            x => st = x
        }
    }
}

#[test]
fn speed_test() {
    fn odd(x: u64) -> bool {
        x % 2 == 1
    }
    fn even(x: u64) -> bool {
        x % 2 == 0
    }

    let formula1 = always(or(is(&odd), and(is(&even), next(is(&odd)))));
    let formula2 = always(until(is(&odd), is(&even)));

    fn run_it(f: Formula<u64>, n: u64) -> Result<'static, u64> {
        let mut s = Result::Continue(f);
        for i in 1..n {
            s = ltl::step(s, i)
        };
        s
    }

    fn gr_it(n: u64) -> bool {
        for i in 1..n {
            if ! (i>0) {
                return false
            }
        }
        return true
    }

    gr_it(1000000);
    run_it(formula1, 1000000);
    run_it(formula2, 1000000);
}
