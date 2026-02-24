use crate::math::vec3 as v3;
use crate::{
    RgmAabb3, RgmBounds3, RgmBoundsMode, RgmObb3, RgmPoint3, RgmStatus, RgmVec3,
};

#[inline]
fn dot_point_axis(point: RgmPoint3, axis: RgmVec3) -> f64 {
    point.x * axis.x + point.y * axis.y + point.z * axis.z
}

#[inline]
fn point_from_axes(origin: RgmPoint3, axes: [RgmVec3; 3], coeffs: [f64; 3]) -> RgmPoint3 {
    RgmPoint3 {
        x: origin.x + axes[0].x * coeffs[0] + axes[1].x * coeffs[1] + axes[2].x * coeffs[2],
        y: origin.y + axes[0].y * coeffs[0] + axes[1].y * coeffs[1] + axes[2].y * coeffs[2],
        z: origin.z + axes[0].z * coeffs[0] + axes[1].z * coeffs[1] + axes[2].z * coeffs[2],
    }
}

#[inline]
fn point_from_vec(vec: RgmVec3) -> RgmPoint3 {
    RgmPoint3 {
        x: vec.x,
        y: vec.y,
        z: vec.z,
    }
}

fn covariance_matrix(points: &[RgmPoint3]) -> [[f64; 3]; 3] {
    let n = points.len() as f64;
    let mut centroid = RgmPoint3 {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    for point in points {
        centroid.x += point.x;
        centroid.y += point.y;
        centroid.z += point.z;
    }
    centroid.x /= n;
    centroid.y /= n;
    centroid.z /= n;

    let mut cov = [[0.0_f64; 3]; 3];
    for point in points {
        let dx = point.x - centroid.x;
        let dy = point.y - centroid.y;
        let dz = point.z - centroid.z;
        cov[0][0] += dx * dx;
        cov[0][1] += dx * dy;
        cov[0][2] += dx * dz;
        cov[1][1] += dy * dy;
        cov[1][2] += dy * dz;
        cov[2][2] += dz * dz;
    }
    cov[1][0] = cov[0][1];
    cov[2][0] = cov[0][2];
    cov[2][1] = cov[1][2];

    if n > 1.0 {
        for row in &mut cov {
            for value in row {
                *value /= n;
            }
        }
    }
    cov
}

fn jacobi_eigen_decomposition(mut matrix: [[f64; 3]; 3]) -> ([f64; 3], [RgmVec3; 3]) {
    let mut vectors = [[0.0_f64; 3]; 3];
    for i in 0..3 {
        vectors[i][i] = 1.0;
    }

    for _ in 0..32 {
        let mut p = 0_usize;
        let mut q = 1_usize;
        let mut max = matrix[0][1].abs();
        for i in 0..3 {
            for j in (i + 1)..3 {
                let value = matrix[i][j].abs();
                if value > max {
                    max = value;
                    p = i;
                    q = j;
                }
            }
        }
        if max <= 1e-14 {
            break;
        }

        let app = matrix[p][p];
        let aqq = matrix[q][q];
        let apq = matrix[p][q];
        let theta = (aqq - app) / (2.0 * apq);
        let t = if theta >= 0.0 {
            1.0 / (theta + (1.0 + theta * theta).sqrt())
        } else {
            -1.0 / (-theta + (1.0 + theta * theta).sqrt())
        };
        let c = 1.0 / (1.0 + t * t).sqrt();
        let s = t * c;

        for k in 0..3 {
            if k != p && k != q {
                let aik = matrix[p][k];
                let akq = matrix[q][k];
                matrix[p][k] = c * aik - s * akq;
                matrix[k][p] = matrix[p][k];
                matrix[q][k] = s * aik + c * akq;
                matrix[k][q] = matrix[q][k];
            }
        }

        matrix[p][p] = c * c * app - 2.0 * s * c * apq + s * s * aqq;
        matrix[q][q] = s * s * app + 2.0 * s * c * apq + c * c * aqq;
        matrix[p][q] = 0.0;
        matrix[q][p] = 0.0;

        for row in &mut vectors {
            let vip = row[p];
            let viq = row[q];
            row[p] = c * vip - s * viq;
            row[q] = s * vip + c * viq;
        }
    }

    let mut eig_pairs = [
        (
            matrix[0][0],
            RgmVec3 {
                x: vectors[0][0],
                y: vectors[1][0],
                z: vectors[2][0],
            },
        ),
        (
            matrix[1][1],
            RgmVec3 {
                x: vectors[0][1],
                y: vectors[1][1],
                z: vectors[2][1],
            },
        ),
        (
            matrix[2][2],
            RgmVec3 {
                x: vectors[0][2],
                y: vectors[1][2],
                z: vectors[2][2],
            },
        ),
    ];
    eig_pairs.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    (
        [eig_pairs[0].0, eig_pairs[1].0, eig_pairs[2].0],
        [eig_pairs[0].1, eig_pairs[1].1, eig_pairs[2].1],
    )
}

fn perpendicular_axis(axis: RgmVec3) -> RgmVec3 {
    let seed = if axis.x.abs() <= axis.y.abs() && axis.x.abs() <= axis.z.abs() {
        RgmVec3 {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        }
    } else if axis.y.abs() <= axis.z.abs() {
        RgmVec3 {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        }
    } else {
        RgmVec3 {
            x: 0.0,
            y: 0.0,
            z: 1.0,
        }
    };
    v3::normalize(v3::cross(axis, seed)).unwrap_or(RgmVec3 {
        x: 0.0,
        y: 1.0,
        z: 0.0,
    })
}

fn orthonormalize_axes(mut axes: [RgmVec3; 3]) -> [RgmVec3; 3] {
    axes[0] = v3::normalize(axes[0]).unwrap_or(RgmVec3 {
        x: 1.0,
        y: 0.0,
        z: 0.0,
    });

    let y_proj = v3::scale(axes[0], v3::dot(axes[1], axes[0]));
    axes[1] = v3::normalize(v3::add(axes[1], v3::neg(y_proj))).unwrap_or_else(|| perpendicular_axis(axes[0]));
    axes[2] = v3::normalize(v3::cross(axes[0], axes[1])).unwrap_or(RgmVec3 {
        x: 0.0,
        y: 0.0,
        z: 1.0,
    });
    axes
}

fn initial_axes_from_points(points: &[RgmPoint3]) -> [RgmVec3; 3] {
    if points.len() <= 1 {
        return [
            RgmVec3 {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            RgmVec3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            RgmVec3 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
        ];
    }

    let cov = covariance_matrix(points);
    let (eigenvalues, eigenvectors) = jacobi_eigen_decomposition(cov);
    let max_ev = eigenvalues[0].abs().max(1e-24);
    let eps = max_ev * 1e-9;
    let rank = eigenvalues.iter().filter(|value| value.abs() > eps).count();

    let mut axes = match rank {
        0 => [
            RgmVec3 {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            },
            RgmVec3 {
                x: 0.0,
                y: 1.0,
                z: 0.0,
            },
            RgmVec3 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
        ],
        1 => {
            let x = v3::normalize(eigenvectors[0]).unwrap_or(RgmVec3 {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            });
            let y = perpendicular_axis(x);
            let z = v3::normalize(v3::cross(x, y)).unwrap_or(RgmVec3 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            });
            [x, y, z]
        }
        2 => {
            let x = v3::normalize(eigenvectors[0]).unwrap_or(RgmVec3 {
                x: 1.0,
                y: 0.0,
                z: 0.0,
            });
            let y = v3::normalize(eigenvectors[1]).unwrap_or_else(|| perpendicular_axis(x));
            let z = v3::normalize(v3::cross(x, y)).unwrap_or(RgmVec3 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            });
            [x, y, z]
        }
        _ => [eigenvectors[0], eigenvectors[1], eigenvectors[2]],
    };
    axes = orthonormalize_axes(axes);
    axes
}

