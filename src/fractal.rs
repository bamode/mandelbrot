use num::Complex;

/// Try to determine if `c` is in the Julia set, using at most `limit`
/// iterations to decide.
///
/// If `c` is not a member, return `Some(i)`, where `i` is the number of
/// iterations it took for `c` to leave the circle of radius 2 centered on the
/// origin. If `c` seems to be a member (more precisely, if we reached the
/// iteration limit without being able to prove that `c` is not a member),
/// return `None`.
pub fn escape_time_mandel(c: Complex<f64>, limit: usize) -> Option<usize> {
    let mut z: Complex<f64> = Complex{ re: 0.0, im: 0.0 };
    for i in 0..limit {
        if z.norm_sqr() >= 4. {
            return Some(i)
        }
        z = z * z + c;
    }
    None
}

pub fn escape_time_burningship(c: Complex<f64>, limit: usize) -> Option<usize> {
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
pub fn escape_time_julia(z: Complex<f64>, c: Complex<f64>, limit: usize) -> Option<usize> {
    let mut z = z;
    for i in 0..limit {
        if z.norm_sqr() >= 4. {
            return Some(i)
        }
        z = z * z + c;
    }
    None
}