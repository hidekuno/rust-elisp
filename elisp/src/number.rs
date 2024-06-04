/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
use std::cmp::Ordering;
use std::cmp::PartialEq;
use std::cmp::PartialOrd;
use std::error::Error;
use std::fmt;
use std::ops::Add;
use std::ops::Div;
use std::ops::Mul;
use std::ops::Sub;

use crate::lisp::ErrCode;
use crate::lisp::Expression;
use crate::lisp::Int;

#[allow(unused_imports)]
use log::{debug, error, info, warn};
//========================================================================
#[derive(Debug)]
pub struct RatParseError {
    pub code: ErrCode,
}
impl fmt::Display for RatParseError {
    fn fmt(&self, format: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(format, "{}", self.code.as_str())
    }
}
impl Error for RatParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}
#[derive(Debug, Copy, Clone)]
pub struct Rat {
    pub numer: Int,
    pub denom: Int,
}
impl Rat {
    pub fn new(n: Int, d: Int) -> Rat {
        let l = gcm(n, d);
        let sign = if n * d < 0 { -1 } else { 1 };

        Rat {
            numer: (n.wrapping_abs() / l) * sign,
            denom: d.wrapping_abs() / l,
        }
    }
    pub fn div_float(&self) -> f64 {
        self.numer as f64 / self.denom as f64
    }
    pub fn abs(&self) -> Rat {
        Rat {
            numer: self.numer.abs(),
            denom: self.denom,
        }
    }
    pub fn from(s: &str) -> Result<Rat, RatParseError> {
        Rat::from_radix(s, 10)
    }
    pub fn from_radix(s: &str, r: u32) -> Result<Rat, RatParseError> {
        let mut v = Vec::new();
        for e in s.split('/') {
            if let Ok(n) = Int::from_str_radix(e, r) {
                v.push(n);
            }
        }
        if v.len() == 2 {
            if v[1] == 0 {
                return Err(RatParseError {
                    code: ErrCode::E1013,
                });
            }
        } else {
            return Err(RatParseError {
                code: ErrCode::E1020,
            });
        }
        Ok(Rat::new(v[0], v[1]))
    }
}
fn gcm(n: Int, m: Int) -> Int {
    match n % m {
        0 => m.wrapping_abs(),
        l => gcm(m, l),
    }
}
// ToString -> Display
// https://rust-lang.github.io/rust-clippy/master/index.html#/to_string_trait_impl
impl fmt::Display for Rat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.denom == 1 {
            write!(f, "{}", self.numer)
        } else {
            write!(f, "{}/{}", self.numer, self.denom)
        }
    }
}
impl Add for Rat {
    type Output = Rat;
    fn add(self: Rat, other: Rat) -> Rat {
        Rat::new(
            (self.numer * other.denom) + (other.numer * self.denom),
            self.denom * other.denom,
        )
    }
}
impl Sub for Rat {
    type Output = Rat;
    fn sub(self: Rat, other: Rat) -> Rat {
        Rat::new(
            (self.numer * other.denom) - (other.numer * self.denom),
            self.denom * other.denom,
        )
    }
}
impl Mul for Rat {
    type Output = Rat;
    fn mul(self: Rat, other: Rat) -> Rat {
        Rat::new(self.numer * other.numer, self.denom * other.denom)
    }
}
impl Div for Rat {
    type Output = Rat;
    fn div(self: Rat, other: Rat) -> Rat {
        Rat::new(self.numer * other.denom, self.denom * other.numer)
    }
}
impl PartialEq for Rat {
    fn eq(&self, other: &Rat) -> bool {
        (self.numer == other.numer) && (self.denom == other.denom)
    }
}
impl Eq for Rat {}

