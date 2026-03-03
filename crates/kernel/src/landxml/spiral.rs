use std::f64::consts::PI;

use super::types::{SpiralSegment, SpiralType};

fn base_phi(kind: SpiralType, t: f64) -> f64 {
    let u = t.clamp(0.0, 1.0);
    match kind {
        SpiralType::Biquadratic => u * u,
        SpiralType::Bloss => 3.0 * u * u - 2.0 * u * u * u,
        SpiralType::Clothoid => u,
        SpiralType::Cosine => 0.5 * (1.0 - (PI * u).cos()),
        SpiralType::Cubic => u * u * u,
        SpiralType::Sinusoid => (0.5 * PI * u).sin(),
        SpiralType::SineHalfWave => 0.5 * (1.0 - (PI * u).cos()),
        SpiralType::BiquadraticParabola => u * u * (2.0 - u),
        SpiralType::CubicParabola => u * u * u,
        SpiralType::JapaneseCubic => 3.0 * u * u - 2.0 * u * u * u,
        SpiralType::Radioid => u.sqrt(),
        SpiralType::WeinerBogen => u - (2.0 * PI * u).sin() / (2.0 * PI),
        SpiralType::RevBiquadratic => 1.0 - (1.0 - u) * (1.0 - u),
        SpiralType::RevBloss => {
            let v = 1.0 - u;
            1.0 - (3.0 * v * v - 2.0 * v * v * v)
        }
        SpiralType::RevCosine => {
            let v = 1.0 - u;
            1.0 - 0.5 * (1.0 - (PI * v).cos())
        }
        SpiralType::RevSinusoid => {
            let v = 1.0 - u;
            1.0 - (0.5 * PI * v).sin()
        }
    }
}

pub fn spiral_curvature(kind: SpiralType, k0: f64, k1: f64, length: f64, s: f64) -> f64 {
    let t = if length.abs() < 1e-12 {
        0.0
    } else {
        (s / length).clamp(0.0, 1.0)
    };
    k0 + (k1 - k0) * base_phi(kind, t)
}

#[derive(Clone, Copy, Debug)]
pub struct SpiralEvaluation {
    pub x_local: f64,
    pub y_local: f64,
    pub heading_rad: f64,
    pub curvature_per_m: f64,
}

pub fn evaluate_spiral_local(seg: &SpiralSegment, s_local: f64) -> SpiralEvaluation {
    let s_target = s_local.clamp(0.0, seg.length_m);
    if s_target <= 0.0 {
        return SpiralEvaluation {
            x_local: 0.0,
            y_local: 0.0,
            heading_rad: seg.start_heading_rad,
            curvature_per_m: seg.k0_per_m,
        };
    }

    let n = ((s_target / seg.length_m.max(1e-9)) * 192.0)
        .ceil()
        .max(24.0) as usize;
    let h = s_target / n as f64;

    let mut x: f64 = 0.0;
    let mut y: f64 = 0.0;
    let mut theta: f64 = 0.0;

    for i in 0..n {
        let s = i as f64 * h;

        let k1 = spiral_curvature(seg.spi_type, seg.k0_per_m, seg.k1_per_m, seg.length_m, s);
        let k2 = spiral_curvature(
            seg.spi_type,
            seg.k0_per_m,
            seg.k1_per_m,
            seg.length_m,
            s + 0.5 * h,
        );
        let k3 = k2;
        let k4 = spiral_curvature(
            seg.spi_type,
            seg.k0_per_m,
            seg.k1_per_m,
            seg.length_m,
            s + h,
        );

        let dx1 = theta.cos();
        let dy1 = theta.sin();

        let th2 = theta + 0.5 * h * k1;
        let dx2 = th2.cos();
        let dy2 = th2.sin();

        let th3 = theta + 0.5 * h * k2;
        let dx3 = th3.cos();
        let dy3 = th3.sin();

        let th4 = theta + h * k3;
        let dx4 = th4.cos();
        let dy4 = th4.sin();

        x += h * (dx1 + 2.0 * dx2 + 2.0 * dx3 + dx4) / 6.0;
        y += h * (dy1 + 2.0 * dy2 + 2.0 * dy3 + dy4) / 6.0;
        theta += h * (k1 + 2.0 * k2 + 2.0 * k3 + k4) / 6.0;
    }

    let k = spiral_curvature(
        seg.spi_type,
        seg.k0_per_m,
        seg.k1_per_m,
        seg.length_m,
        s_target,
    );

    SpiralEvaluation {
        x_local: x,
        y_local: y,
        heading_rad: seg.start_heading_rad + theta,
        curvature_per_m: k,
    }
}
