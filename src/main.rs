use std::env;
use std::fs;

mod ltl;
use ltl::Result;

mod rules;

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
