pub fn monotonic_cubic_preprocess(y: &[u8], knots: &[f64]) -> Vec<f64> {
    let n: usize = y.len();
    let mut slopes: Vec<f64> = Vec::new();
    for i in 0..n - 1 {
        slopes.push((y[i+1] as f64 - y[i] as f64) / (knots[i+1] - knots[i]));
    }
    let mut m: Vec<f64> = Vec::new();
    for k in 1..slopes.len() {
        if slopes[k-1] * slopes[k] < 0.0 {
            m.push(0.0);
        } else {
            m.push((slopes[k-1] + slopes[k]) / 2.0);
        }
    }
    m.insert(0, slopes[0]);
    m.push(slopes[n-2]);
    let mut ignores: Vec<usize> = Vec::new();
    for k in 0..n - 1 {
        if slopes[k] == 0.0 {
            m[k] = 0.0;
            m[k+1] = 0.0;
            ignores.push(k);
        }
    }
    let mut cur = 0;
    let mut i = 0;
    let mut to_do = Vec::new();
    while cur < ignores.len() {
        if i < ignores[cur] {
            to_do.push(i);
            i += 1;
        } else {
            i = ignores[cur] + 1;
            cur += 1;
        }
    }
    
    for k in to_do.into_iter() {
        let a = m[k] / slopes[k];
        let b = m[k+1] / slopes[k];
        if a < 0.0 {
            m[k] = 0.0;
        } else if  b < 0.0 {
            m[k+1] = 0.0;
        } else if a.powi(2) + b.powi(2) > 9.0 {
            let t = 3.0 / (a.powi(2) + b.powi(2)).sqrt();
            m[k] = t * a * slopes[k];
            m[k+1] = t * b * slopes[k];
        }
    }
    
    return Vec::from(m)
}

pub fn interpolate(x: f64, knots: &[f64], y: &[u8], m: &Vec<f64>) -> u8 {
    let n: usize = knots.len();
    if x >= knots[n - 1] {
        let k = n - 2;
        let delta = knots[k+1] - knots[k];
        let t = (x - knots[k+1]) / delta;
        return (y[k] as f64 * h00(t) + delta * m[k] * h10(t) + y[k+1] as f64 * h01(t) + delta * m[k+1] * h11(t)) as u8
    }
    for k in 0..n - 1 {
        if knots[k] <= x && x <= knots[k+1] {
            let delta = knots[k+1] - knots[k];
            let t = (x - knots[k]) / delta;
            return (y[k] as f64 * h00(t) + delta * m[k] * h10(t) + y[k+1] as f64 * h01(t) + delta * m[k+1] * h11(t)) as u8
        }
    }
    panic!("should be logically impossible")
}

#[inline]
fn h00(t: f64) -> f64 {
    2.0 * t.powi(3) - 3.0 * t.powi(2) + 1.0
}

#[inline]
fn h10(t: f64) -> f64 {
    t.powi(3) - 2.0 * t.powi(2) + t
}

#[inline]
fn h01(t: f64) -> f64 {
    -2.0 * t.powi(3) + 3.0 * t.powi(2)
}

#[inline]
fn h11(t: f64) -> f64 {
    t.powi(3) - t.powi(2)
}