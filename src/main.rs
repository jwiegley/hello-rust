use std::env;
use std::fs;

mod ltl;
use ltl::*;

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