fn rotate_vector(vector: RgmVec3, axis: RgmVec3, angle: f64) -> RgmVec3 {
    let c = angle.cos();
    let s = angle.sin();
    let term0 = v3::scale(vector, c);
    let term1 = v3::scale(v3::cross(axis, vector), s);
    let term2 = v3::scale(axis, v3::dot(axis, vector) * (1.0 - c));
    v3::add(v3::add(term0, term1), term2)
}

fn projection_extents(points: &[RgmPoint3], axes: [RgmVec3; 3]) -> ([f64; 3], [f64; 3], f64) {
    let mut mins = [f64::INFINITY; 3];
    let mut maxs = [f64::NEG_INFINITY; 3];
    for point in points {
        for axis_idx in 0..3 {
            let coord = dot_point_axis(*point, axes[axis_idx]);
            mins[axis_idx] = mins[axis_idx].min(coord);
            maxs[axis_idx] = maxs[axis_idx].max(coord);
        }
    }
    let extents = [
        (maxs[0] - mins[0]).max(0.0),
        (maxs[1] - mins[1]).max(0.0),
        (maxs[2] - mins[2]).max(0.0),
    ];
    (mins, maxs, extents[0] * extents[1] * extents[2])
}

fn refine_axes_for_optimal(points: &[RgmPoint3], axes: [RgmVec3; 3]) -> [RgmVec3; 3] {
    let mut best_axes = axes;
    let (_, _, mut best_volume) = projection_extents(points, best_axes);
    for angle in [0.24_f64, 0.1_f64, 0.04_f64] {
        let mut improved = true;
        while improved {
            improved = false;
            for axis_idx in 0..3 {
                for sign in [-1.0_f64, 1.0_f64] {
                    let theta = angle * sign;
                    let mut candidate = best_axes;
                    let rot_axis = best_axes[axis_idx];
                    for idx in 0..3 {
                        if idx == axis_idx {
                            continue;
                        }
                        candidate[idx] = rotate_vector(candidate[idx], rot_axis, theta);
                    }
                    candidate = orthonormalize_axes(candidate);
                    let (_, _, volume) = projection_extents(points, candidate);
                    if volume + 1e-18 < best_volume {
                        best_axes = candidate;
                        best_volume = volume;
                        improved = true;
                    }
                }
            }
        }
    }
    best_axes
}

