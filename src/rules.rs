// #[macro_use] extern crate lazy_static;
extern crate regex;

use regex::Regex;

use crate::ltl::*;

fn look_for_foo(text: &str) -> Formula {
    // lazy_static! {
    // static ref RE: Regex = Regex::new("^foo$").unwrap();
    // }
    let re: Regex = Regex::new("^foo$").unwrap();
    if re.is_match(text) {
        top()
    } else {
        bottom("look_for_foo".to_string())
    }
}

pub fn analysis_rules() -> Formula {
    with(&look_for_foo)
}
