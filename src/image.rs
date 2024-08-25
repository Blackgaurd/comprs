use std::{
    ops::{Add, Div, Mul, Sub},
    vec,
};

use image::ImageReader;

use crate::psa::{PrefixSum2D, Zero};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RGB<T> {
    pub r: T,
    pub g: T,
    pub b: T,
}

impl<T> RGB<T> {
    pub fn new(r: T, g: T, b: T) -> Self {
        RGB { r, g, b }
    }
}

impl<T: Mul<Output = T> + Clone + Copy> RGB<T> {
    fn comp_prod(&self, other: Self) -> Self {
        Self::new(self.r * other.r, self.g * other.g, self.b * other.b)
    }
}

impl From<RGB<u8>> for RGB<u64> {
    fn from(value: RGB<u8>) -> Self {
        RGB::new(value.r.into(), value.g.into(), value.b.into())
    }
}

impl<T: Add<Output = T>> Add for RGB<T> {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.r + rhs.r, self.g + rhs.g, self.b + rhs.b)
    }
}

impl<T: Sub<Output = T>> Sub for RGB<T> {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.r - rhs.r, self.g - rhs.g, self.b - rhs.b)
    }
}

impl<T> Div<T> for RGB<T>
where
    T: Div<T, Output = T> + Clone + Copy,
{
    type Output = Self;
    fn div(self, rhs: T) -> Self::Output {
        Self::new(self.r / rhs, self.g / rhs, self.b / rhs)
    }
}

impl Zero for RGB<u64> {
    fn zero() -> Self {
        Self::new(0, 0, 0)
    }
}

pub struct ImageData {
    height: usize,
    width: usize,
    sums: PrefixSum2D<RGB<u64>>,
    square_sums: PrefixSum2D<RGB<u64>>,
}

impl ImageData {
    pub fn new(data: &Vec<Vec<RGB<u64>>>) -> Result<Self, String> {
        let sums = PrefixSum2D::new(&data)?;
        let squares = data
            .into_iter()
            .map(|row| row.into_iter().map(|x| x.comp_prod(*x)).collect())
            .collect();
        let square_sums = PrefixSum2D::new(&squares)?;
        Ok(Self {
            height: sums.height(),
            width: sums.width(),
            sums,
            square_sums,
        })
    }

    pub fn from_path(path: &String) -> Result<Self, String> {
        let Ok(img) = ImageReader::open(path) else {
            return Err("unable to open image".into());
        };
        let Ok(decoded) = img.decode() else {
            return Err("unable to decode image".into());
        };
        let Some(colors) = decoded.as_rgb8() else {
            return Err("unable to convert image to RGB8".into());
        };

        let (w, h) = colors.dimensions();
        let mut data = vec![vec![RGB::zero(); w as usize]; h as usize];
        for (x, y, pixel) in colors.enumerate_pixels() {
            let rgb = RGB::new(pixel[0], pixel[1], pixel[2]);
            data[y as usize][x as usize] = rgb.into();
        }

        Self::new(&data)
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn sum(&self, top_left: (usize, usize), bottom_right: (usize, usize)) -> RGB<u64> {
        self.sums.query_sum(top_left, bottom_right)
    }

    pub fn average(&self, top_left: (usize, usize), bottom_right: (usize, usize)) -> RGB<u64> {
        let height = (bottom_right.0 - top_left.0 + 1) as u64;
        let width = (bottom_right.1 - top_left.1 + 1) as u64;
        /* println!(
            "calc average, sum: {:?}, pixels: {}",
            self.sum(top_left, bottom_right),
            (height * width)
        ); */
        self.sum(top_left, bottom_right) / (height * width)
    }

    pub fn metric(&self, top_left: (usize, usize), bottom_right: (usize, usize)) -> u64 {
        let height = (bottom_right.0 - top_left.0 + 1) as u64;
        let width = (bottom_right.1 - top_left.1 + 1) as u64;

        let mean = self.sum(top_left, bottom_right) / (height * width);
        let square_sum = self.square_sums.query_sum(top_left, bottom_right);

        let variance = square_sum / (height * width) - mean.comp_prod(mean);
        (variance.r + variance.g + variance.b) * (height * width)
    }
}