impl Ord for Rat {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.lt(other) {
            Ordering::Less
        } else if self.gt(other) {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}
impl PartialOrd for Rat {
    fn lt(&self, other: &Rat) -> bool {
        (self.numer * other.denom) < (other.numer * self.denom)
    }
    fn le(&self, other: &Rat) -> bool {
        (self.numer * other.denom) <= (other.numer * self.denom)
    }
    fn gt(&self, other: &Rat) -> bool {
        (self.numer * other.denom) > (other.numer * self.denom)
    }
    fn ge(&self, other: &Rat) -> bool {
        (self.numer * other.denom) >= (other.numer * self.denom)
    }
    fn partial_cmp(&self, other: &Rat) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
#[derive(Debug, Copy, Clone)]
pub enum Number {
    Integer(Int),
    Float(f64),
    Rational(Rat),
}
impl Number {
    fn calc<I, F, R, V>(self: Number, other: Number, icalc: I, fcalc: F, rcalc: R) -> V
    where
        I: Fn(Int, Int) -> V,
        F: Fn(f64, f64) -> V,
        R: Fn(Rat, Rat) -> V,
    {
        match self {
            Number::Integer(a) => match other {
                Number::Integer(b) => icalc(a, b),
                Number::Float(b) => fcalc(a as f64, b),
                Number::Rational(b) => rcalc(Rat::new(a, 1), b),
            },
            Number::Float(a) => match other {
                Number::Integer(b) => fcalc(a, b as f64),
                Number::Float(b) => fcalc(a, b),
                Number::Rational(b) => fcalc(a, b.div_float()),
            },
            Number::Rational(a) => match other {
                Number::Integer(b) => rcalc(a, Rat::new(b, 1)),
                Number::Float(b) => fcalc(a.div_float(), b),
                Number::Rational(b) => rcalc(a, b),
            },
        }
    }
    pub fn to_expression(self: Number) -> Expression {
        match self {
            Number::Integer(a) => Expression::Integer(a),
            Number::Float(a) => Expression::Float(a),
            Number::Rational(a) => Expression::Rational(a),
        }
    }
}
//impl<T: Add<Output=T>> Add for Number<T> {
impl Add for Number {
    type Output = Number;

