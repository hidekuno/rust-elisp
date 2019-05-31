/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
use std::cmp::Ordering;
use std::cmp::PartialEq;
use std::cmp::PartialOrd;
use std::ops::Add;
use std::ops::Div;
use std::ops::Mul;
use std::ops::Sub;

#[allow(unused_imports)]
use log::{debug, error, info, warn};

//========================================================================
#[derive(Debug, Copy, Clone)]
pub struct Rat {
    pub numer: i64,
    pub denom: i64,
}
impl Rat {
    pub fn new(n: i64, d: i64) -> Rat {
        let l = gcm(n, d);
        Rat {
            numer: n / l,
            denom: d / l,
        }
    }
    pub fn to_string(&self) -> String {
        if self.denom == 1 {
            self.numer.to_string()
        } else {
            format!("{}/{}", self.numer, self.denom)
        }
    }
    pub fn div_float(&self) -> f64 {
        self.numer as f64 / self.denom as f64
    }
}
fn rat_add(x: Rat, y: Rat) -> Rat {
    Rat::new((x.numer * y.denom) + (y.numer * x.denom), x.denom * y.denom)
}
fn rat_sub(x: Rat, y: Rat) -> Rat {
    Rat::new((x.numer * y.denom) - (y.numer * x.denom), x.denom * y.denom)
}
fn rat_mul(x: Rat, y: Rat) -> Rat {
    Rat::new(x.numer * y.numer, x.denom * y.denom)
}
fn rat_div(x: Rat, y: Rat) -> Rat {
    Rat::new(x.numer * y.denom, x.denom * y.numer)
}
fn rat_eq(x: Rat, y: Rat) -> bool {
    return (x.numer == y.numer) && (x.denom == y.denom);
}
fn rat_lt(x: Rat, y: Rat) -> bool {
    return (x.numer * y.denom) < (y.numer * x.denom);
}
fn rat_le(x: Rat, y: Rat) -> bool {
    return (x.numer * y.denom) <= (y.numer * x.denom);
}
fn rat_gt(x: Rat, y: Rat) -> bool {
    return (x.numer * y.denom) > (y.numer * x.denom);
}
fn rat_ge(x: Rat, y: Rat) -> bool {
    return (x.numer * y.denom) >= (y.numer * x.denom);
}

pub fn gcm(n: i64, m: i64) -> i64 {
    match n % m {
        0 => m.wrapping_abs(),
        l => gcm(m, l),
    }
}
struct Calc {
    fcalc: fn(f64, f64) -> f64,
    icalc: fn(i64, i64) -> i64,
    rcalc: fn(Rat, Rat) -> Rat,
}
impl Calc {
    fn new(f: fn(f64, f64) -> f64, i: fn(i64, i64) -> i64, r: fn(Rat, Rat) -> Rat) -> Calc {
        Calc {
            fcalc: f,
            icalc: i,
            rcalc: r,
        }
    }
}
struct Cmp {
    fop: fn(f64, f64) -> bool,
    iop: fn(i64, i64) -> bool,
    rop: fn(Rat, Rat) -> bool,
}
impl Cmp {
    fn new(f: fn(f64, f64) -> bool, i: fn(i64, i64) -> bool, r: fn(Rat, Rat) -> bool) -> Cmp {
        Cmp {
            fop: f,
            iop: i,
            rop: r,
        }
    }
}
#[derive(Debug, Copy, Clone)]
pub enum Number {
    Integer(i64),
    Float(f64),
    Rational(Rat),
}
impl Number {
    fn calc(self: Number, other: Number, operator: Calc) -> Number {
        match self {
            Number::Integer(a) => match other {
                Number::Integer(b) => Number::Integer((operator.icalc)(a, b)),
                Number::Float(b) => Number::Float((operator.fcalc)(a as f64, b)),
                Number::Rational(b) => Number::Rational((operator.rcalc)(Rat::new(a, 1), b)),
            },
            Number::Float(a) => match other {
                Number::Integer(b) => Number::Float((operator.fcalc)(a, b as f64)),
                Number::Float(b) => Number::Float((operator.fcalc)(a, b)),
                Number::Rational(b) => Number::Float((operator.fcalc)(a, b.div_float())),
            },
            Number::Rational(a) => match other {
                Number::Integer(b) => Number::Rational((operator.rcalc)(a, Rat::new(b, 1))),
                Number::Float(b) => Number::Float((operator.fcalc)(a.div_float(), b)),
                Number::Rational(b) => Number::Rational((operator.rcalc)(a, b)),
            },
        }
    }
    fn cmp(self: Number, other: Number, operator: Cmp) -> bool {
        match self {
            Number::Integer(a) => match other {
                Number::Integer(b) => (operator.iop)(a, b),
                Number::Float(b) => (operator.fop)(a as f64, b),
                Number::Rational(b) => (operator.rop)(Rat::new(a, 1), b),
            },
            Number::Float(a) => match other {
                Number::Integer(b) => (operator.fop)(a, b as f64),
                Number::Float(b) => (operator.fop)(a, b),
                Number::Rational(b) => (operator.fop)(a, b.div_float()),
            },
            Number::Rational(a) => match other {
                Number::Integer(b) => (operator.rop)(a, Rat::new(b, 1)),
                Number::Float(b) => (operator.fop)(a.div_float(), b),
                Number::Rational(b) => (operator.rop)(a, b),
            },
        }
    }
}
//impl<T: Add<Output=T>> Add for Number<T> {
impl Add for Number {
    type Output = Number;

