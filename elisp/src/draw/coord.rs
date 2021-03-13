/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   hidekuno@gmail.com
*/
use std::ops::Add;
use std::ops::Div;
use std::ops::Mul;
use std::ops::Sub;

#[derive(Debug, Copy, Clone)]
pub struct Coord<T> {
    pub x: T,
    pub y: T,
}
impl<T> Coord<T> {
    pub fn new(x: T, y: T) -> Self {
        Coord { x: x, y: y }
    }
    pub fn scale(&self, s: T) -> Coord<T>
    where
        T: Copy + Mul + Mul<Output = T>,
    {
        Coord::new(self.x * s, self.y * s)
    }
    pub fn sum(&self) -> T
    where
        T: Copy + Add + Add<Output = T>,
    {
        self.x + self.y
    }
}

impl<T> ToString for Coord<T>
where
    T: std::fmt::Display,
{
    fn to_string(&self) -> String {
        format!("{}/{}", self.x, self.y)
    }
}
impl<T> Add for Coord<T>
where
    T: Add + Add<Output = T>,
{
    type Output = Coord<T>;
    fn add(self: Coord<T>, other: Coord<T>) -> Coord<T> {
        Coord::new(self.x + other.x, self.y + other.y)
    }
}
impl<T> Sub for Coord<T>
where
    T: Sub + Sub<Output = T>,
{
    type Output = Coord<T>;
    fn sub(self: Coord<T>, other: Coord<T>) -> Coord<T> {
        Coord::new(self.x - other.x, self.y - other.y)
    }
}
impl<T> Mul for Coord<T>
where
    T: Mul + Mul<Output = T>,
{
    type Output = Coord<T>;
    fn mul(self: Coord<T>, other: Coord<T>) -> Coord<T> {
        Coord::new(self.x * other.x, self.y * other.y)
    }
}
impl<T> Div for Coord<T>
where
    T: Div + Div<Output = T>,
{
    type Output = Coord<T>;

    fn div(self: Coord<T>, other: Coord<T>) -> Coord<T> {
        Coord::new(self.x / other.x, self.y / other.y)
    }
}
#[cfg(test)]
mod tests {
    use crate::draw::coord::Coord;

    #[test]
    fn add() {
        let a = Coord::<f64>::new(1.0, 1.0);
        let b = Coord::<f64>::new(0.5, 0.25);
        let v = a + b;
        assert_eq!(v.x, 1.5);
        assert_eq!(v.y, 1.25);
    }
    #[test]
    fn sub() {
        let a = Coord::<f64>::new(1.0, 1.0);
        let b = Coord::<f64>::new(0.5, 0.25);
        let v = a - b;
        assert_eq!(v.x, 0.5);
        assert_eq!(v.y, 0.75);
    }
    #[test]
    fn mul() {
        let a = Coord::<f64>::new(1.0, 1.0);
        let b = Coord::<f64>::new(0.5, 0.25);

        let v = a * b;
        assert_eq!(v.x, 0.5);
        assert_eq!(v.y, 0.25);
    }
    #[test]
    fn div() {
        let a = Coord::<f64>::new(1.0, 2.0);
        let b = Coord::<f64>::new(2.0, 2.0);

        let v = a / b;
        assert_eq!(v.x, 0.5);
        assert_eq!(v.y, 1.0);
    }

    #[test]
    fn scale() {
        let a = Coord::<f64>::new(1.0, 2.0);
        let v = a.scale(3.0);
        assert_eq!(v.x, 3.0);
        assert_eq!(v.y, 6.0);
    }
    #[test]
    fn sum() {
        let a = Coord::<f64>::new(1.0, 2.0);
        let v = a.sum();
        assert_eq!(v, 3.0);
    }
}
