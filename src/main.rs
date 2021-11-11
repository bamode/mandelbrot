use std::env;

use rayon::prelude::*;

mod color;
mod fractal;
mod monocub;

use color::{Color, colors};
use fractal::*;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 6 {
        eprintln!("Usage: {} FILE PIXELS UPPERLEFT LOWERRIGHT COLORSCHEME", args[0]);
        eprintln!("Example: {} mandel.png 1000x750 -1.20,0.35 -1,0.20 viridis", args[0]);
        std::process::exit(1);
    }

    let bounds: (usize, usize) = parse_pair(&args[2], 'x').expect("error parsing image dimensions");
    let upper_left = parse_complex(&args[3]).expect("error parsing upper left corner point");
    let lower_right = parse_complex(&args[4]).expect("error parsing lower right corner point");
    let colors: [Color; 2048] = colors(&args[5]).unwrap();

    let mut pixels = vec![0; (bounds.0 * 3) * bounds.1]; // * 3 for rgb

    let bands: Vec<(usize, &mut [u8])> = pixels.chunks_mut(bounds.0 * 3).enumerate().collect();
    bands.into_par_iter().for_each(|(i, band)| {
        let top = i;
        let band_bounds = (bounds.0, 1);
        let band_upper_left = pixel_to_point(bounds, (0, top), upper_left, lower_right);
        let band_lower_right = pixel_to_point(bounds, (bounds.0, top + 1), upper_left, lower_right);
        render(band, band_bounds, band_upper_left, band_lower_right, &colors);
    });
    write_image(&pixels, bounds, &args[1]).expect("error writing PNG file");
}