    fn add(self, other: Number) -> Number {
        return self.calc(
            other,
            Calc::new(|x: f64, y: f64| x + y, |x: i64, y: i64| x + y, rat_add),
        );
    }
}
impl Sub for Number {
    type Output = Number;
    fn sub(self, other: Number) -> Number {
        return self.calc(
            other,
            Calc::new(|x: f64, y: f64| x - y, |x: i64, y: i64| x - y, rat_sub),
        );
    }
}
impl Mul for Number {
    type Output = Number;
    fn mul(self, other: Number) -> Number {
        return self.calc(
            other,
            Calc::new(|x: f64, y: f64| x * y, |x: i64, y: i64| x * y, rat_mul),
        );
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
                return self.calc(
                    Number::Rational(Rat::new(y, 1)),
                    Calc::new(|x: f64, y: f64| x / y, |x: i64, y: i64| x / y, rat_div),
                );
            }
        }
        return self.calc(
            other,
            Calc::new(|x: f64, y: f64| x / y, |x: i64, y: i64| x / y, rat_div),
        );
    }
}
impl PartialEq for Number {
    fn eq(&self, other: &Number) -> bool {
        return self.cmp(
            *other,
            Cmp::new(|x: f64, y: f64| x == y, |x: i64, y: i64| x == y, rat_eq),
        );
    }
}
impl PartialOrd for Number {
    fn lt(&self, other: &Number) -> bool {
        return self.cmp(
            *other,
            Cmp::new(|x: f64, y: f64| x < y, |x: i64, y: i64| x < y, rat_lt),
        );
    }
    fn le(&self, other: &Number) -> bool {
        return self.cmp(
            *other,
            Cmp::new(|x: f64, y: f64| x <= y, |x: i64, y: i64| x <= y, rat_le),
        );
    }
    fn gt(&self, other: &Number) -> bool {
        return self.cmp(
            *other,
            Cmp::new(|x: f64, y: f64| x > y, |x: i64, y: i64| x > y, rat_gt),
        );
    }
    fn ge(&self, other: &Number) -> bool {
        return self.cmp(
            *other,
            Cmp::new(|x: f64, y: f64| x >= y, |x: i64, y: i64| x >= y, rat_ge),
        );
    }
    fn partial_cmp(&self, _: &Number) -> Option<Ordering> {
        // This is same as nop
        Some(Ordering::Equal)
    }
}
#[test]
fn test_gcm() {
    assert_eq!(gcm(17, 2), 1);
    assert_eq!(gcm(36, 27), 9);
    assert_eq!(gcm(27, 36), 9);
    assert_eq!(gcm(-27, 36), 9);
    assert_eq!(gcm(27, -36), 9);
}
