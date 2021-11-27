use std::fs::File;

use image::{ColorType, ImageResult, png::PngEncoder};
use num::Complex;
use rayon::prelude::*;

use crate::{color::{Color, color},
            fractal::{escape_time_burningship, escape_time_julia, escape_time_mandel}};

/// Ok, let's document our plans for this eh,
/// 
/// TODO:
/// Want to figure out a nice idiomatic way to express rendering an image
/// and then writing it to a file. The obvious approach would be to
/// define some fractal types that implement `Render` and `WriteImage` traits,
/// that provide `render()` and `write_image()` respectively. Then, 
/// perhaps let's consider the types `BurningShip`, `Julia`, and `Mandel`
/// that all implement `Render` and `WriteImage`. Then they can provide
/// `create_julia` and `create_mandel` as a public interface.
/// 
/// 
/// FIXME:
/// Hmm, bit of a snag I think. I don't think I can declare a `Julia` outside the
/// scope of the parallelized closure that I can then act on `Julia`. I think
/// maybe instead I need to implement `WriteFractalImage` (yes, that would be a 
/// rename, this is more specific), for the pixels: a `&[u8]` slice of the rendered image.

pub fn create_julia(file: &str,
    bounds: Bounds<usize>, 
    upper_left: Complex<f64>, 
    lower_right: Complex<f64>,
    seed: Complex<f64>,
    colors: [Color; 2048]) {

    let mut pixels = vec![0; (bounds.0 * 3) * bounds.1]; // * 3 for rgb

    let bands: Vec<(usize, &mut [u8])> = pixels.chunks_mut(bounds.0 * 3).enumerate().collect();
    bands.into_par_iter().for_each(|(i, band)| {
        let top = i;
        let band_bounds: Bounds<usize> = Bounds(bounds.0, 1);
        let band_upper_left = pixel_to_point(bounds, (0, top), upper_left, lower_right);
        let band_lower_right = pixel_to_point(bounds, (bounds.0, top + 1), upper_left, lower_right);
        let julia = Julia::new(band_bounds, band_upper_left, band_lower_right, seed, &colors);
        julia.render(band);
    });
    write_image(&pixels, bounds, file).expect("error writing PNG file");
}


pub fn create_mandel(file: &str,
    bounds: Bounds<usize>, 
    upper_left: Complex<f64>, 
    lower_right: Complex<f64>,
    colors: [Color; 2048],
    altfn: bool) {

    let mut pixels = vec![0; (bounds.0 * 3) * bounds.1]; // * 3 for rgb

    let bands: Vec<(usize, &mut [u8])> = pixels.chunks_mut(bounds.0 * 3).enumerate().collect();
    bands.into_par_iter().for_each(|(i, band)| {
        let top = i;
        let band_bounds: Bounds<usize> = Bounds(bounds.0, 1);
        let band_upper_left = pixel_to_point(bounds, (0, top), upper_left, lower_right);
        let band_lower_right = pixel_to_point(bounds, (bounds.0, top + 1), upper_left, lower_right);
        if altfn {
            let burning_ship = BurningShip::new(band_bounds, band_upper_left, band_lower_right, &colors);
            burning_ship.render(band);
        } else {
            let mandel = Mandel::new(band_bounds, band_upper_left, band_lower_right, &colors);
            mandel.render(band);
        }
    });
    write_image(&pixels, bounds, file).expect("error writing PNG file");
}
/// Given the row and column of a pixel in the output image, return the
/// corresponding point on the complex plane.
///
/// `bounds` is a pair giving the width and height of the image in pixels.
/// `pixel` is a (column, row) pair indicating a particular pixel in that image.
/// The `upper_left` and `lower_right` parameters are points on the complex
/// plane designating the area our image covers.
pub fn pixel_to_point(bounds: Bounds<usize>,
                  pixel: (usize, usize),
                  upper_left: Complex<f64>,
                  lower_right: Complex<f64>) -> Complex<f64> {
    let (width, height) = (lower_right.re - upper_left.re, upper_left.im - lower_right.im);
    Complex {
        re: upper_left.re + pixel.0 as f64 * width / bounds.0 as f64,
        im: upper_left.im - pixel.1 as f64 * height / bounds.1 as f64
    }
}

#[derive(Debug)]
pub struct Mandel<'m> {
    bounds: Bounds<usize>, 
    upper_left: Complex<f64>, 
    lower_right: Complex<f64>,
    colors: &'m [Color; 2048],
}

