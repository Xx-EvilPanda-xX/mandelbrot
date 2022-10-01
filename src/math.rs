use std::ops::{Add, Mul, Sub};

pub struct ComplexPoint<T> {
    pub re: T,
    pub im: T,
}

impl<T> ComplexPoint<T>
where
    T: Mul<Output = T> + Add<Output = T> + Sub<Output = T> + Copy,
{
    pub fn new(re: T, im: T) -> ComplexPoint<T> {
        ComplexPoint { re, im }
    }

    pub fn mul(&self, other: &ComplexPoint<T>) -> ComplexPoint<T> {
        ComplexPoint {
            re: self.re * other.re - self.im * other.im,
            im: self.re * other.im + self.im * other.re,
        }
    }

}

impl<T> ComplexPoint<T>
    where T: Add<Output = T> + Copy
{
    pub fn add(&self, other: &ComplexPoint<T>) -> ComplexPoint<T> {
        ComplexPoint {
            re: self.re + other.re,
            im: self.im + other.im,
        }
    }
}

impl<T> Clone for ComplexPoint<T>
where
    T: Mul<Output = T> + Add<Output = T> + Sub<Output = T> + Copy,
{
    fn clone(&self) -> Self {
        ComplexPoint {
            re: self.re,
            im: self.im,
        }
    }
}
