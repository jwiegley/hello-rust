use std::env;
use std::fs;

enum LTL<A, S> {
    Top,
    Bottom,
    Accept(fn(A, S) -> (LTL<A, S>, S)),
    And(LTL<A, S>, LTL<A, S>),
    Or(LTL<A, S>, LTL<A, S>),

    Next(LTL<A, S>),

    Until(LTL<A, S>, LTL<A, S>),
    Release(LTL<A, S>, LTL<A, S>)
}

enum Either<A, B> {
    Left(A),
    Right(B)
}

struct LTL<T>
where T: Fn(String) -> Either<LTL<T>, Result<String>>
{
    step: T
}

enum Reason<A> {
    HitBottom(String),
    Rejected(A),
    BothFailed(Box<Reason<A>>, Box<Reason<A>>),
    LeftFailed(Box<Reason<A>>),
    RightFailed(Box<Reason<A>>)
}

type Result<A> = Option<Reason<A>>;

impl<T> LTL<T>
where T: Fn(String) -> Either<LTL<T>, Result<String>>
{
    pub fn stop(res: Result<String>) -> LTL<T> {
        LTL {
            step: |_x: String| -> Either<LTL<T>, Result<String>> {
                Right::<LTL<T>, Result<String>>(res)
            }
        }
    }
}

fn main() {
    let args : Vec<String> = env::args().collect();
    if args.len() <= 1 {
        println!("usage: logscan <FILES...>");
    }
    let path = &args[0];
    let contents = fs::read_to_string(path)
        .expect("Could not read file");
    let scanner = LTL::stop(None);
    for line in contents.lines() {
        (scanner.step)(line)
    }
}

