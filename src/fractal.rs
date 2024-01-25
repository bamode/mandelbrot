use std::fs::File;
use std::str::FromStr;

use image::{ColorType, ImageResult};
use image::png::PngEncoder;
use num::Complex;

use crate::color::{Color, color};

/// Try to determine if `c` is in the Julia set, using at most `limit`
/// iterations to decide.
///
/// If `c` is not a member, return `Some(i)`, where `i` is the number of
/// iterations it took for `c` to leave the circle of radius 2 centered on the
/// origin. If `c` seems to be a member (more precisely, if we reached the
/// iteration limit without being able to prove that `c` is not a member),
/// return `None`.
fn escape_time_mandel(c: Complex<f64>, limit: usize) -> Option<usize> {
    let mut z: Complex<f64> = Complex{ re: 0.0, im: 0.0 };
    for i in 0..limit {
        if z.norm_sqr() >= 4. {
            return Some(i)
        }
        z = z * z + c;
    }
    None
}

fn escape_time_burningship(c: Complex<f64>, limit: usize) -> Option<usize> {
    let mut z: Complex<f64> = Complex{ re: 0.0, im: 0.0 };
    for i in 0..limit {
        if z.norm_sqr() >= 4. {
            return Some(i)
        }
        z = Complex { re: z.re.abs(), im: -z.im.abs() }.powi(2) + c;
    }
    None
}

/// Try to determine if `c` is in the Julia set, using at most `limit`
/// iterations to decide.
///
/// If `c` is not a member, return `Some(i)`, where `i` is the number of
/// iterations it took for `c` to leave the circle of radius 2 centered on the
/// origin. If `c` seems to be a member (more precisely, if we reached the
/// iteration limit without being able to prove that `c` is not a member),
/// return `None`.
fn escape_time_julia(z: Complex<f64>, c: Complex<f64>, limit: usize) -> Option<usize> {
    let mut z = z;
    for i in 0..limit {
        if z.norm_sqr() >= 4. {
            return Some(i)
        }
        z = z * z + c;
    }
    None
}

/// Parse the string `s` as a coordinate pair, like `"400x600"` or `"1.0,0.5"`.
///
/// Specifically, `s` should have the form <left><sep><right>, where <sep> is
/// the character given by the `separator` argument, and <left> and <right> are
/// both strings that can be parsed by `T::from_str`. `separator` must be an
/// ASCII character.
///
/// If `s` has the proper form, return `Some<(x, y)>`. If it doesn't parse /// correctly, return `None`.
pub fn parse_pair<T: FromStr>(s: &str, separator: char) -> Option<(T, T)> {
    s.find(separator).and_then(|index| {
        match (T::from_str(&s[..index]), T::from_str(&s[index + 1..])) {
            (Ok(l), Ok(r)) => Some((l, r)),
            _ => None,
        }
    })
}

/// Parse a pair of floating-point numbers separated by a comma as a complex 
/// number.
pub fn parse_complex(s: &str) -> Option<Complex<f64>> {
    parse_pair(s, ',').and_then(|(re, im)| Some(Complex { re, im }))
}

/// Given the row and column of a pixel in the output image, return the
/// corresponding point on the complex plane.
///
/// `bounds` is a pair giving the width and height of the image in pixels.
/// `pixel` is a (column, row) pair indicating a particular pixel in that image.
/// The `upper_left` and `lower_right` parameters are points on the complex
/// plane designating the area our image covers.
pub fn pixel_to_point(bounds: (usize, usize),
                  pixel: (usize, usize),
                  upper_left: Complex<f64>,
                  lower_right: Complex<f64>) -> Complex<f64> {
    let (width, height) = (lower_right.re - upper_left.re, upper_left.im - lower_right.im);
    Complex {
        re: upper_left.re + pixel.0 as f64 * width / bounds.0 as f64,
        im: upper_left.im - pixel.1 as f64 * height / bounds.1 as f64
    }
}