pub(crate) fn aabb_from_points(points: &[RgmPoint3]) -> Result<RgmAabb3, RgmStatus> {
    if points.is_empty() {
        return Err(RgmStatus::InvalidInput);
    }
    let mut min = points[0];
    let mut max = points[0];
    for point in points.iter().skip(1) {
        min.x = min.x.min(point.x);
        min.y = min.y.min(point.y);
        min.z = min.z.min(point.z);
        max.x = max.x.max(point.x);
        max.y = max.y.max(point.y);
        max.z = max.z.max(point.z);
    }
    Ok(RgmAabb3 { min, max })
}

pub(crate) fn aabb_union(a: RgmAabb3, b: RgmAabb3) -> RgmAabb3 {
    RgmAabb3 {
        min: RgmPoint3 {
            x: a.min.x.min(b.min.x),
            y: a.min.y.min(b.min.y),
            z: a.min.z.min(b.min.z),
        },
        max: RgmPoint3 {
            x: a.max.x.max(b.max.x),
            y: a.max.y.max(b.max.y),
            z: a.max.z.max(b.max.z),
        },
    }
}

pub(crate) fn aabb_pad(aabb: RgmAabb3, padding: f64) -> RgmAabb3 {
    let pad = if padding.is_finite() { padding.max(0.0) } else { 0.0 };
    RgmAabb3 {
        min: RgmPoint3 {
            x: aabb.min.x - pad,
            y: aabb.min.y - pad,
            z: aabb.min.z - pad,
        },
        max: RgmPoint3 {
            x: aabb.max.x + pad,
            y: aabb.max.y + pad,
            z: aabb.max.z + pad,
        },
    }
}

pub(crate) fn aabb_corners(aabb: RgmAabb3) -> [RgmPoint3; 8] {
    let min = aabb.min;
    let max = aabb.max;
    [
        RgmPoint3 {
            x: min.x,
            y: min.y,
            z: min.z,
        },
        RgmPoint3 {
            x: max.x,
            y: min.y,
            z: min.z,
        },
        RgmPoint3 {
            x: min.x,
            y: max.y,
            z: min.z,
        },
        RgmPoint3 {
            x: max.x,
            y: max.y,
            z: min.z,
        },
        RgmPoint3 {
            x: min.x,
            y: min.y,
            z: max.z,
        },
        RgmPoint3 {
            x: max.x,
            y: min.y,
            z: max.z,
        },
        RgmPoint3 {
            x: min.x,
            y: max.y,
            z: max.z,
        },
        RgmPoint3 {
            x: max.x,
            y: max.y,
            z: max.z,
        },
    ]
}

