/*
   Rust study program.
   This is prototype program mini scheme subset what porting from go-scheme.

   this is library for glis,wasmlisp.

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
        format!("({},{})", self.x, self.y)
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
// scalar
impl<T> Add<T> for Coord<T>
where
    T: Copy + Add + Add<Output = T>,
{
    type Output = Coord<T>;
    fn add(self: Coord<T>, other: T) -> Coord<T> {
        Coord::new(self.x + other, self.y + other)
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
// scalar
impl<T> Sub<T> for Coord<T>
where
    T: Copy + Sub + Sub<Output = T>,
{
    type Output = Coord<T>;
    fn sub(self: Coord<T>, other: T) -> Coord<T> {
        Coord::new(self.x - other, self.y - other)
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
// scalar
impl<T> Mul<T> for Coord<T>
where
    T: Copy + Mul + Mul<Output = T>,
{
    type Output = Coord<T>;
    fn mul(self: Coord<T>, other: T) -> Coord<T> {
        Coord::new(self.x * other, self.y * other)
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
// scalar
impl<T> Div<T> for Coord<T>
where
    T: Copy + Div + Div<Output = T>,
{
    type Output = Coord<T>;
    fn div(self: Coord<T>, other: T) -> Coord<T> {
        Coord::new(self.x / other, self.y / other)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Matrix<T> {
    pub a: Coord<T>,
    pub b: Coord<T>,
}
impl<T> Matrix<T> {
    pub fn new(a: T, b: T, c: T, d: T) -> Self {
        Matrix {
            a: Coord::new(a, b),
            b: Coord::new(c, d),
        }
    }
    pub fn sum(&self) -> Coord<T>
    where
        T: Copy + Add + Add<Output = T>,
    {
        Coord::new(self.a.sum(), self.b.sum())
    }
}
impl<T> Mul<Coord<T>> for Matrix<T>
where
    T: Copy + Mul + Mul<Output = T>,
{
    type Output = Matrix<T>;
    fn mul(self: Matrix<T>, other: Coord<T>) -> Matrix<T> {
        Matrix::new(
            self.a.x * other.x,
            self.a.y * other.y,
            self.b.x * other.x,
            self.b.y * other.y,
        )
    }
}
impl<T> Mul<T> for Matrix<T>
where
    T: Copy + Mul + Mul<Output = T>,
{
    type Output = Matrix<T>;
    fn mul(self: Matrix<T>, other: T) -> Matrix<T> {
        Matrix::new(
            self.a.x * other,
            self.a.y * other,
            self.b.x * other,
            self.b.y * other,
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::draw::coord::Coord;
    use crate::draw::coord::Matrix;

    #[test]
    fn add() {
        let a = Coord::<f64>::new(1.0, 1.0);
        let b = Coord::<f64>::new(0.5, 0.25);
        let v = a + b;
        assert_eq!(v.x, 1.5);
        assert_eq!(v.y, 1.25);
    }
    #[test]
    fn add_scalar() {
        let a = Coord::<f64>::new(4.0, 5.0);
        let v = a + 3.0;
        assert_eq!(v.x, 7.0);
        assert_eq!(v.y, 8.0);
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
    fn sub_scalar() {
        let a = Coord::<f64>::new(4.0, 5.0);
        let v = a - 3.0;
        assert_eq!(v.x, 1.0);
        assert_eq!(v.y, 2.0);
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
    fn mul_scalar() {
        let a = Coord::<f64>::new(2.0, 3.0);
        let v = a * 3.0;
        assert_eq!(v.x, 6.0);
        assert_eq!(v.y, 9.0);
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
    fn div_scalar() {
        let a = Coord::<f64>::new(6.0, 3.0);
        let v = a / 3.0;
        assert_eq!(v.x, 2.0);
        assert_eq!(v.y, 1.0);
    }
    #[test]
    fn sum() {
        let a = Coord::<f64>::new(1.0, 2.0);
        let v = a.sum();
        assert_eq!(v, 3.0);
    }
    #[test]
    fn to_string() {
        let a = Coord::<f64>::new(1.0, 2.0);
        assert_eq!(a.to_string(), "(1,2)");
    }
    #[test]
    fn matrix_sum() {
        let a = Matrix::<f64>::new(1.0, 2.0, 3.0, 4.0);
        let v = a.sum();
        assert_eq!(v.x, 3.0);
        assert_eq!(v.y, 7.0);
    }
    #[test]
    fn matrix() {
        let a = Matrix::<f64>::new(1.0, 2.0, 3.0, 4.0);
        let b = Coord::<f64>::new(1.0, 2.0);
        let v = a * b;
        assert_eq!(v.a.x, 1.0);
        assert_eq!(v.a.y, 4.0);
        assert_eq!(v.b.x, 3.0);
        assert_eq!(v.b.y, 8.0);

        let v = a * 3.0;
        assert_eq!(v.a.x, 3.0);
        assert_eq!(v.a.y, 6.0);
        assert_eq!(v.b.x, 9.0);
        assert_eq!(v.b.y, 12.0);
    }
}
