use std::fmt;
use std::rc::Rc;

#[allow(dead_code)]
pub enum LTL {
    Top,
    Bottom(String),

    // Accept rules take a state which is global to the aggregate LTL formula.
    // There is no way to "scope" information using closures, such as there is
    // in Coq or Haskell, so intermediate states must be represented the
    // old-fashioned way.
    Accept(Box<dyn Fn(&str) -> Rc<LTL>>),

    And(Rc<LTL>, Rc<LTL>),
    Or(Rc<LTL>, Rc<LTL>),

    Next(Rc<LTL>),

    Until(Rc<LTL>, Rc<LTL>),
    Release(Rc<LTL>, Rc<LTL>),
}

pub type Formula = Rc<LTL>;

pub fn top() -> Formula {
    Rc::new(LTL::Top)
}

pub fn bottom(reason: String) -> Formula {
    Rc::new(LTL::Bottom(reason))
}

pub fn accept(f: Box<dyn Fn(&str) -> Rc<LTL>>) -> Formula {
    Rc::new(LTL::Accept(f))
}

pub fn with<T>(f: &'static T) -> Formula where T: Fn(&str) -> Formula {
    accept(Box::new(f))
}

pub fn and(p: Formula, q: Formula) -> Formula {
    Rc::new(LTL::And(p, q))
}

pub fn or(p: Formula, q: Formula) -> Formula {
    Rc::new(LTL::Or(p, q))
}

#[allow(dead_code)]
pub fn next(p: Formula) -> Formula {
    Rc::new(LTL::Next(p))
}

#[allow(dead_code)]
pub fn until(p: Formula, q: Formula) -> Formula {
    Rc::new(LTL::Until(p, q))
}

#[allow(dead_code)]
pub fn release(p: Formula, q: Formula) -> Formula {
    Rc::new(LTL::Release(p, q))
}

#[allow(dead_code)]
pub fn eventually(p: Formula) -> Formula {
    until(top(), p)
}

#[allow(dead_code)]
pub fn always(p: Formula) -> Formula {
    release(bottom("always".to_string()), p)
}

impl fmt::Display for LTL {
    fn fmt(&self, dest: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LTL::Top => write!(dest, "Top"),
            LTL::Bottom(s) => write!(dest, "Bottom {}", s),
            LTL::Accept(_) => write!(dest, "Accept"),
            LTL::And(p, q) => write!(dest, "(And {} {})", p, q),
            LTL::Or(p, q) => write!(dest, "(Or {} {})", p, q),
            LTL::Next(p) => write!(dest, "(Next {})", p),
            LTL::Until(p, q) => write!(dest, "(Until {} {})", p, q),
            LTL::Release(p, q) => write!(dest, "(Release {} {})", p, q),
        }
    }
}

#[derive(Clone)]
#[allow(dead_code)]
pub enum Failed<'a> {
    HitBottom(String),
    EndOfTrace,
    Rejected(&'a str),
    Both(Box<Failed<'a>>, Box<Failed<'a>>),
    Left(Box<Failed<'a>>),
    Right(Box<Failed<'a>>),
}

impl<'a> fmt::Display for Failed<'a> {
    fn fmt(&self, dest: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Failed::HitBottom(reason) => write!(dest, "HitBottom {}", reason),
            Failed::EndOfTrace => write!(dest, "EndOfTrace"),
            Failed::Rejected(reason) => write!(dest, "Rejected {}", reason),
            Failed::Both(p, q) => write!(dest, "(Both {} {})", p, q),
            Failed::Left(p) => write!(dest, "(Left {})", p),
            Failed::Right(q) => write!(dest, "(Right {})", q),
        }
    }
}

pub enum Result<'a> {
    Failure(Failed<'a>),
    Continue(Formula),
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

fn compile<'a>(l: Formula, mx: Option<&str>) -> Result<'a> {
    match &*l {
        LTL::Top => Result::Success,
        LTL::Bottom(s) => Result::Failure(Failed::HitBottom(s.to_string())),

        LTL::Accept(v) => match mx {
            None => Result::Success,
            Some(x) => compile(v(x), mx),
        },

        LTL::And(p, q) => match compile(Rc::clone(p), mx) {
            Result::Failure(e) => Result::Failure(Failed::Left(Box::new(e))),
            Result::Success => compile(Rc::clone(q), mx),
            Result::Continue(f2) => match compile(Rc::clone(q), mx) {
                Result::Failure(e) => Result::Failure(Failed::Right(Box::new(e))),
                Result::Success => Result::Continue(f2),
                Result::Continue(g2) => Result::Continue(and(f2, g2)),
            },
        },

        LTL::Or(p, q) => match compile(Rc::clone(p), mx) {
            Result::Success => Result::Success,
            Result::Failure(e1) => match compile(Rc::clone(q), mx) {
                Result::Failure(e2) => Result::Failure(Failed::Both(Box::new(e1), Box::new(e2))),
                g2 => g2,
            },
            Result::Continue(f2) => match compile(Rc::clone(q), mx) {
                Result::Success => Result::Success,
                Result::Failure(_) => Result::Continue(f2),
                Result::Continue(g2) => Result::Continue(or(f2, g2)),
            },
        },

        LTL::Next(p) => match mx {
            None => compile(Rc::clone(p), mx),
            Some(_) => Result::Continue(Rc::clone(p)),
        },

        LTL::Until(p, q) => match mx {
            None => compile(Rc::clone(q), mx),
            Some(_) => compile(
                or(Rc::clone(q), and(Rc::clone(p), l)),
                mx,
            ),
        },

        LTL::Release(p, q) => match mx {
            None => compile(Rc::clone(q), mx),
            Some(_) => compile(
                and(Rc::clone(q), and(Rc::clone(p), l)),
                mx,
            ),
        },
    }
}

pub fn step<'a>(m: Result<'a>, x: &str) -> Result<'a> {
    match m {
        Result::Continue(l) => compile(Rc::clone(&l), Some(x)),
        r => r,
    }
}

#[allow(dead_code)]
pub fn run<'a>(m: Formula, xs: &[&str]) -> Result<'a> {
    if xs.len() == 0 {
        compile(m, None)
    } else {
        match compile(m, Some(xs[0])) {
            Result::Continue(l) => run(l, &xs[1..]),
            r => r,
        }
    }
}