//     newtype Machine a b = Machine { step :: a -> Either (Machine a b) b }
//
// instance Functor (Machine a) where
//     fmap f (Machine k) = Machine $ either (Left . fmap f) (Right . f) . k
//
//     run :: Machine a b -> [a] -> Either (Machine a b) b
//     run = foldl' (either step (const . Right)) . Left
// {-# INLINE run #-}
//
// data Reason a
//     = HitBottom   String
//     | Rejected    a
//     | BothFailed  (Reason a) (Reason a)
//     | LeftFailed  (Reason a)
//     | RightFailed (Reason a)
//     deriving (Show, Generic, NFData, Functor)
//
// type Result a = Maybe (Reason a)
//
// type LTL a = Machine a (Result a)
//
//     -- | ⊤, or "true"
//     top :: LTL a
//     top = stop Nothing
// {-# INLINE top #-}
//
// -- | ⊥, or "false"
//     bottom :: String -> LTL a
//     bottom = stop . Just . HitBottom
// {-# INLINE bottom #-}
//
// -- | Negate a formula: ¬ p
//     neg :: LTL a -> LTL a
//     neg = fmap invert
// {-# INLINE neg #-}
//
// -- | Boolean conjunction: ∧
//     and :: LTL a -> LTL a -> LTL a
//     and (Machine f) g = Machine $ \a -> case f a of
//     Right (Just e) -> Right (Just (LeftFailed e))
//     Right Nothing  -> step g a
//     Left f'        -> case step g a of
//     Right (Just e) -> Right (Just (RightFailed e))
//     Right Nothing  -> Left f'
//     Left g'        -> Left $! f' `and` g'
//
//     andNext :: LTL a -> LTL a -> LTL a
//     andNext (Machine f) g = Machine $ \a -> case f a of
//     Right (Just e) -> Right (Just (LeftFailed e))
//     Right Nothing  -> Left g
//     Left f'        -> Left $! f' `and` g
// {-# INLINE andNext #-}
//
// -- | Boolean disjunction: ∨
//     or :: LTL a -> LTL a -> LTL a
//     or (Machine f) g = Machine $ \a -> case f a of
//     Right Nothing   -> Right Nothing
//     Right (Just e1) -> case step g a of
//     Right (Just e2) -> Right (Just (BothFailed e1 e2))
//     g'              -> g'
//     Left f' -> case step g a of
//     Right Nothing  -> Right Nothing
//     Right (Just _) -> Left f'
//     Left g'        -> Left $! f' `or` g'
//
//     orNext :: LTL a -> LTL a -> LTL a
//     orNext (Machine f) g = Machine $ \a -> case f a of
//     Right Nothing  -> Right Nothing
//     Right (Just _) -> Left g
//     Left f'        -> Left $! f' `or` g
// {-# INLINE orNext #-}
//
// invert :: Result a -> Result a
//     invert = \case
//     Nothing -> Just (HitBottom "neg")
//     Just _  -> Nothing
// {-# INLINE invert #-}
//
// stop :: Result a -> LTL a
//     stop = Machine . const . Right
// {-# INLINE stop #-}
//
// -- | Given an input element, provide a formula to determine its truth. These
//     --   can be nested, making it possible to have conditional formulas. Consider
//     --   the following:
// --
//     -- @
//     -- always (accept (\n -> next (eq (succ n))))
//     -- @
//     --
//     --   One way to read this would be: "for every input n, always accept n if its
// --   next element is the successor".
//     accept :: (a -> LTL a) -> LTL a
//     accept f = Machine $ \a -> step (f a) a
// {-# INLINE accept #-}
//
// -- | The opposite in meaning to 'accept', defined simply as 'neg . accept'.
//     reject :: (a -> LTL a) -> LTL a
//     reject = neg . accept
// {-# INLINE reject #-}
//
// -- | The "next" temporal modality, typically written 'X p' or '◯ p'.
//     next :: LTL a -> LTL a
//     next = Machine . const . Left
// {-# INLINE next #-}
//
// -- | The "until" temporal modality, typically written 'p U q'.
//     until :: LTL a -> LTL a -> LTL a
//     until p = \q -> fix $ or q . andNext p
// {-# INLINE until #-}
//
// -- | Weak until.
//     weakUntil :: LTL a -> LTL a -> LTL a
//     weakUntil p = \q -> (p `until` q) `or` always p
// {-# INLINE weakUntil #-}
//
// -- | Release, the dual of 'until'.
//     release :: LTL a -> LTL a -> LTL a
//     release p = \q -> fix $ and q . orNext p
// {-# INLINE release #-}
//
// -- | Strong release.
//     strongRelease :: LTL a -> LTL a -> LTL a
//     strongRelease p = \q -> (p `release` q) `and` eventually p
// {-# INLINE strongRelease #-}
//
// -- | Logical implication: p → q
//     implies :: LTL a -> LTL a -> LTL a
//     implies = or . neg
// {-# INLINE implies #-}
//
// -- | Eventually the formula will hold, typically written F p or ◇ p.
//     eventually :: LTL a -> LTL a
//     eventually = until top
// {-# INLINE eventually #-}
//
// -- | Always the formula must hold, typically written G p or □ p.
//     always :: LTL a -> LTL a
//     always = release (bottom "always")
// {-# INLINE always #-}
//
// -- | True if the given Haskell boolean is true.
//     truth :: Bool -> LTL a
//     truth True  = top
//     truth False = bottom "truth"
// {-# INLINE truth #-}
//
// -- | True if the given predicate on the input is true.
//     is :: (a -> Bool) -> LTL a
//     is = accept . (truth .)
// {-# INLINE is #-}
//
// -- | Another name for 'is'.
//     test :: (a -> Bool) -> LTL a
//     test = is
// {-# INLINE test #-}
//
// -- | Check for equality with the input.
//     eq :: Eq a => a -> LTL a
//     eq = is . (==)
// {-# INLINE eq #-}
//
// -- | Render a 'step' result as a string.
//     showResult :: Show a => Either (LTL a) (Result a) -> String
//     showResult (Left _)  = "<need input>"
//     showResult (Right b) = show b
// {-# INLINE showResult #-}
