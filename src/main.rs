use clap::{App, Arg, ArgMatches};
use num::Complex;
use rayon::prelude::*;

mod color;
mod fractal;
mod monocub;

use fractal::{parse_complex, parse_pair, pixel_to_point, render_julia, render_mandel, render_burningship, write_image};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches: ArgMatches = App::new("mandelbrot")
                          .version("1.1.1")
                          .author("Brent Mode <bmode@wisc.edu")
                          .about("creates mandelbrot and julia set images")
                          .subcommand(App::new("julia")
                                      .about("creates a julia set image")
                                      .arg(Arg::new("FILE")
                                           .about("Set the file name\nEx: julia.png")
                                           .required(true))
                                      .arg(Arg::new("PIXELS")
                                           .about("Set the image size\nEx: 1000x1000")
                                           .required(true))
                                      .arg(Arg::new("UPPERLEFT")
                                           .about("Set the upper left corner of the complex plane\nEx: -2.0,2.0    (-2 + 2i)")
                                           .required(true))
                                      .arg(Arg::new("LOWERRIGHT")
                                           .about("Set the lower right corner of the complex plane\nEx: 2.0,-2.0    (2 - 2i)")
                                           .required(true))
                                      .arg(Arg::new("COLORSCHEME")
                                           .short('c')
                                           .long("color")
                                           .about(&*(format!("Set the color scheme from {}", color::COLORLIST)))
                                           .takes_value(true)
                                           .required(false))
                                      .arg(Arg::new("SEED")
                                           .short('s')
                                           .long("seed")
                                           .about("Set the seed for the Julia set image\nEx: -0.4,0.6    (-0.4 + 0.6i)")
                                           .takes_value(true)
                                           .required(false))
                                      .after_help("Full example:\nmandelbrot julia --color=vaportest --seed=-0.4,0.6 -- julia.png 5000x5000 -2,2 2,-2"))
                          .subcommand(App::new("mandel")
                                      .about("creates a mandelbrot set image")
                                      .arg(Arg::new("FILE")
                                           .about("Set the file name\nEx: mandel.png")
                                           .required(true))
                                      .arg(Arg::new("PIXELS")
                                           .about("Set the image size\nEx: 1000x750")
                                           .required(true))
                                      .arg(Arg::new("UPPERLEFT")
                                           .about("Set the upper left corner of the complex plane\nEx: -1.20,0.35    (-1.20 + 0.35i)")
                                           .required(true))
                                      .arg(Arg::new("LOWERRIGHT")
                                           .about("Set the lower right corner of the complex plane\nEx: -1,0.20    (-1 + 0.2i)")
                                           .required(true))
                                      .arg(Arg::new("COLORSCHEME")
                                           .short('c')
                                           .long("color")
                                           .about(&*(format!("Set the color scheme from:\n{}", color::COLORLIST)))
                                           .takes_value(true)
                                           .required(false))
                                      .arg(Arg::new("ALTFN")
                                           .short('a')
                                           .long("altfn")
                                           .about("For now, just uses the burning ship fractal in place of the Mandelbrot fractal")
                                           .required(false))
                                      .after_help("Full example:\nmandelbrot mandel --color=vaportest --altfn -- bs.png 5000x5000 -2,2 2,-2"))
                          .after_help("Full example:\nmandelbrot julia --color=vaportest --seed=-0.4,0.6 -- julia.png 5000x5000 -2,2 2,-2").get_matches();

    if let Some(ref matches) = matches.subcommand_matches("julia") {
        let (file, bounds, upper_left, lower_right, colors) = common_args(*matches);

        let seed: Complex<f64>;
        if let Some(seed_str) = matches.value_of("SEED") {
            seed = parse_complex(seed_str).unwrap();
        } else {
            seed = Complex{ re: 0.4, im: 0.6 };
        }

        create_julia(file, bounds, upper_left, lower_right, seed, colors);
    } else if let Some(ref matches) = matches.subcommand_matches("mandel") {
        let (file, bounds, upper_left, lower_right, colors) = common_args(*matches);
        if matches.occurrences_of("ALTFN") > 0 {
            create_mandel(file, bounds, upper_left, lower_right, colors, true);
        } else {
            create_mandel(file, bounds, upper_left, lower_right, colors, false);
        }
    }

    Ok(())
}

fn common_args<'a>(matches: &'a ArgMatches) -> (&'a str, (usize, usize), Complex<f64>, Complex<f64>, [color::Color; 2048]) {
    let file: &'a str = matches.value_of("FILE").unwrap();
    let bounds: (usize, usize) = parse_pair(matches
        .value_of("PIXELS")
        .unwrap(), 'x')
        .expect("error parsing image dimensions");
    let ratio: f64 = bounds.0 as f64 / bounds.1 as f64; // x / y
    let upper_left = parse_complex(matches
        .value_of("UPPERLEFT").unwrap())
        .expect("error parsing upper left corner point");
    let lower_right = parse_complex(matches
        .value_of("LOWERRIGHT").unwrap())
        .expect("error parsing lower right corner point");
    
    let plane_ratio = (lower_right.re - upper_left.re) / (upper_left.im - lower_right.im);
    if ratio != plane_ratio {
       unimplemented!() 
    }
    let colors: [color::Color; 2048];
    if let Some(color) = matches.value_of("COLORSCHEME") {
        colors = color::colors(color).unwrap();
    } else {
        colors = color::colors("wikipedia").unwrap();
    }

    (file, bounds, upper_left, lower_right, colors)
}

pub fn create_julia(file: &str,
                    bounds: (usize, usize), 
                    upper_left: Complex<f64>, 
                    lower_right: Complex<f64>,
                    seed: Complex<f64>,
                    colors: [color::Color; 2048],) {
    
    let mut pixels = vec![0; (bounds.0 * 3) * bounds.1]; // * 3 for rgb

    let bands: Vec<(usize, &mut [u8])> = pixels.chunks_mut(bounds.0 * 3).enumerate().collect();
    bands.into_par_iter().for_each(|(i, band)| {
        let top = i;
        let band_bounds = (bounds.0, 1);
        let band_upper_left = pixel_to_point(bounds, (0, top), upper_left, lower_right);
        let band_lower_right = pixel_to_point(bounds, (bounds.0, top + 1), upper_left, lower_right);
        render_julia(band, band_bounds, band_upper_left, band_lower_right, seed, &colors);
    });
    write_image(&pixels, bounds, file).expect("error writing PNG file");
}


pub fn create_mandel(file: &str,
                    bounds: (usize, usize), 
                    upper_left: Complex<f64>, 
                    lower_right: Complex<f64>,
                    colors: [color::Color; 2048],
                    altfn: bool) {
    
    let mut pixels = vec![0; (bounds.0 * 3) * bounds.1]; // * 3 for rgb

    let bands: Vec<(usize, &mut [u8])> = pixels.chunks_mut(bounds.0 * 3).enumerate().collect();
    bands.into_par_iter().for_each(|(i, band)| {
        let top = i;
        let band_bounds = (bounds.0, 1);
        let band_upper_left = pixel_to_point(bounds, (0, top), upper_left, lower_right);
        let band_lower_right = pixel_to_point(bounds, (bounds.0, top + 1), upper_left, lower_right);
        if altfn {
            render_burningship(band, band_bounds, band_upper_left, band_lower_right, &colors);
        } else {
            render_mandel(band, band_bounds, band_upper_left, band_lower_right, &colors);
        }
    });
    write_image(&pixels, bounds, file).expect("error writing PNG file");
}
