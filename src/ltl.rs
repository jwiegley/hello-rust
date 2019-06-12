use std::fmt;
use std::rc::Rc;

#[allow(dead_code)]
pub enum LTL<A> {
    Top,
    Bottom(String),

    // Accept rules take a state which is global to the aggregate LTL<A> formula.
    // There is no way to "scope" information using closures, such as there is
    // in Coq or Haskell, so intermediate states must be represented the
    // old-fashioned way.
    Accept(Box<dyn Fn(A) -> Rc<LTL<A>>>),

    And(Rc<LTL<A>>, Rc<LTL<A>>),
    Or(Rc<LTL<A>>, Rc<LTL<A>>),

    Next(Rc<LTL<A>>),

    Until(Rc<LTL<A>>, Rc<LTL<A>>),
    Release(Rc<LTL<A>>, Rc<LTL<A>>),
}

pub type Formula<A> = Rc<LTL<A>>;

pub fn top<A>() -> Formula<A> {
    Rc::new(LTL::Top)
}

pub fn bottom<A>(reason: String) -> Formula<A> {
    Rc::new(LTL::Bottom(reason))
}

pub fn accept<A>(f: Box<dyn Fn(A) -> Rc<LTL<A>>>) -> Formula<A> {
    Rc::new(LTL::Accept(f))
}

pub fn with<A, T>(f: &'static T) -> Formula<A>
where
    T: Fn(A) -> Formula<A>,
{
    accept(Box::new(f))
}

pub fn and<A>(p: Formula<A>, q: Formula<A>) -> Formula<A> {
    Rc::new(LTL::And(p, q))
}

pub fn or<A>(p: Formula<A>, q: Formula<A>) -> Formula<A> {
    Rc::new(LTL::Or(p, q))
}

#[allow(dead_code)]
pub fn next<A>(p: Formula<A>) -> Formula<A> {
    Rc::new(LTL::Next(p))
}

#[allow(dead_code)]
pub fn until<A>(p: Formula<A>, q: Formula<A>) -> Formula<A> {
    Rc::new(LTL::Until(p, q))
}

#[allow(dead_code)]
pub fn release<A>(p: Formula<A>, q: Formula<A>) -> Formula<A> {
    Rc::new(LTL::Release(p, q))
}

#[allow(dead_code)]
pub fn eventually<A>(p: Formula<A>) -> Formula<A> {
    until(top(), p)
}

#[allow(dead_code)]
pub fn always<A>(p: Formula<A>) -> Formula<A> {
    release(bottom("always".to_string()), p)
}

/// True if the given Haskell boolean is true.
#[allow(dead_code)]
pub fn truth<A>(b: bool) -> Formula<A> {
    if b {
        top()
    } else {
        bottom("truth".to_string())
    }
}

/// True if the given predicate on the input is true.
#[allow(dead_code)]
pub fn is<A: 'static, T>(f: &'static T) -> Formula<A>
where
    T: Fn(A) -> bool,
{
    accept(Box::new(move |x: A| truth(f(x))))
}

/// Another name for 'is'.
#[allow(dead_code)]
pub fn test<A: 'static, T>(f: &'static T) -> Formula<A>
where
    T: Fn(A) -> bool,
{
    is(f)
}

/// Check for equality with the input.
#[allow(dead_code)]
pub fn eq<A: 'static + PartialEq>(x: A) -> Formula<A> {
    accept(Box::new(move |y| truth(x == y)))
}

impl<A> fmt::Display for LTL<A> {
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

#[allow(dead_code)]
#[derive(Clone)]
pub enum Failed {
    HitBottom(String),
    EndOfTrace,
    Both(Box<Failed>, Box<Failed>),
    Left(Box<Failed>),
    Right(Box<Failed>),
}

impl fmt::Display for Failed {
    fn fmt(&self, dest: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Failed::HitBottom(reason) => write!(dest, "HitBottom {}", reason),
            Failed::EndOfTrace => write!(dest, "EndOfTrace"),
            Failed::Both(p, q) => write!(dest, "(Both {} {})", p, q),
            Failed::Left(p) => write!(dest, "(Left {})", p),
            Failed::Right(q) => write!(dest, "(Right {})", q),
        }
    }
}

pub enum Result<A> {
    Failure(Failed),
    Continue(Formula<A>),
    Success,
}

impl<A: fmt::Display> fmt::Display for Result<A> {
    fn fmt(&self, dest: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Result::Failure(f) => write!(dest, "Failure {}", f),
            Result::Continue(l) => write!(dest, "Continue {}", l),
            Result::Success => write!(dest, "Success"),
        }
    }
}

fn compile<A: Copy>(l: Formula<A>, mx: Option<A>) -> Result<A> {
    match &*l {
        LTL::Top => Result::Success,
        LTL::Bottom(s) => Result::Failure(Failed::HitBottom(s.clone())),

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
            Some(_) => compile(or(Rc::clone(q), and(Rc::clone(p), next(l))), mx),
        },

        LTL::Release(p, q) => match mx {
            None => compile(Rc::clone(q), mx),
            Some(_) => compile(and(Rc::clone(q), and(Rc::clone(p), next(l))), mx),
        },
    }
}

pub fn step<A: Copy>(m: Result<A>, x: A) -> Result<A> {
    match m {
        Result::Continue(l) => compile(l, Some(x)),
        r => r,
    }
}

#[allow(dead_code)]
pub fn run<A: Copy>(m: Formula<A>, xs: &[A]) -> Result<A> {
    if xs.len() == 0 {
        compile(m, None)
    } else {
        match compile(m, Some(xs[0])) {
            Result::Continue(l) => run(l, &xs[1..]),
            r => r,
        }
    }
}