    fn add(self, other: Number) -> Number {
        self.calc::<fn(Int, Int) -> Number, fn(f64, f64) -> Number, fn(Rat, Rat) -> Number, Number>(
            other,
            |x: Int, y: Int| Number::Integer(x + y),
            |x: f64, y: f64| Number::Float(x + y),
            |x: Rat, y: Rat| Number::Rational(x + y),
        )
    }
}
impl Sub for Number {
    type Output = Number;
    fn sub(self, other: Number) -> Number {
        self.calc::<fn(Int, Int) -> Number, fn(f64, f64) -> Number, fn(Rat, Rat) -> Number, Number>(
            other,
            |x: Int, y: Int| Number::Integer(x - y),
            |x: f64, y: f64| Number::Float(x - y),
            |x: Rat, y: Rat| Number::Rational(x - y),
        )
    }
}
impl Mul for Number {
    type Output = Number;
    fn mul(self, other: Number) -> Number {
        self.calc::<fn(Int, Int) -> Number, fn(f64, f64) -> Number, fn(Rat, Rat) -> Number, Number>(
            other,
            |x: Int, y: Int| Number::Integer(x * y),
            |x: f64, y: f64| Number::Float(x * y),
            |x: Rat, y: Rat| Number::Rational(x * y),
        )
    }
}
impl Div for Number {
    type Output = Number;
    fn div(self, other: Number) -> Number {
        if let (Number::Integer(x), Number::Integer(y)) = (self, other) {
            if x == 0 && y == 0 {
                return Number::Float(std::f64::NAN);
            }
            if y == 0 {
                return Number::Float(std::f64::INFINITY);
            }
            if 0 != (x % y) {
                return self
                    .calc::<fn(Int, Int) -> Number, fn(f64, f64) -> Number, fn(Rat, Rat) -> Number, Number>(
                        Number::Rational(Rat::new(y, 1)),
                        |x: Int, y: Int| Number::Integer(x / y),
                        |x: f64, y: f64| Number::Float(x / y),
                        |x: Rat, y: Rat| Number::Rational(x / y),
                    );
            }
        }
        self.calc::<fn(Int, Int) -> Number, fn(f64, f64) -> Number, fn(Rat, Rat) -> Number, Number>(
            other,
            |x: Int, y: Int| Number::Integer(x / y),
            |x: f64, y: f64| Number::Float(x / y),
            |x: Rat, y: Rat| Number::Rational(x / y),
        )
    }
}
impl PartialEq for Number {
    fn eq(&self, other: &Number) -> bool {
        self.calc::<fn(Int, Int) -> bool, fn(f64, f64) -> bool, fn(Rat, Rat) -> bool, bool>(
            *other,
            |x: Int, y: Int| x == y,
            |x: f64, y: f64| x == y,
            |x: Rat, y: Rat| x == y,
        )
    }
}
impl Eq for Number {}
impl Ord for Number {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.lt(other) {
            Ordering::Less
        } else if self.gt(other) {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}
impl PartialOrd for Number {
    fn lt(&self, other: &Number) -> bool {
        self.calc::<fn(Int, Int) -> bool, fn(f64, f64) -> bool, fn(Rat, Rat) -> bool, bool>(
            *other,
            |x: Int, y: Int| x < y,
            |x: f64, y: f64| x < y,
            |x: Rat, y: Rat| x < y,
        )
    }
    fn le(&self, other: &Number) -> bool {
        self.calc::<fn(Int, Int) -> bool, fn(f64, f64) -> bool, fn(Rat, Rat) -> bool, bool>(
            *other,
            |x: Int, y: Int| x <= y,
            |x: f64, y: f64| x <= y,
            |x: Rat, y: Rat| x <= y,
        )
    }
    fn gt(&self, other: &Number) -> bool {
        self.calc::<fn(Int, Int) -> bool, fn(f64, f64) -> bool, fn(Rat, Rat) -> bool, bool>(
            *other,
            |x: Int, y: Int| x > y,
            |x: f64, y: f64| x > y,
            |x: Rat, y: Rat| x > y,
        )
    }
    fn ge(&self, other: &Number) -> bool {
        self.calc::<fn(Int, Int) -> bool, fn(f64, f64) -> bool, fn(Rat, Rat) -> bool, bool>(
            *other,
            |x: Int, y: Int| x >= y,
            |x: f64, y: f64| x >= y,
            |x: Rat, y: Rat| x >= y,
        )
    }
    fn partial_cmp(&self, other: &Number) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
// ToString -> Display
// https://rust-lang.github.io/rust-clippy/master/index.html#/to_string_trait_impl
impl fmt::Display for Number {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Number::Integer(v) => write!(f, "{}", v),
            Number::Float(v) => write!(f, "{}", v),
            Number::Rational(v) => write!(f, "{}", v),
        }
    }
}
#[test]
fn test_rat_order() {
    let mut vec = [
        Rat::new(1, 2),
        Rat::new(1, 3),
        Rat::new(1, 4),
        Rat::new(1, 3),
    ];
    vec.sort();
    assert_eq!(vec[0].to_string(), "1/4");

    vec.sort_by(|a, b| b.cmp(a));
    assert_eq!(vec[0].to_string(), "1/2");

    vec.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(vec[0].to_string(), "1/4");

    // for branch check
    vec.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(vec[0].to_string(), "1/4");
}
#[test]
fn test_number_order() {
    let mut vec = [
        Number::Integer(4),
        Number::Integer(3),
        Number::Integer(3),
        Number::Integer(2),
    ];
    vec.sort();
    assert_eq!(vec[0].to_string(), "2");

    vec.sort_by(|a, b| b.cmp(a));
    assert_eq!(vec[0].to_string(), "4");

    vec.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(vec[0].to_string(), "2");
}
#[test]
fn test_rat() {
    assert_eq!(Rat::new(1, 2).cmp(&Rat::new(1, 4)), Ordering::Greater);
}
#[test]
fn test_rat_error() {
    let _ = Rat::from("31.1").map_err(|e| assert_eq!(e.to_string(), "E1020"));
    let _ = Rat::from("31.1").map_err(|e| assert!(e.source().is_none()));
}
#[test]
fn test_gcm() {
    assert_eq!(gcm(17, 2), 1);
    assert_eq!(gcm(36, 27), 9);
    assert_eq!(gcm(27, 36), 9);
    assert_eq!(gcm(-27, 36), 9);
    assert_eq!(gcm(27, -36), 9);
}
#[test]
fn test_add_integer() {
    assert_eq!(Number::Integer(2) + Number::Integer(3), Number::Integer(5));
    assert_eq!(
        Number::Integer(1) + Number::Rational(Rat::new(1, 2)),
        Number::Rational(Rat::new(3, 2))
    );
    assert!(
        matches!(Number::Integer(2) + Number::Float(2.5), Number::Float(v) if (v - 4.5).abs() < f64::EPSILON),
    );
}
#[test]
fn test_add_float() {
    assert!(
        matches!(Number::Float(1.5) + Number::Integer(3), Number::Float(v) if (v - 4.5).abs() < f64::EPSILON)
    );
    assert!(
        matches!(Number::Float(1.5) + Number::Float(1.25),Number::Float(v) if (v - 2.75).abs() < f64::EPSILON)
    );
    assert!(
        matches!(Number::Float(2.5) + Number::Rational(Rat::new(1, 4)),
                 Number::Float(v) if (v - 2.75).abs() < f64::EPSILON)
    );
}
#[test]
fn test_add_rational() {
    assert_eq!(
        Number::Rational(Rat::new(3, 4)) + Number::Integer(1),
        Number::Rational(Rat::new(7, 4))
    );
    assert!(
        matches!(Number::Rational(Rat::new(1, 4)) + Number::Float(2.5),
                 Number::Float(v) if (v - 2.75).abs() < f64::EPSILON)
    );
    assert_eq!(
        Number::Rational(Rat::new(3, 4)) + Number::Rational(Rat::new(1, 3)),
        Number::Rational(Rat::new(13, 12))
    );
}
#[test]
fn test_sub_integer() {
    assert_eq!(Number::Integer(10) - Number::Integer(3), Number::Integer(7));
    assert_eq!(
        Number::Integer(1) - Number::Rational(Rat::new(1, 2)),
        Number::Rational(Rat::new(1, 2))
    );
    assert!(
        matches!(Number::Integer(1) - Number::Float(2.5), Number::Float(v) if (v - (-1.5)).abs() < f64::EPSILON)
    );
}
#[test]
fn test_sub_float() {
    assert!(matches!(Number::Float(4.5) - Number::Integer(3),
                 Number::Float(v) if (v - 1.5).abs() < f64::EPSILON));
    assert!(matches!(Number::Float(1.5) - Number::Float(1.25),
              Number::Float(v) if (v - 0.25).abs() < f64::EPSILON));
    assert!(
        matches!(Number::Float(2.5) - Number::Rational(Rat::new(1, 4)),
                 Number::Float(v) if (v - 2.25).abs() < f64::EPSILON)
    );
}
#[test]
fn test_sub_rational() {
    assert_eq!(
        Number::Rational(Rat::new(1, 2)) - Number::Integer(1),
        Number::Rational(Rat::new(-1, 2))
    );
    assert!(
        matches!(Number::Rational(Rat::new(3, 4)) - Number::Float(0.5),
                 Number::Float(v) if (v - 0.25).abs() < f64::EPSILON)
    );
    assert_eq!(
        Number::Rational(Rat::new(3, 4)) - Number::Rational(Rat::new(1, 2)),
        Number::Rational(Rat::new(1, 4))
    );
}
#[test]
fn test_mul_integer() {
    assert_eq!(
        Number::Integer(10) * Number::Integer(3),
        Number::Integer(30)
    );
    assert_eq!(
        Number::Integer(3) * Number::Rational(Rat::new(1, 2)),
        Number::Rational(Rat::new(3, 2))
    );
    assert!(matches!(
        Number::Integer(1) * Number::Float(2.5),
        Number::Float(v) if (v - 2.5).abs() < f64::EPSILON
    ));
}
#[test]
fn test_mul_float() {
    assert!(matches!( Number::Float(4.5) * Number::Integer(3),
        Number::Float(v) if (v - 13.5).abs() < f64::EPSILON));
    assert!(matches!( Number::Float(1.8) * Number::Float(1.8),
        Number::Float(v) if (v - 3.24).abs() < f64::EPSILON),);
    assert!(
        matches!( Number::Float(2.5) * Number::Rational(Rat::new(1, 4)),
        Number::Float(v) if (v - 0.625).abs() < f64::EPSILON),
    );
}
#[test]
fn test_mul_rational() {
    assert_eq!(
        Number::Rational(Rat::new(1, 2)) * Number::Integer(3),
        Number::Rational(Rat::new(3, 2))
    );
    assert!(
        matches!(Number::Rational(Rat::new(3, 4)) * Number::Float(0.5),
             Number::Float(v) if (v - 0.375).abs() < f64::EPSILON)
    );
    assert_eq!(
        Number::Rational(Rat::new(3, 4)) * Number::Rational(Rat::new(1, 2)),
        Number::Rational(Rat::new(3, 8))
    );
}
#[test]
fn test_div_integer() {
    assert_eq!(Number::Integer(8) / Number::Integer(2), Number::Integer(4));
    assert_eq!(
        Number::Integer(3) / Number::Rational(Rat::new(1, 2)),
        Number::Rational(Rat::new(6, 1))
    );
    assert!(matches!(Number::Integer(1) / Number::Float(2.5),
                 Number::Float(v) if(v - 0.4).abs() < f64::EPSILON));
}
#[test]
fn test_div_float() {
    assert!(matches!( Number::Float(4.5) / Number::Integer(3),
        Number::Float(v) if(v - 1.5).abs() < f64::EPSILON),);
    assert!(matches!( Number::Float(3.6) / Number::Float(3.2),
        Number::Float(v) if(v - 1.125).abs() < f64::EPSILON),);
    assert!(
        matches!( Number::Float(2.5) / Number::Rational(Rat::new(1, 3)),
        Number::Float(v) if(v - 7.5).abs() < f64::EPSILON),
    );
}
#[test]
fn test_div_rational() {
    assert_eq!(
        Number::Rational(Rat::new(1, 2)) / Number::Integer(3),
        Number::Rational(Rat::new(1, 6))
    );
    assert!(
        matches!(Number::Rational(Rat::new(3, 4)) / Number::Float(0.5),
        Number::Float(v) if(v - 1.5).abs() < f64::EPSILON)
    );
    assert_eq!(
        Number::Rational(Rat::new(3, 4)) / Number::Rational(Rat::new(1, 2)),
        Number::Rational(Rat::new(3, 2))
    );
}
#[test]
fn test_eq_rational() {
    assert!(Number::Integer(3) == Number::Rational(Rat::new(6, 2)));
    assert!(Number::Float(0.5) == Number::Rational(Rat::new(1, 2)));

    assert!(Number::Rational(Rat::new(6, 2)) == Number::Integer(3));
    assert!(Number::Rational(Rat::new(3, 2)) == Number::Float(1.5));
    assert!(Number::Rational(Rat::new(4, 8)) == Number::Rational(Rat::new(2, 4)));
}

#[test]
fn test_lt_rational() {
    assert!(Number::Integer(3) < Number::Rational(Rat::new(7, 2)));
    assert!(Number::Float(0.3) < Number::Rational(Rat::new(1, 2)));

    assert!(Number::Rational(Rat::new(6, 2)) < Number::Integer(4));
    assert!(Number::Rational(Rat::new(3, 2)) < Number::Float(1.6));
    assert!(Number::Rational(Rat::new(4, 8)) < Number::Rational(Rat::new(3, 4)));
}

#[test]
fn test_le_rational() {
    assert!(Number::Integer(3) <= Number::Rational(Rat::new(7, 2)));
    assert!(Number::Float(0.3) <= Number::Rational(Rat::new(1, 2)));
    assert!(Number::Rational(Rat::new(6, 2)) <= Number::Integer(4));
    assert!(Number::Rational(Rat::new(3, 2)) <= Number::Float(1.6));
    assert!(Number::Rational(Rat::new(4, 8)) <= Number::Rational(Rat::new(3, 4)));

    assert!(Number::Integer(3) <= Number::Rational(Rat::new(6, 2)));
    assert!(Number::Float(0.5) <= Number::Rational(Rat::new(1, 2)));
    assert!(Number::Rational(Rat::new(6, 2)) <= Number::Integer(3));
    assert!(Number::Rational(Rat::new(3, 2)) <= Number::Float(1.5));
    assert!(Number::Rational(Rat::new(4, 8)) <= Number::Rational(Rat::new(2, 4)));
}
#[test]
fn test_gt_rational() {
    assert!(Number::Rational(Rat::new(7, 2)) > Number::Integer(3));
    assert!(Number::Rational(Rat::new(1, 2)) > Number::Float(0.3));

    assert!(Number::Integer(4) > Number::Rational(Rat::new(6, 2)));
    assert!(Number::Float(1.6) > Number::Rational(Rat::new(3, 2)));
    assert!(Number::Rational(Rat::new(3, 4)) > Number::Rational(Rat::new(4, 8)));
}
#[test]
fn test_ge_rational() {
    assert!(Number::Rational(Rat::new(7, 2)) >= Number::Integer(3));
    assert!(Number::Rational(Rat::new(1, 2)) >= Number::Float(0.3));
    assert!(Number::Integer(4) >= Number::Rational(Rat::new(6, 2)));
    assert!(Number::Float(1.6) >= Number::Rational(Rat::new(3, 2)));
    assert!(Number::Rational(Rat::new(3, 4)) >= Number::Rational(Rat::new(4, 8)));

    assert!(Number::Integer(3) >= Number::Rational(Rat::new(6, 2)));
    assert!(Number::Float(0.5) >= Number::Rational(Rat::new(1, 2)));
    assert!(Number::Rational(Rat::new(6, 2)) >= Number::Integer(3));
    assert!(Number::Rational(Rat::new(3, 2)) >= Number::Float(1.5));
    assert!(Number::Rational(Rat::new(4, 8)) >= Number::Rational(Rat::new(2, 4)));
}
#[cfg(test)]
mod tests {
    use crate::do_lisp;

