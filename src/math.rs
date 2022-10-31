use std::ops::{Add, Mul, Sub};

pub struct ComplexPoint<T> {
    pub re: T,
    pub im: T,
}

impl<T> ComplexPoint<T> {
    pub fn new(re: T, im: T) -> ComplexPoint<T> {
        ComplexPoint { re, im }
    }
}

impl<T> ComplexPoint<T>
where
    T: Mul<Output = T> + Add<Output = T> + Sub<Output = T> + Clone,
{
    pub fn mul(&self, other: &ComplexPoint<T>) -> ComplexPoint<T> {
        let re = &self.re;
        let other_re = &other.re;
        let im = &self.im;
        let other_im = &other.im;
        ComplexPoint {
            re: re.clone() * other_re.clone() - im.clone() * other_im.clone(),
            im: re.clone() * other_im.clone() + im.clone() * other_re.clone(),
        }
    }

}

impl<T> ComplexPoint<T>
    where T: Add<Output = T> + Clone
{
    pub fn add(&self, other: &ComplexPoint<T>) -> ComplexPoint<T> {
        let re = &self.re;
        let other_re = &other.re;
        let im = &self.im;
        let other_im = &other.im;
        ComplexPoint {
            re: re.clone() + other_re.clone(),
            im: im.clone() + other_im.clone(),
        }
    }
}

impl<T: Clone> Clone for ComplexPoint<T> {
    fn clone(&self) -> Self {
        ComplexPoint {
            re: self.re.clone(),
            im: self.im.clone(),
        }
    }
}
