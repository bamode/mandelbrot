use std::str::FromStr;

use clap::ArgMatches;
use num::Complex;

use crate::{color, {render::Bounds}};

pub fn common_args<'a>(matches: &'a ArgMatches) -> (&'a str, Bounds<usize>, Complex<f64>, Complex<f64>, [color::Color; 2048]) {
    let file: &'a str = matches.value_of("FILE").unwrap();
    let bounds: Bounds<usize> = parse_pair(matches
        .value_of("PIXELS")
        .unwrap(), 'x')
        .expect("error parsing image dimensions");
    let mut ratio: f64 = bounds.0 as f64 / bounds.1 as f64; // x / y
    let mut upper_left = parse_complex(matches
        .value_of("UPPERLEFT").unwrap())
        .expect("error parsing upper left corner point");
    let mut lower_right = parse_complex(matches
        .value_of("LOWERRIGHT").unwrap())
        .expect("error parsing lower right corner point");
    
    let plane_ratio = (lower_right.re - upper_left.re) / (upper_left.im - lower_right.im);
    if ratio != plane_ratio {
        if ratio > 1.0 {
            ratio = 1.0 / ratio;
            upper_left.im = upper_left.im - ratio / 2.0;
            lower_right.im = lower_right.im + ratio / 2.0;
        } else { 
            upper_left.re = upper_left.re + ratio / 2.0;
            lower_right.re = lower_right.re - ratio / 2.0;
        }
        println!("NEW UPPERLEFT\t\t{}", upper_left);
        println!("NEW LOWERRIGHT\t   {}", lower_right);
    }
    // 300x400 : (-2, 2), (1, -1)
    // ratio = 300 / 400 = 0.75
    // plane_ratio = 3 / 3 = 1.0
    // cbound_y = 2 - (-1) = 3;
    // cur_x = 1 - (-2) = 3;
    // cbound_x = ratio * cbound_y = 0.75 * 3 = 2.25

    let colors: [color::Color; 2048];
    if let Some(color) = matches.value_of("COLORSCHEME") {
        colors = color::colors(color).unwrap();
    } else {
        colors = color::colors("wikipedia").unwrap();
    }

    (file, bounds, upper_left, lower_right, colors)
}

/// Parse the string `s` as a coordinate pair, like `"400x600"` or `"1.0,0.5"`.
///
/// Specifically, `s` should have the form <left><sep><right>, where <sep> is
/// the character given by the `separator` argument, and <left> and <right> are
/// both strings that can be parsed by `T::from_str`. `separator` must be an
/// ASCII character.
///
/// If `s` has the proper form, return `Some<(x, y)>`. If it doesn't parse /// correctly, return `None`.
pub fn parse_pair<T: FromStr>(s: &str, separator: char) -> Option<Bounds<T>> {
    s.find(separator).and_then(|index| {
        match (T::from_str(&s[..index]), T::from_str(&s[index + 1..])) {
            (Ok(l), Ok(r)) => Some(Bounds(l, r)),
            _ => None,
        }
    })
}

/// Parse a pair of floating-point numbers separated by a comma as a complex 
/// number.
pub fn parse_complex(s: &str) -> Option<Complex<f64>> {
    parse_pair(s, ',').and_then(|Bounds(re, im)| Some(Complex { re, im }))
}