    #[test]
    fn test_sing_rational() {
        assert_eq!(do_lisp("(/ -1 3)"), "-1/3");
        assert_eq!(do_lisp("(/ 1 -3)"), "-1/3");
        assert_eq!(do_lisp("(/ -1 -3)"), "1/3");
        assert_eq!(do_lisp("(+ (/ -1 3)(/ 1 3))"), "0");
    }
    #[test]
    fn test_add_rational() {
        assert_eq!(do_lisp("(+ 1 1/2)"), "3/2");
        assert_eq!(do_lisp("(+ 2.5 1/4)"), "2.75");
        assert_eq!(do_lisp("(+ 3/4 1)"), "7/4");
        assert_eq!(do_lisp("(+ 1/4 2.5)"), "2.75");
        assert_eq!(do_lisp("(+ 3/4 1/3)"), "13/12");
        assert_eq!(do_lisp("(+ -1/3 1/3)"), "0");
    }
    #[test]
    fn test_sub_rational() {
        assert_eq!(do_lisp("(- 1 1/2)"), "1/2");
        assert_eq!(do_lisp("(- 2.5 1/4)"), "2.25");
        assert_eq!(do_lisp("(- 1/2 1)"), "-1/2");
        assert_eq!(do_lisp("(- 3/4 0.5)"), "0.25");
        assert_eq!(do_lisp("(- 3/4 1/2)"), "1/4");
    }
    #[test]
    fn test_mul_rational() {
        assert_eq!(do_lisp("(* 3 1/2)"), "3/2");
        assert_eq!(do_lisp("(* 2.5 1/4)"), "0.625");
        assert_eq!(do_lisp("(* 1/2 3)"), "3/2");
        assert_eq!(do_lisp("(* 3/4 0.5)"), "0.375");
        assert_eq!(do_lisp("(* 3/4 1/2)"), "3/8");
    }
    #[test]
    fn test_div_rational() {
        assert_eq!(do_lisp("(/ 3 1/2)"), "6");
        assert_eq!(do_lisp("(/ 2.5 1/3)"), "7.5");
        assert_eq!(do_lisp("(/ 1/2 3)"), "1/6");
        assert_eq!(do_lisp("(/ 3/4 0.5)"), "1.5");
        assert_eq!(do_lisp("(/ 3/4 1/2)"), "3/2");
    }
    #[test]
    fn test_eq_rational() {
        assert_eq!(do_lisp("(= 3 6/2)"), "#t");
        assert_eq!(do_lisp("(= 0.5 1/2)"), "#t");
        assert_eq!(do_lisp("(= 6/2 3)"), "#t");
        assert_eq!(do_lisp("(= 3/2 1.5)"), "#t");
        assert_eq!(do_lisp("(= 4/8 2/4)"), "#t");
    }
    #[test]
    fn test_lt_rational() {
        assert_eq!(do_lisp("(< 3 7/2)"), "#t");
        assert_eq!(do_lisp("(< 0.3 1/2)"), "#t");
        assert_eq!(do_lisp("(< 6/2 4)"), "#t");
        assert_eq!(do_lisp("(< 4/8 3/4)"), "#t");
    }
    #[test]
    fn test_le_rational() {
        assert_eq!(do_lisp("(<= 3 7/2)"), "#t");
        assert_eq!(do_lisp("(<= 0.3 1/2)"), "#t");
        assert_eq!(do_lisp("(<= 6/2 4)"), "#t");
        assert_eq!(do_lisp("(<= 4/8 3/4)"), "#t");

        assert_eq!(do_lisp("(<= 3 6/2)"), "#t");
        assert_eq!(do_lisp("(<= 0.5 1/2)"), "#t");
        assert_eq!(do_lisp("(<= 6/2 3)"), "#t");
        assert_eq!(do_lisp("(<= 3/2 1.5)"), "#t");
        assert_eq!(do_lisp("(<= 4/8 2/4)"), "#t");
    }
    #[test]
    fn test_gt_rational() {
        assert_eq!(do_lisp("(> 7/2 3)"), "#t");
        assert_eq!(do_lisp("(> 1/2 0.3)"), "#t");
        assert_eq!(do_lisp("(> 4 6/2)"), "#t");
        assert_eq!(do_lisp("(> 1.6 3/2)"), "#t");
        assert_eq!(do_lisp("(> 3/4 4/8)"), "#t");
    }
    #[test]
    fn test_ge_rational() {
        assert_eq!(do_lisp("(>= 7/2 3)"), "#t");
        assert_eq!(do_lisp("(>= 1/2 0.3)"), "#t");
        assert_eq!(do_lisp("(>= 4 6/2)"), "#t");
        assert_eq!(do_lisp("(>= 1.6 3/2)"), "#t");
        assert_eq!(do_lisp("(>= 3/4 4/8)"), "#t");

        assert_eq!(do_lisp("(>= 3 6/2)"), "#t");
        assert_eq!(do_lisp("(>= 0.5 1/2)"), "#t");
        assert_eq!(do_lisp("(>= 6/2 3)"), "#t");
        assert_eq!(do_lisp("(>= 3/2 1.5)"), "#t");
        assert_eq!(do_lisp("(>= 4/8 2/4)"), "#t");
    }
}
