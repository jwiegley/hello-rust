// #[macro_use] extern crate lazy_static;
extern crate regex;

use chrono::DateTime;
use chrono::Duration;
use chrono::FixedOffset;
use regex::Regex;

use crate::ltl::*;

fn parse_time(text: &str) -> Option<DateTime<FixedOffset>> {
    let re: Regex = Regex::new(r" node-([0-9]+): \|([-0-9]+ [:0-9]+\.[0-9]+) UTC\|").ok()?;
    let caps = re.captures(text)?;
    DateTime::parse_from_str(&caps[1], "%F %T.%q").ok()
}

fn transactions_always_handled(text: &str) -> Formula<&str> {
    let re: Regex = Regex::new("node-0:.*?node/restapi: submitted tx: ([0-9a-f]{64})").unwrap();
    if let Some(begin) = parse_time(text) {
        if let Some(caps) = re.captures(text) {
            fn look_for_handled(
                begin: DateTime<FixedOffset>,
                tx: String,
            ) -> Box<dyn Fn(&str) -> Formula<&str>> {
                Box::new(move |text: &str| {
                    if let Some(now) = parse_time(text) {
                        if now.signed_duration_since(begin) > Duration::seconds(30) {
                            bottom("Failed to finalize transaction".to_string())
                        } else {
                            let re: Regex = Regex::new(
                                &("node-0:.*?consensus: finalized round [0-9]+, .*\\[.*"
                                    .to_string()
                                    + &tx
                                    + ".*\\]"),
                            )
                            .unwrap();
                            if re.is_match(text) {
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
    // lazy_static! {
    // static ref RE: Regex = Regex::new("^foo$").unwrap();
    // }
    let re: Regex = Regex::new("tx not valid for any round").unwrap();
    if re.is_match(text) {
        bottom("encountered \"tx not valid for any round\"".to_string())
    } else {
        top()
    }
}

fn confirm_round_finalization(text: &str) -> Formula<&str> {
    let re: Regex =
        Regex::new("node-([0-9]+):.*?consensus: starting new round: ([0-9]+),").unwrap();
    if let Some(begin) = parse_time(text) {
        if let Some(caps) = re.captures(text) {
            fn look_for_finalized(
                begin: DateTime<FixedOffset>,
                node: String,
                rnd: String,
            ) -> Box<dyn Fn(&str) -> Formula<&str>> {
                Box::new(move |text: &str| {
                    if let Some(now) = parse_time(text) {
                        if now.signed_duration_since(begin) > Duration::seconds(60) {
                            bottom("Failed to complete round".to_string())
                        } else {
                            let re: Regex = Regex::new(
                                &("node-".to_string()
                                    + &node
                                    + ":.*?consensus: finalized round ([0-9]+),"),
                            )
                            .unwrap();
                            if let Some(caps) = re.captures(text) {
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
