use std::{
    fmt::Debug,
    ops::{Add, Sub},
};

pub trait Zero {
    fn zero() -> Self;
}

/// 2D prefix sum array for fast range sum queries
pub struct PrefixSum2D<T>
where
    T: Add<Output = T> + Sub<Output = T> + Zero + Clone + Copy,
{
    height: usize,
    width: usize,
    data: Vec<Vec<T>>,
}

impl<T> PrefixSum2D<T>
where
    T: Add<Output = T> + Sub<Output = T> + Zero + Clone + Copy + Debug,
{
    pub fn new(arr: &Vec<Vec<T>>) -> Result<Self, String> {
        let height = arr.len();
        let width = match arr.first() {
            Some(f) => f.len(),
            None => return Err("array has height 0".into()),
        };
        if width == 0 {
            return Err("array has width 0".into());
        }

        let mut data = vec![vec![T::zero(); width + 1]; height + 1];
        for i in 0..height {
            for j in 0..width {
                data[i + 1][j + 1] = arr[i][j] + data[i][j + 1] + data[i + 1][j] - data[i][j];
            }
        }

        Ok(Self {
            height,
            width,
            data,
        })
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn width(&self) -> usize {
        self.width
    }

    /// get the sum of values from top_left to bottom_right (inclusive)
    pub fn query_sum(&self, top_left: (usize, usize), bottom_right: (usize, usize)) -> T {
        let a = self.data[bottom_right.0 + 1][bottom_right.1 + 1];
        let b = self.data[top_left.0][top_left.1];
        let c = self.data[bottom_right.0 + 1][top_left.1];
        let d = self.data[top_left.0][bottom_right.1 + 1];

        a + b - c - d
    }
}
