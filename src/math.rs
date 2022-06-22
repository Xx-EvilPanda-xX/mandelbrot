use std::ops::{Mul, Add, Sub};

pub struct ComplexPoint<T: Mul + Add> {
    pub re: T,
    pub im: T
}

impl<T> ComplexPoint<T> 
    where T: Mul<Output = T> +
            Add<Output = T> +
            Sub<Output = T> +
            Copy
{
    pub fn new(re: T, im: T) -> ComplexPoint<T> {
        ComplexPoint { re, im }
    }

    pub fn mul(&self, other: &ComplexPoint<T>) -> ComplexPoint<T> {
        ComplexPoint { re: self.re * other.re - self.im * other.im,
            im: self.re * other.im + self.im * other.re }
    }

    pub fn add(&self, other: &ComplexPoint<T>) -> ComplexPoint<T> {
        ComplexPoint { re: self.re + other.re,
            im: self.im + other.im }
    }
}