pub(crate) fn compute_bounds_from_points(
    points: &[RgmPoint3],
    mode: RgmBoundsMode,
    padding: f64,
) -> Result<RgmBounds3, RgmStatus> {
    if points.is_empty() {
        return Err(RgmStatus::InvalidInput);
    }

    let world_aabb = aabb_pad(aabb_from_points(points)?, padding);

    let mut axes = initial_axes_from_points(points);
    if mode == RgmBoundsMode::Optimal && points.len() >= 8 {
        axes = refine_axes_for_optimal(points, axes);
    }

    let (mins, maxs, _) = projection_extents(points, axes);
    let pad = if padding.is_finite() { padding.max(0.0) } else { 0.0 };
    let mut half_extents = RgmVec3 {
        x: ((maxs[0] - mins[0]).max(0.0) * 0.5) + pad,
        y: ((maxs[1] - mins[1]).max(0.0) * 0.5) + pad,
        z: ((maxs[2] - mins[2]).max(0.0) * 0.5) + pad,
    };
    if !half_extents.x.is_finite() || !half_extents.y.is_finite() || !half_extents.z.is_finite() {
        half_extents = RgmVec3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        };
    }

    let mids = [
        (mins[0] + maxs[0]) * 0.5,
        (mins[1] + maxs[1]) * 0.5,
        (mins[2] + maxs[2]) * 0.5,
    ];
    let center = point_from_axes(
        RgmPoint3 {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        },
        axes,
        mids,
    );

    let world_obb = RgmObb3 {
        center,
        x_axis: axes[0],
        y_axis: axes[1],
        z_axis: axes[2],
        half_extents,
    };

    let local_aabb = RgmAabb3 {
        min: point_from_vec(RgmVec3 {
            x: -half_extents.x,
            y: -half_extents.y,
            z: -half_extents.z,
        }),
        max: point_from_vec(half_extents),
    };

    Ok(RgmBounds3 {
        world_aabb,
        world_obb,
        local_aabb,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn volume(bounds: RgmBounds3) -> f64 {
        let h = bounds.world_obb.half_extents;
        (h.x * 2.0) * (h.y * 2.0) * (h.z * 2.0)
    }

    #[test]
    fn degenerate_point_cloud_rank_zero() {
        let point = RgmPoint3 {
            x: 2.0,
            y: -1.0,
            z: 4.0,
        };
        let bounds = compute_bounds_from_points(&[point], RgmBoundsMode::Fast, 0.0)
            .expect("bounds");
        assert!((bounds.world_aabb.min.x - 2.0).abs() < 1e-12);
        assert!((bounds.world_aabb.max.y + 1.0).abs() < 1e-12);
        assert!(bounds.world_obb.half_extents.x.abs() < 1e-12);
        assert!(bounds.world_obb.half_extents.y.abs() < 1e-12);
        assert!(bounds.world_obb.half_extents.z.abs() < 1e-12);
    }

    #[test]
    fn degenerate_line_rank_one() {
        let mut points = Vec::new();
        for i in 0..16 {
            let t = i as f64 / 15.0;
            points.push(RgmPoint3 {
                x: -2.0 + 5.0 * t,
                y: 1.0 + 10.0 * t,
                z: -3.0 + 2.0 * t,
            });
        }
        let bounds = compute_bounds_from_points(&points, RgmBoundsMode::Fast, 0.0)
            .expect("bounds");
        assert!(bounds.world_obb.half_extents.x >= 0.0);
        assert!(bounds.world_obb.half_extents.y >= 0.0);
        assert!(bounds.world_obb.half_extents.z >= 0.0);
    }

    #[test]
    fn degenerate_plane_rank_two() {
        let mut points = Vec::new();
        for u in 0..10 {
            for v in 0..10 {
                points.push(RgmPoint3 {
                    x: u as f64 * 0.3,
                    y: v as f64 * 0.4,
                    z: 0.0,
                });
            }
        }
        let bounds = compute_bounds_from_points(&points, RgmBoundsMode::Fast, 0.0)
            .expect("bounds");
        assert!(bounds.world_obb.half_extents.z <= 1e-8);
    }

    #[test]
    fn optimal_volume_not_worse_than_fast() {
        let mut points = Vec::new();
        for i in 0..64 {
            let t = i as f64 / 63.0;
            points.push(RgmPoint3 {
                x: 3.0 * t,
                y: (8.0 * t).sin() * 0.7 + t * 2.0,
                z: (6.0 * t).cos() * 0.5 + t * 1.3,
            });
        }
        let fast = compute_bounds_from_points(&points, RgmBoundsMode::Fast, 0.0)
            .expect("fast");
        let optimal = compute_bounds_from_points(&points, RgmBoundsMode::Optimal, 0.0)
            .expect("optimal");
        assert!(volume(optimal) <= volume(fast) + 1e-9);
    }
}
