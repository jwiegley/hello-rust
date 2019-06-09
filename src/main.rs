// #[macro_use] extern crate lazy_static;
extern crate regex;

use regex::Regex;

// use std::env;
use std::fmt;
// use std::fs;
use std::rc::Rc;

enum LTL {
    Top,
    Bottom,

    // Accept rules take a state which is global to the aggregate LTL formula.
    // There is no way to "scope" information using closures, such as there is
    // in Coq or Haskell, so intermediate states must be represented the
    // old-fashioned way.
    Accept(fn(&str) -> Rc<LTL>),

    And(Rc<LTL>, Rc<LTL>),
    Or(Rc<LTL>, Rc<LTL>),

    Next(Rc<LTL>),

    Until(Rc<LTL>, Rc<LTL>),
    Release(Rc<LTL>, Rc<LTL>),
}

impl fmt::Display for LTL {
    fn fmt(&self, dest: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LTL::Top => write!(dest, "Top"),
            LTL::Bottom => write!(dest, "Bottom"),
            LTL::Accept(_) => write!(dest, "Accept"),
            LTL::And(p, q) => write!(dest, "(And {} {})", p, q),
            LTL::Or(p, q) => write!(dest, "(Or {} {})", p, q),
            LTL::Next(p) => write!(dest, "(Next {})", p),
            LTL::Until(p, q) => write!(dest, "(Until {} {})", p, q),
            LTL::Release(p, q) => write!(dest, "(Release {} {})", p, q),
        }
    }
}

enum Failed<'a> {
    HitBottom,
    EndOfTrace,
    Rejected(&'a str),
    Both(Box<Failed<'a>>, Box<Failed<'a>>),
    Left(Box<Failed<'a>>),
    Right(Box<Failed<'a>>),
}

impl<'a> fmt::Display for Failed<'a> {
    fn fmt(&self, dest: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Failed::HitBottom => write!(dest, "HitBottom"),
            Failed::EndOfTrace => write!(dest, "EndOfTrace"),
            Failed::Rejected(reason) => write!(dest, "Rejected {}", reason),
            Failed::Both(p, q) => write!(dest, "(Both {} {})", p, q),
            Failed::Left(p) => write!(dest, "(Left {})", p),
            Failed::Right(q) => write!(dest, "(Right {})", q),
        }
    }
}

enum Result<'a> {
    Failure(Failed<'a>),
    Continue(Rc<LTL>),
    Success,
}

impl<'a> fmt::Display for Result<'a> {
    fn fmt(&self, dest: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Result::Failure(f) => write!(dest, "Failure {}", f),
            Result::Continue(l) => write!(dest, "Continue {}", l),
            Result::Success => write!(dest, "Success"),
        }
    }
}

fn and<'a>(f: Result<'a>, g: Result<'a>) -> Result<'a> {
    match (f, g) {
        (Result::Failure(e), _) => Result::Failure(Failed::Left(Box::new(e))),
        (_, Result::Failure(e)) => Result::Failure(Failed::Right(Box::new(e))),
        (Result::Continue(f2), Result::Continue(g2)) => Result::Continue(Rc::new(LTL::And(f2, g2))),
        (f2, Result::Success) => f2,
        (Result::Success, g2) => g2,
    }
}

fn or<'a>(f: Result<'a>, g: Result<'a>) -> Result<'a> {
    match (f, g) {
        (Result::Failure(e1), Result::Failure(e2)) => {
            Result::Failure(Failed::Both(Box::new(e1), Box::new(e2)))
        }
        (Result::Success, _) => Result::Success,
        (_, Result::Success) => Result::Success,
        (Result::Continue(f2), Result::Continue(g2)) => Result::Continue(Rc::new(LTL::Or(f2, g2))),
        (Result::Failure(_), g2) => g2,
        (f2, Result::Failure(_)) => f2,
    }
}

fn compile<'a>(l: Rc<LTL>, mx: Option<&str>) -> Result<'a> {
    match &*l {
        LTL::Top => Result::Success,
        LTL::Bottom => Result::Failure(Failed::HitBottom),

        LTL::Accept(v) => match mx {
            None => Result::Success,
            Some(x) => compile(v(x), mx),
        },

        LTL::And(p, q) => and(compile(Rc::clone(p), mx), compile(Rc::clone(&q), mx)),
        LTL::Or(p, q) => or(compile(Rc::clone(p), mx), compile(Rc::clone(&q), mx)),

        LTL::Next(p) => match mx {
            None => compile(Rc::clone(&p), mx),
            Some(_) => Result::Continue(Rc::clone(p)),
        },

        LTL::Until(p, q) => match mx {
            None => compile(Rc::clone(&q), mx),
            Some(_) => or(
                compile(Rc::clone(&q), mx),
                and(
                    compile(Rc::clone(&p), mx),
                    Result::Continue(Rc::new(LTL::Until(Rc::clone(p), Rc::clone(q)))),
                ),
            ),
        },

        LTL::Release(p, q) => match mx {
            None => compile(Rc::clone(&q), mx),
            Some(_) => and(
                compile(Rc::clone(&q), mx),
                or(
                    compile(Rc::clone(&p), mx),
                    Result::Continue(Rc::new(LTL::Release(Rc::clone(p), Rc::clone(q)))),
                ),
            ),
        },
    }
}

fn step<'a>(m: Result<'a>, x: &str) -> Result<'a> {
    match m {
        Result::Continue(l) => compile(Rc::clone(&l), Some(x)),
        r => r,
    }
}

fn run<'a>(m: Rc<LTL>, xs: &[&str]) -> Result<'a> {
    if xs.len() == 0 {
        compile(m, None)
    } else {
        match compile(m, Some(xs[0])) {
            Result::Continue(l) => run(l, &xs[1..]),
            r => r,
        }
    }
}

fn look_for_foo(text: &str) -> Rc<LTL> {
    // lazy_static! {
        // static ref RE: Regex = Regex::new("^foo$").unwrap();
    // }
    let re: Regex = Regex::new("^foo$").unwrap();
    if re.is_match(text) {
        Rc::new(LTL::Top)
    } else {
        Rc::new(LTL::Bottom)
    }
}

fn main() {
    // let args: Vec<String> = env::args().collect();
    // if args.len() <= 1 {
    //     println!("usage: logscan <FILES...>");
    // }

    // let path = &args[0];
    // let contents = fs::read_to_string(path).expect("Could not read file");
    let contents = "foo";

    let formula = Rc::new(LTL::Accept(look_for_foo));
    let mut st = Result::Continue(formula);

    for line in contents.lines() {
        st = step(st, line);
    }

    // run(Rc::new(LTL::Top), contents.lines());

    println!("Result = {}", st)
}

#[test]
fn main_test() {
    let contents = "foo";

    let mut formula = Result::Continue(Rc::new(LTL::Top));

    for line in contents.lines() {
        formula = step(formula, line);
    }

    // run(Rc::new(LTL::Top), contents.lines());

    println!("Result = {}", formula)
}
