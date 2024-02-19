use std::ops::Div;
use num_traits::Float;

pub struct Scalar { value: f32 }

#[derive(Debug, PartialEq)]
struct Vector { value: Vec<f32> }

impl Div<Scalar> for Vector {
    type Output = Self;

    fn div(self, rhs: Scalar) -> Self::Output {
        Self { value: self.value.iter().map(|v| v / rhs.value).collect() }
    }
}


pub fn linspace<T: Float>(start: T, end: T, n: usize) -> Vec<T> {
    let dx = (end - start) / T::from(n - 1).unwrap();
    (0..n).map(|i| start + T::from(i).unwrap() * dx).collect()
}