impl<'m> Mandel<'m> {
    pub fn new(bounds: Bounds<usize>, 
               upper_left: Complex<f64>, 
               lower_right: Complex<f64>, 
               colors: &'m [Color; 2048]) -> Self {
        Mandel { bounds, upper_left, lower_right, colors }
    }
}

impl<'m> Render for Mandel<'m> {
    fn render(&self, pixels: &mut [u8]) {
        assert!(pixels.len() == self.bounds.0 * 3 * self.bounds.1);
        for row in 0..self.bounds.1 {
            for col in (0..self.bounds.0 * 3).step_by(3) {
                let point = pixel_to_point(self.bounds, (col / 3, row), self.upper_left, self.lower_right);
                let pix = row * self.bounds.0 * 3 + col;
                let cur_color: Color = match escape_time_mandel(point, 255) {
                    None => Color(0, 0, 0),
                    Some(count) => color(self.colors, count),
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
}

#[derive(Debug)]
pub struct BurningShip<'bs> {
    bounds: Bounds<usize>, 
    upper_left: Complex<f64>, 
    lower_right: Complex<f64>,
    colors: &'bs [Color; 2048],
}

impl<'bs> BurningShip<'bs> {
    pub fn new(bounds: Bounds<usize>,
               upper_left: Complex<f64>,
               lower_right: Complex<f64>,
               colors: &'bs [Color; 2048]) -> Self {
        BurningShip { bounds, upper_left, lower_right, colors }
    }
}

impl<'bs> Render for BurningShip<'bs> {
    fn render(&self, pixels: &mut [u8]) {
        assert!(pixels.len() == self.bounds.0 * 3 * self.bounds.1);
        for row in 0..self.bounds.1 {
            for col in (0..self.bounds.0 * 3).step_by(3) {
                let point = pixel_to_point(self.bounds, (col / 3, row), self.upper_left, self.lower_right);
                let pix = row * self.bounds.0 * 3 + col;
                let cur_color: Color = match escape_time_burningship(point, 255) {
                    None => Color(0, 0, 0),
                    Some(count) => color(self.colors, count),
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
}

#[derive(Debug)]
pub struct Julia<'j> {
    bounds: Bounds<usize>,
    upper_left: Complex<f64>,
    lower_right: Complex<f64>,
    seed: Complex<f64>,
    colors: &'j [Color],
}

impl<'j> Julia<'j> { 
    pub fn new(bounds: Bounds<usize>,
               upper_left: Complex<f64>,
               lower_right: Complex<f64>,
               seed: Complex<f64>,
               colors: &'j [Color]) -> Self {
        Julia { bounds, upper_left, lower_right, seed, colors }           
    }
 }

 impl<'j> Render for Julia<'j> {
     fn render(&self, pixels: &mut [u8]) {
         assert!(pixels.len() == self.bounds.0 * 3 * self.bounds.1);
         for row in 0..self.bounds.1 {
             for col in (0..self.bounds.0 * 3).step_by(3) {
                 let point = pixel_to_point(self.bounds, (col / 3, row), self.upper_left, self.lower_right);
                 let pix = row * self.bounds.0 * 3 + col;
                 let cur_color: Color = match escape_time_julia(point,
                                                                self.seed,
                                                                255) {
                     None => Color(0, 0, 0),
                     Some(count) => color(self.colors, count),
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
 }

trait Render {
    fn render(&self, pixels: &mut [u8]);
}

#[derive(Copy, Clone, Debug)]
pub struct Bounds<T: Sized>(pub T, pub T);
pub fn write_image(pixels: &[u8], bounds: Bounds<usize>, filename: &str) -> ImageResult<()> {
    let output = File::create(filename)?;
    let encoder = PngEncoder::new(output);
    encoder.encode(&pixels, bounds.0 as u32, bounds.1 as u32, ColorType::Rgb8)?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::args::{parse_complex, parse_pair};

    #[test]
    fn test_parse_pair() {
        assert_eq!(parse_pair::<i32>("", ','), None);
        assert_eq!(parse_pair::<i32>("10,", ','), None);
        assert_eq!(parse_pair::<i32>(",10", ','), None);
        assert_eq!(parse_pair::<i32>("10,20", ','), Some((10, 20)));
        assert_eq!(parse_pair::<f64>("0.5x", 'x'), None);
        assert_eq!(parse_pair::<f64>("0.5x1.5", 'x'), Some((0.5, 1.5)));
    }

    #[test]
    fn test_parse_complex() {
        assert_eq!(parse_complex("0.5,-10.32"), Some(Complex { re: 0.5, im: -10.32 }));
        assert_eq!(parse_complex("0.2,"), None);
    }
}