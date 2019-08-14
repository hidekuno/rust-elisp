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
use std::string::ToString;

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
    pub fn div_float(&self) -> f64 {
        self.numer as f64 / self.denom as f64
    }
}
fn gcm(n: i64, m: i64) -> i64 {
    match n % m {
        0 => m.wrapping_abs(),
        l => gcm(m, l),
    }
}
impl ToString for Rat {
    fn to_string(&self) -> String {
        if self.denom == 1 {
            self.numer.to_string()
        } else {
            format!("{}/{}", self.numer, self.denom)
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
    fn partial_cmp(&self, _: &Rat) -> Option<Ordering> {
        // This is same as nop
        Some(Ordering::Equal)
    }
}
#[derive(Debug, Copy, Clone)]
pub enum Number {
    Integer(i64),
    Float(f64),
    Rational(Rat),
}
impl Number {
    fn calc<I, F, R, V>(self: Number, other: Number, icalc: I, fcalc: F, rcalc: R) -> V
    where
        I: Fn(i64, i64) -> V,
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
}
//impl<T: Add<Output=T>> Add for Number<T> {
impl Add for Number {
    type Output = Number;

    fn add(self, other: Number) -> Number {
        return self
            .calc::<fn(i64, i64) -> Number, fn(f64, f64) -> Number, fn(Rat, Rat) -> Number, Number>(
                other,
                |x: i64, y: i64| Number::Integer(x + y),
                |x: f64, y: f64| Number::Float(x + y),
                |x: Rat, y: Rat| Number::Rational(x + y),
            );
    }
}
impl Sub for Number {
    type Output = Number;
    fn sub(self, other: Number) -> Number {
        return self
            .calc::<fn(i64, i64) -> Number, fn(f64, f64) -> Number, fn(Rat, Rat) -> Number, Number>(
                other,
                |x: i64, y: i64| Number::Integer(x - y),
                |x: f64, y: f64| Number::Float(x - y),
                |x: Rat, y: Rat| Number::Rational(x - y),
            );
    }
}
impl Mul for Number {
    type Output = Number;
    fn mul(self, other: Number) -> Number {
        return self
            .calc::<fn(i64, i64) -> Number, fn(f64, f64) -> Number, fn(Rat, Rat) -> Number, Number>(
                other,
                |x: i64, y: i64| Number::Integer(x * y),
                |x: f64, y: f64| Number::Float(x * y),
                |x: Rat, y: Rat| Number::Rational(x * y),
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
                return self
                    .calc::<fn(i64, i64) -> Number, fn(f64, f64) -> Number, fn(Rat, Rat) -> Number, Number>(
                        Number::Rational(Rat::new(y, 1)),
                        |x: i64, y: i64| Number::Integer(x / y),
                        |x: f64, y: f64| Number::Float(x / y),
                        |x: Rat, y: Rat| Number::Rational(x / y),
                    );
            }
        }
        return self
            .calc::<fn(i64, i64) -> Number, fn(f64, f64) -> Number, fn(Rat, Rat) -> Number, Number>(
                other,
                |x: i64, y: i64| Number::Integer(x / y),
                |x: f64, y: f64| Number::Float(x / y),
                |x: Rat, y: Rat| Number::Rational(x / y),
            );
    }
}
impl PartialEq for Number {
    fn eq(&self, other: &Number) -> bool {
        return self
            .calc::<fn(i64, i64) -> bool, fn(f64, f64) -> bool, fn(Rat, Rat) -> bool, bool>(
                *other,
                |x: i64, y: i64| x == y,
                |x: f64, y: f64| x == y,
                |x: Rat, y: Rat| x == y,
            );
    }
}
impl PartialOrd for Number {
    fn lt(&self, other: &Number) -> bool {
        return self
            .calc::<fn(i64, i64) -> bool, fn(f64, f64) -> bool, fn(Rat, Rat) -> bool, bool>(
                *other,
                |x: i64, y: i64| x < y,
                |x: f64, y: f64| x < y,
                |x: Rat, y: Rat| x < y,
            );
    }
    fn le(&self, other: &Number) -> bool {
        return self
            .calc::<fn(i64, i64) -> bool, fn(f64, f64) -> bool, fn(Rat, Rat) -> bool, bool>(
                *other,
                |x: i64, y: i64| x <= y,
                |x: f64, y: f64| x <= y,
                |x: Rat, y: Rat| x <= y,
            );
    }
    fn gt(&self, other: &Number) -> bool {
        return self
            .calc::<fn(i64, i64) -> bool, fn(f64, f64) -> bool, fn(Rat, Rat) -> bool, bool>(
                *other,
                |x: i64, y: i64| x > y,
                |x: f64, y: f64| x > y,
                |x: Rat, y: Rat| x > y,
            );
    }
    fn ge(&self, other: &Number) -> bool {
        return self
            .calc::<fn(i64, i64) -> bool, fn(f64, f64) -> bool, fn(Rat, Rat) -> bool, bool>(
                *other,
                |x: i64, y: i64| x >= y,
                |x: f64, y: f64| x >= y,
                |x: Rat, y: Rat| x >= y,
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
#[test]
fn test_add_integer() {
    match Number::Integer(2) + Number::Integer(3) {
        Number::Integer(v) => assert_eq!(v, 5),
        _ => panic!("test failure"),
    }
    match Number::Integer(1) + Number::Rational(Rat::new(1, 2)) {
        Number::Rational(v) => {
            assert_eq!(v.numer, 3);
            assert_eq!(v.denom, 2);
        }
        _ => panic!("test failure"),
    }
    match Number::Integer(2) + Number::Float(2.5) {
        Number::Float(v) => assert_eq!(v, 4.5),
        _ => panic!("test failure"),
    }
}
#[test]
fn test_add_float() {
    match Number::Float(1.5) + Number::Integer(3) {
        Number::Float(v) => assert_eq!(v, 4.5),
        _ => panic!("test failure"),
    }
    match Number::Float(1.5) + Number::Float(1.25) {
        Number::Float(v) => assert_eq!(v, 2.75),
        _ => panic!("test failure"),
    }
    match Number::Float(2.5) + Number::Rational(Rat::new(1, 4)) {
        Number::Float(v) => assert_eq!(v, 2.75),
        _ => panic!("test failure"),
    }
}
#[test]
fn test_add_rational() {
    match Number::Rational(Rat::new(3, 4)) + Number::Integer(1) {
        Number::Rational(v) => {
            assert_eq!(v.numer, 7);
            assert_eq!(v.denom, 4);
        }
        _ => panic!("test failure"),
    }
    match Number::Rational(Rat::new(1, 4)) + Number::Float(2.5) {
        Number::Float(v) => assert_eq!(v, 2.75),
        _ => panic!("test failure"),
    }
    match Number::Rational(Rat::new(3, 4)) + Number::Rational(Rat::new(1, 3)) {
        Number::Rational(v) => {
            assert_eq!(v.numer, 13);
            assert_eq!(v.denom, 12);
        }
        _ => panic!("test failure"),
    }
}
#[test]
fn test_sub_integer() {
    match Number::Integer(10) - Number::Integer(3) {
        Number::Integer(v) => assert_eq!(v, 7),
        _ => panic!("test failure"),
    }
    match Number::Integer(1) - Number::Rational(Rat::new(1, 2)) {
        Number::Rational(v) => {
            assert_eq!(v.numer, 1);
            assert_eq!(v.denom, 2);
        }
        _ => panic!("test failure"),
    }
    match Number::Integer(1) - Number::Float(2.5) {
        Number::Float(v) => assert_eq!(v, -1.5),
        _ => panic!("test failure"),
    }
}
#[test]
fn test_sub_float() {
    match Number::Float(4.5) - Number::Integer(3) {
        Number::Float(v) => assert_eq!(v, 1.5),
        _ => panic!("test failure"),
    }
    match Number::Float(1.5) - Number::Float(1.25) {
        Number::Float(v) => assert_eq!(v, 0.25),
        _ => panic!("test failure"),
    }
    match Number::Float(2.5) - Number::Rational(Rat::new(1, 4)) {
        Number::Float(v) => assert_eq!(v, 2.25),
        _ => panic!("test failure"),
    }
}
#[test]
fn test_sub_rational() {
    match Number::Rational(Rat::new(1, 2)) - Number::Integer(1) {
        Number::Rational(v) => {
            assert_eq!(v.numer, -1);
            assert_eq!(v.denom, 2);
        }
        _ => panic!("test failure"),
    }
    match Number::Rational(Rat::new(3, 4)) - Number::Float(0.5) {
        Number::Float(v) => assert_eq!(v, 0.25),
        _ => panic!("test failure"),
    }
    match Number::Rational(Rat::new(3, 4)) - Number::Rational(Rat::new(1, 2)) {
        Number::Rational(v) => {
            assert_eq!(v.numer, 1);
            assert_eq!(v.denom, 4);
        }
        _ => panic!("test failure"),
    }
}
#[test]
fn test_mul_integer() {
    match Number::Integer(10) * Number::Integer(3) {
        Number::Integer(v) => assert_eq!(v, 30),
        _ => panic!("test failure"),
    }
    match Number::Integer(3) * Number::Rational(Rat::new(1, 2)) {
        Number::Rational(v) => {
            assert_eq!(v.numer, 3);
            assert_eq!(v.denom, 2);
        }
        _ => panic!("test failure"),
    }
    match Number::Integer(1) * Number::Float(2.5) {
        Number::Float(v) => assert_eq!(v, 2.5),
        _ => panic!("test failure"),
    }
}
#[test]
fn test_mul_float() {
    match Number::Float(4.5) * Number::Integer(3) {
        Number::Float(v) => assert_eq!(v, 13.5),
        _ => panic!("test failure"),
    }
    match Number::Float(1.8) * Number::Float(1.8) {
        Number::Float(v) => assert_eq!(v, 3.24),
        _ => panic!("test failure"),
    }
    match Number::Float(2.5) * Number::Rational(Rat::new(1, 4)) {
        Number::Float(v) => assert_eq!(v, 0.625),
        _ => panic!("test failure"),
    }
}
#[test]
fn test_mul_rational() {
    match Number::Rational(Rat::new(1, 2)) * Number::Integer(3) {
        Number::Rational(v) => {
            assert_eq!(v.numer, 3);
            assert_eq!(v.denom, 2);
        }
        _ => panic!("test failure"),
    }
    match Number::Rational(Rat::new(3, 4)) * Number::Float(0.5) {
        Number::Float(v) => assert_eq!(v, 0.375),
        _ => panic!("test failure"),
    }
    match Number::Rational(Rat::new(3, 4)) * Number::Rational(Rat::new(1, 2)) {
        Number::Rational(v) => {
            assert_eq!(v.numer, 3);
            assert_eq!(v.denom, 8);
        }
        _ => panic!("test failure"),
    }
}
#[test]
fn test_div_integer() {
    match Number::Integer(8) / Number::Integer(2) {
        Number::Integer(v) => assert_eq!(v, 4),
        _ => panic!("test failure"),
    }
    match Number::Integer(3) / Number::Rational(Rat::new(1, 2)) {
        Number::Rational(v) => {
            assert_eq!(v.numer, 6);
            assert_eq!(v.denom, 1);
        }
        _ => panic!("test failure"),
    }
    match Number::Integer(1) / Number::Float(2.5) {
        Number::Float(v) => assert_eq!(v, 0.4),
        _ => panic!("test failure"),
    }
}
#[test]
fn test_div_float() {
    match Number::Float(4.5) / Number::Integer(3) {
        Number::Float(v) => assert_eq!(v, 1.5),
        _ => panic!("test failure"),
    }
    match Number::Float(3.6) / Number::Float(3.2) {
        Number::Float(v) => assert_eq!(v, 1.125),
        _ => panic!("test failure"),
    }
    match Number::Float(2.5) / Number::Rational(Rat::new(1, 3)) {
        Number::Float(v) => assert_eq!(v, 7.5),
        _ => panic!("test failure"),
    }
}
#[test]
fn test_div_rational() {
    match Number::Rational(Rat::new(1, 2)) / Number::Integer(3) {
        Number::Rational(v) => {
            assert_eq!(v.numer, 1);
            assert_eq!(v.denom, 6);
        }
        _ => panic!("test failure"),
    }
    match Number::Rational(Rat::new(3, 4)) / Number::Float(0.5) {
        Number::Float(v) => assert_eq!(v, 1.5),
        _ => panic!("test failure"),
    }
    match Number::Rational(Rat::new(3, 4)) / Number::Rational(Rat::new(1, 2)) {
        Number::Rational(v) => {
            assert_eq!(v.numer, 3);
            assert_eq!(v.denom, 2);
        }
        _ => panic!("test failure"),
    }
}
#[test]
fn test_eq_rational() {
    assert_eq!(Number::Integer(3) == Number::Rational(Rat::new(6, 2)), true);
    assert_eq!(Number::Float(0.5) == Number::Rational(Rat::new(1, 2)), true);

    assert_eq!(Number::Rational(Rat::new(6, 2)) == Number::Integer(3), true);
    assert_eq!(Number::Rational(Rat::new(3, 2)) == Number::Float(1.5), true);
    assert_eq!(
        Number::Rational(Rat::new(4, 8)) == Number::Rational(Rat::new(2, 4)),
        true
    );
}

#[test]
fn test_lt_rational() {
    assert_eq!(Number::Integer(3) < Number::Rational(Rat::new(7, 2)), true);
    assert_eq!(Number::Float(0.3) < Number::Rational(Rat::new(1, 2)), true);

    assert_eq!(Number::Rational(Rat::new(6, 2)) < Number::Integer(4), true);
    assert_eq!(Number::Rational(Rat::new(3, 2)) < Number::Float(1.6), true);
    assert_eq!(
        Number::Rational(Rat::new(4, 8)) < Number::Rational(Rat::new(3, 4)),
        true
    );
}

#[test]
fn test_le_rational() {
    assert_eq!(Number::Integer(3) <= Number::Rational(Rat::new(7, 2)), true);
    assert_eq!(Number::Float(0.3) <= Number::Rational(Rat::new(1, 2)), true);
    assert_eq!(Number::Rational(Rat::new(6, 2)) <= Number::Integer(4), true);
    assert_eq!(Number::Rational(Rat::new(3, 2)) <= Number::Float(1.6), true);
    assert_eq!(
        Number::Rational(Rat::new(4, 8)) <= Number::Rational(Rat::new(3, 4)),
        true
    );

    assert_eq!(Number::Integer(3) <= Number::Rational(Rat::new(6, 2)), true);
    assert_eq!(Number::Float(0.5) <= Number::Rational(Rat::new(1, 2)), true);
    assert_eq!(Number::Rational(Rat::new(6, 2)) <= Number::Integer(3), true);
    assert_eq!(Number::Rational(Rat::new(3, 2)) <= Number::Float(1.5), true);
    assert_eq!(
        Number::Rational(Rat::new(4, 8)) <= Number::Rational(Rat::new(2, 4)),
        true
    );
}

#[test]
fn test_gt_rational() {
    assert_eq!(Number::Rational(Rat::new(7, 2)) > Number::Integer(3), true);
    assert_eq!(Number::Rational(Rat::new(1, 2)) > Number::Float(0.3), true);

    assert_eq!(Number::Integer(4) > Number::Rational(Rat::new(6, 2)), true);
    assert_eq!(Number::Float(1.6) > Number::Rational(Rat::new(3, 2)), true);
    assert_eq!(
        Number::Rational(Rat::new(3, 4)) > Number::Rational(Rat::new(4, 8)),
        true
    );
}
#[test]
fn test_ge_rational() {
    assert_eq!(Number::Rational(Rat::new(7, 2)) >= Number::Integer(3), true);
    assert_eq!(Number::Rational(Rat::new(1, 2)) >= Number::Float(0.3), true);
    assert_eq!(Number::Integer(4) >= Number::Rational(Rat::new(6, 2)), true);
    assert_eq!(Number::Float(1.6) >= Number::Rational(Rat::new(3, 2)), true);
    assert_eq!(
        Number::Rational(Rat::new(3, 4)) >= Number::Rational(Rat::new(4, 8)),
        true
    );

    assert_eq!(Number::Integer(3) >= Number::Rational(Rat::new(6, 2)), true);
    assert_eq!(Number::Float(0.5) >= Number::Rational(Rat::new(1, 2)), true);
    assert_eq!(Number::Rational(Rat::new(6, 2)) >= Number::Integer(3), true);
    assert_eq!(Number::Rational(Rat::new(3, 2)) >= Number::Float(1.5), true);
    assert_eq!(
        Number::Rational(Rat::new(4, 8)) >= Number::Rational(Rat::new(2, 4)),
        true
    );
}