/// Render a rectangle of the Julia set into a buffer of pixels.
/// 
/// The `bounds` argument gives the width and height of the buffer `pixels`,
/// which holds one grayscale pixel per byte. the `upper_left` and `lower_right`
/// arguments specify points on the complex plane corresponding to the upper-
/// left and lower-right corners of the pixel buffer. 
pub fn render_julia(pixels: &mut [u8],
          bounds: (usize, usize),
          upper_left: Complex<f64>,
          lower_right: Complex<f64>,
          seed: Complex<f64>,
          colors: &[Color]) {
    assert!(pixels.len() == bounds.0 * 3 * bounds.1);
    for row in 0..bounds.1 {
        for col in (0..bounds.0 * 3).step_by(3) {
            let point = pixel_to_point(bounds, (col / 3, row), upper_left, lower_right);
            let pix = row * bounds.0 * 3 + col;
            let cur_color: Color = match escape_time_julia(point, seed, 255) {
                None => Color(0, 0, 0),
                Some(count) => color(colors, count),
            };
            match cur_color {
                Color(r, g, b) => {
                    pixels[pix] = r;
                    pixels[pix+1] = g;
                    pixels[pix+2] = b;
                }
            }
        }
    }
}

/// Render a rectangle of the Mandelbrot set into a buffer of pixels.
/// 
/// The `bounds` argument gives the width and height of the buffer `pixels`,
/// which holds one grayscale pixel per byte. the `upper_left` and `lower_right`
/// arguments specify points on the complex plane corresponding to the upper-
/// left and lower-right corners of the pixel buffer. 
pub fn render_mandel(pixels: &mut [u8],
                     bounds: (usize, usize),
                     upper_left: Complex<f64>,
                     lower_right: Complex<f64>,
                     colors: &[Color]) {
    assert!(pixels.len() == bounds.0 * 3 * bounds.1);
    for row in 0..bounds.1 {
        for col in (0..bounds.0 * 3).step_by(3) {
            let point = pixel_to_point(bounds, (col / 3, row), upper_left, lower_right);
            let pix = row * bounds.0 * 3 + col;
            let cur_color: Color = match escape_time_mandel(point, 255) {
                None => Color(0, 0, 0),
                Some(count) => color(colors, count),
            };
            match cur_color {
                Color(r, g, b) => {
                    pixels[pix] = r;
                    pixels[pix+1] = g;
                    pixels[pix+2] = b;
                }
            }
        }
    }
}

pub fn render_burningship(pixels: &mut [u8],
                     bounds: (usize, usize),
                     upper_left: Complex<f64>,
                     lower_right: Complex<f64>,
                     colors: &[Color]) {
    assert!(pixels.len() == bounds.0 * 3 * bounds.1);
    for row in 0..bounds.1 {
        for col in (0..bounds.0 * 3).step_by(3) {
            let point = pixel_to_point(bounds, (col / 3, row), upper_left, lower_right);
            let pix = row * bounds.0 * 3 + col;
            let cur_color: Color = match escape_time_burningship(point, 255) {
                None => Color(0, 0, 0),
                Some(count) => color(colors, count),
            };
            match cur_color {
                Color(r, g, b) => {
                    pixels[pix] = r;
                    pixels[pix+1] = g;
                    pixels[pix+2] = b;
                }
            }
        }
    }
                     }

/// Write the buffer `pixels`, whose dimensions are given by `bounds`, to the
/// file named `filename`.
pub fn write_image(pixels: &[u8], bounds: (usize, usize), filename: &str) -> ImageResult<()> {
    let output = File::create(filename)?;
    let encoder = PngEncoder::new(output);
    encoder.encode(&pixels, bounds.0 as u32, bounds.1 as u32, ColorType::Rgb8)?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_pair() {
        assert_eq!(parse_pair::<i32>("", ','), None);
        assert_eq!(parse_pair::<i32>("10,", ','), None);
        assert_eq!(parse_pair::<i32>(",10", ','), None);
        assert_eq!(parse_pair::<i32>("10,20", ','), Some((10, 20)));
        assert_eq!(parse_pair::<f64>("0.5x", 'x'), None);
        assert_eq!(parse_pair::<f64>("0.5x1.5", 'x'), Some((0.5, 1.5)));
        assert_eq!(parse_pair::<usize>("1920x1080", 'x'), Some((1920, 1080)));
    }

    #[test]
    fn test_parse_complex() {
        assert_eq!(parse_complex("0.5,-10.32"), Some(Complex { re: 0.5, im: -10.32 }));
        assert_eq!(parse_complex("0.2,"), None);
    }
}