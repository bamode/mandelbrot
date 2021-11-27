use clap::{App, Arg, ArgMatches};
use num::Complex;

mod args;
mod color;
mod fractal;
mod monocub;
mod render;

use args::{common_args, parse_complex};
use render::{create_julia, create_mandel};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches: ArgMatches = App::new("mandelbrot")
                          .version("1.2.1")
                          .author("Brent Mode <bmode@wisc.edu>")
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