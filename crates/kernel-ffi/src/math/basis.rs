use crate::RgmStatus;

pub(crate) fn find_span(
    n: usize,
    degree: usize,
    u: f64,
    knots: &[f64],
) -> Result<usize, RgmStatus> {
    if degree > n {
        return Err(RgmStatus::InvalidInput);
    }
    if knots.len() < n + degree + 2 {
        return Err(RgmStatus::InvalidInput);
    }

    let u_low = knots[degree];
    let u_high = knots[n + 1];

    if u <= u_low {
        return Ok(degree);
    }
    if u >= u_high {
        return Ok(n);
    }

    let mut low = degree;
    let mut high = n + 1;
    let mut mid = (low + high) / 2;

    while u < knots[mid] || u >= knots[mid + 1] {
        if u < knots[mid] {
            high = mid;
        } else {
            low = mid;
        }
        mid = (low + high) / 2;
    }

    Ok(mid)
}

#[allow(dead_code)]
pub(crate) fn basis_funs(
    span: usize,
    u: f64,
    degree: usize,
    knots: &[f64],
) -> Result<Vec<f64>, RgmStatus> {
    let mut n = vec![0.0_f64; degree + 1];
    let mut left = vec![0.0_f64; degree + 1];
    let mut right = vec![0.0_f64; degree + 1];

    n[0] = 1.0;

    for j in 1..=degree {
        left[j] = u - knots[span + 1 - j];
        right[j] = knots[span + j] - u;
        let mut saved = 0.0;

        for r in 0..j {
            let denom = right[r + 1] + left[j - r];
            let temp = if denom.abs() <= f64::EPSILON {
                0.0
            } else {
                n[r] / denom
            };
            n[r] = saved + right[r + 1] * temp;
            saved = left[j - r] * temp;
        }
        n[j] = saved;
    }

    Ok(n)
}

pub(crate) fn ders_basis_funs(
    span: usize,
    u: f64,
    degree: usize,
    order: usize,
    knots: &[f64],
) -> Result<Vec<Vec<f64>>, RgmStatus> {
    let order = order.min(degree);

    let mut ndu = vec![vec![0.0_f64; degree + 1]; degree + 1];
    let mut left = vec![0.0_f64; degree + 1];
    let mut right = vec![0.0_f64; degree + 1];

    ndu[0][0] = 1.0;

    for j in 1..=degree {
        left[j] = u - knots[span + 1 - j];
        right[j] = knots[span + j] - u;
        let mut saved = 0.0;

        for r in 0..j {
            ndu[j][r] = right[r + 1] + left[j - r];
            let temp = if ndu[j][r].abs() <= f64::EPSILON {
                0.0
            } else {
                ndu[r][j - 1] / ndu[j][r]
            };
            ndu[r][j] = saved + right[r + 1] * temp;
            saved = left[j - r] * temp;
        }

        ndu[j][j] = saved;
    }

    let mut ders = vec![vec![0.0_f64; degree + 1]; order + 1];
    for (j, value) in ders[0].iter_mut().enumerate().take(degree + 1) {
        *value = ndu[j][degree];
    }

    let mut a = vec![vec![0.0_f64; degree + 1]; 2];

    for r in 0..=degree {
        let mut s1 = 0_usize;
        let mut s2 = 1_usize;
        a[0][0] = 1.0;

        for k in 1..=order {
            let mut d = 0.0;
            let rk = r as isize - k as isize;
            let pk = degree as isize - k as isize;

            if r >= k {
                let denom = ndu[(pk + 1) as usize][rk as usize];
                a[s2][0] = if denom.abs() <= f64::EPSILON {
                    0.0
                } else {
                    a[s1][0] / denom
                };
                d = a[s2][0] * ndu[rk as usize][pk as usize];
            }

            let j1 = if rk >= -1 { 1 } else { (-rk) as usize };
            let j2 = if (r as isize - 1) <= pk {
                k - 1
            } else {
                degree - r
            };

            for j in j1..=j2 {
                let denom = ndu[(pk + 1) as usize][(rk + j as isize) as usize];
                a[s2][j] = if denom.abs() <= f64::EPSILON {
                    0.0
                } else {
                    (a[s1][j] - a[s1][j - 1]) / denom
                };
                d += a[s2][j] * ndu[(rk + j as isize) as usize][pk as usize];
            }

            if (r as isize) <= pk {
                let denom = ndu[(pk + 1) as usize][r];
                a[s2][k] = if denom.abs() <= f64::EPSILON {
                    0.0
                } else {
                    -a[s1][k - 1] / denom
                };
                d += a[s2][k] * ndu[r][pk as usize];
            }

            ders[k][r] = d;
            std::mem::swap(&mut s1, &mut s2);
        }
    }

    let mut factor = degree as f64;
    for k in 1..=order {
        for j in 0..=degree {
            ders[k][j] *= factor;
        }
        factor *= (degree - k) as f64;
    }

    Ok(ders)
}
