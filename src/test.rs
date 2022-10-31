use crate::pixel_to_complex;
use crate::ComplexPoint;
use crate::wraping_add;

#[test]
fn complex_mul() {
    let p1 = ComplexPoint::new(3, 2);
    let p2 = ComplexPoint::new(1, 7);
    let answer = ComplexPoint::new(-11, 23);
    assert_eq!(p1.mul(&p2).re, answer.re);
    assert_eq!(p1.mul(&p2).im, answer.im);
}

#[test]
fn complex_add() {
    let p1 = ComplexPoint::new(5, 2);
    let p2 = ComplexPoint::new(7, 12);
    let answer = ComplexPoint::new(12, 14);
    assert_eq!(p1.add(&p2).re, answer.re);
    assert_eq!(p1.add(&p2).im, answer.im);
}

#[test]
fn pixel_complex() {
    let pixel = (400, 150);
    let dimensions = (2000, 1000);
    let lower_left = ComplexPoint::new(2.5, 2.5);
    let upper_right = ComplexPoint::new(5.0, 5.0);
    let answer = ComplexPoint::new(lower_left.re + 0.5, lower_left.im + 0.375);
    let complex = pixel_to_complex(pixel, dimensions, &lower_left, &upper_right);
    assert_eq!(complex.re, answer.re);
    assert_eq!(complex.im, answer.im);
}

#[test]
fn test_wraping_add() {
    assert_eq!(wraping_add(5, 9, 10), 4);
    assert_eq!(wraping_add(5, 3, 10), 8);
    assert_eq!(wraping_add(8, 15, 10), 3);
}
