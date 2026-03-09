fn midpoint(a: RgmPoint3, b: RgmPoint3) -> RgmPoint3 {
    RgmPoint3 {
        x: 0.5 * (a.x + b.x),
        y: 0.5 * (a.y + b.y),
        z: 0.5 * (a.z + b.z),
    }
}

fn solve_linear_system<const N: usize>(mut a: [[f64; N]; N], mut b: [f64; N]) -> Option<[f64; N]> {
    for col in 0..N {
        let mut pivot = col;
        let mut pivot_abs = a[col][col].abs();
        for row in (col + 1)..N {
            let value = a[row][col].abs();
            if value > pivot_abs {
                pivot = row;
                pivot_abs = value;
            }
        }
        if pivot_abs <= 1e-20 {
            return None;
        }
        if pivot != col {
            a.swap(col, pivot);
            b.swap(col, pivot);
        }

        let diag = a[col][col];
        for row in (col + 1)..N {
            let factor = a[row][col] / diag;
            if factor.abs() <= f64::EPSILON {
                continue;
            }
            a[row][col] = 0.0;
            for k in (col + 1)..N {
                a[row][k] -= factor * a[col][k];
            }
            b[row] -= factor * b[col];
        }
    }

    let mut x = [0.0; N];
    for row in (0..N).rev() {
        let mut sum = b[row];
        for col in (row + 1)..N {
            sum -= a[row][col] * x[col];
        }
        let diag = a[row][row];
        if diag.abs() <= 1e-20 {
            return None;
        }
        x[row] = sum / diag;
    }
    Some(x)
}

fn matrix_identity() -> [[f64; 4]; 4] {
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

fn matrix_mul(a: [[f64; 4]; 4], b: [[f64; 4]; 4]) -> [[f64; 4]; 4] {
    let mut result = [[0.0; 4]; 4];
    for r in 0..4 {
        for c in 0..4 {
            result[r][c] =
                a[r][0] * b[0][c] + a[r][1] * b[1][c] + a[r][2] * b[2][c] + a[r][3] * b[3][c];
        }
    }
    result
}

fn matrix_translation(delta: RgmVec3) -> [[f64; 4]; 4] {
    let mut m = matrix_identity();
    m[0][3] = delta.x;
    m[1][3] = delta.y;
    m[2][3] = delta.z;
    m
}

fn matrix_scale(scale: RgmVec3) -> [[f64; 4]; 4] {
    [
        [scale.x, 0.0, 0.0, 0.0],
        [0.0, scale.y, 0.0, 0.0],
        [0.0, 0.0, scale.z, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

fn matrix_rotation(axis: RgmVec3, angle_rad: f64) -> Result<[[f64; 4]; 4], RgmStatus> {
    let unit = v3::normalize(axis).ok_or(RgmStatus::InvalidInput)?;
    let c = angle_rad.cos();
    let s = angle_rad.sin();
    let t = 1.0 - c;
    let x = unit.x;
    let y = unit.y;
    let z = unit.z;
    Ok([
        [t * x * x + c, t * x * y - s * z, t * x * z + s * y, 0.0],
        [t * x * y + s * z, t * y * y + c, t * y * z - s * x, 0.0],
        [t * x * z - s * y, t * y * z + s * x, t * z * z + c, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ])
}

fn matrix_about_pivot(transform: [[f64; 4]; 4], pivot: RgmPoint3) -> [[f64; 4]; 4] {
    let to_pivot = matrix_translation(RgmVec3 {
        x: pivot.x,
        y: pivot.y,
        z: pivot.z,
    });
    let from_pivot = matrix_translation(RgmVec3 {
        x: -pivot.x,
        y: -pivot.y,
        z: -pivot.z,
    });
    matrix_mul(to_pivot, matrix_mul(transform, from_pivot))
}

fn matrix_apply_point(matrix: [[f64; 4]; 4], point: RgmPoint3) -> RgmPoint3 {
    RgmPoint3 {
        x: matrix[0][0] * point.x + matrix[0][1] * point.y + matrix[0][2] * point.z + matrix[0][3],
        y: matrix[1][0] * point.x + matrix[1][1] * point.y + matrix[1][2] * point.z + matrix[1][3],
        z: matrix[2][0] * point.x + matrix[2][1] * point.y + matrix[2][2] * point.z + matrix[2][3],
    }
}

fn wrap_angle_0_2pi(angle: f64) -> f64 {
    let two_pi = 2.0 * PI;
    let mut wrapped = angle % two_pi;
    if wrapped < 0.0 {
        wrapped += two_pi;
    }
    wrapped
}

fn parse_coordinate_system(value: i32) -> Result<RgmAlignmentCoordinateSystem, RgmStatus> {
    match value {
        0 => Ok(RgmAlignmentCoordinateSystem::EastingNorthing),
        1 => Ok(RgmAlignmentCoordinateSystem::NorthingEasting),
        _ => Err(RgmStatus::InvalidInput),
    }
}

fn convert_point_coordinate_system(
    point: RgmPoint3,
    source: RgmAlignmentCoordinateSystem,
    target: RgmAlignmentCoordinateSystem,
) -> RgmPoint3 {
    if source == target {
        return point;
    }

    RgmPoint3 {
        x: point.y,
        y: point.x,
        z: point.z,
    }
}

fn arc_sweep_from_start_mid_end(
    start_angle: f64,
    mid_angle: f64,
    end_angle: f64,
    angle_tol: f64,
) -> Result<f64, RgmStatus> {
    let end_ccw = wrap_angle_0_2pi(end_angle - start_angle);
    let mid_ccw = wrap_angle_0_2pi(mid_angle - start_angle);
    let eps = angle_tol.max(1e-12);
    if end_ccw <= eps {
        return Err(RgmStatus::InvalidInput);
    }

    let sweep = if mid_ccw <= end_ccw + eps {
        end_ccw
    } else {
        end_ccw - 2.0 * PI
    };
    if sweep.abs() <= eps {
        return Err(RgmStatus::InvalidInput);
    }
    Ok(sweep)
}

fn build_arc_from_three_points(
    start: RgmPoint3,
    mid: RgmPoint3,
    end: RgmPoint3,
    tol: RgmToleranceContext,
) -> Result<RgmArc3, RgmStatus> {
    let eps = tol.abs_tol.max(1e-12);
    if v3::distance(start, mid) <= eps || v3::distance(mid, end) <= eps || v3::distance(start, end) <= eps {
        return Err(RgmStatus::InvalidInput);
    }

    let ab = v3::sub(mid, start);
    let ac = v3::sub(end, start);
    let normal = v3::cross(ab, ac);
    let normal_len2 = v3::dot(normal, normal);
    if normal_len2 <= eps * eps {
        return Err(RgmStatus::InvalidInput);
    }

    let term1 = v3::scale(v3::cross(ac, normal), v3::dot(ab, ab));
    let term2 = v3::scale(v3::cross(normal, ab), v3::dot(ac, ac));
    let center_offset = v3::scale(v3::add(term1, term2), 1.0 / (2.0 * normal_len2));
    let center = v3::add_vec(start, center_offset);
    let radius = v3::distance(center, start);
    if radius <= eps {
        return Err(RgmStatus::DegenerateGeometry);
    }

    let z_axis = v3::normalize(normal).ok_or(RgmStatus::DegenerateGeometry)?;
    let x_axis = v3::normalize(v3::sub(start, center)).ok_or(RgmStatus::DegenerateGeometry)?;
    let y_axis = v3::normalize(v3::cross(z_axis, x_axis)).ok_or(RgmStatus::DegenerateGeometry)?;

    let mid_vec = v3::sub(mid, center);
    let end_vec = v3::sub(end, center);
    let mid_angle = v3::dot(mid_vec, y_axis).atan2(v3::dot(mid_vec, x_axis));
    let end_angle = v3::dot(end_vec, y_axis).atan2(v3::dot(end_vec, x_axis));
    let sweep = arc_sweep_from_start_mid_end(0.0, mid_angle, end_angle, tol.angle_tol)?;

    Ok(RgmArc3 {
        plane: RgmPlane {
            origin: center,
            x_axis,
            y_axis,
            z_axis,
        },
        radius,
        start_angle: 0.0,
        sweep_angle: sweep,
    })
}

fn build_arc_from_start_end_angles(
    plane: RgmPlane,
    radius: f64,
    start_angle: f64,
    end_angle: f64,
    tol: RgmToleranceContext,
) -> Result<RgmArc3, RgmStatus> {
    let sweep = end_angle - start_angle;
    if sweep.abs() <= tol.angle_tol.max(1e-12) {
        return Err(RgmStatus::InvalidInput);
    }

    Ok(RgmArc3 {
        plane,
        radius,
        start_angle,
        sweep_angle: sweep,
    })
}

fn dedup_closed_points(mut points: Vec<RgmPoint3>, tol: f64) -> Vec<RgmPoint3> {
    if points.len() > 1 {
        let first = points[0];
        let last = points[points.len() - 1];
        if v3::distance(first, last) <= tol {
            points.pop();
        }
    }
    points
}

fn centripetal_params(points: &[RgmPoint3]) -> Vec<f64> {
    if points.len() <= 1 {
        return vec![0.0; points.len()];
    }

    let mut cumulative = vec![0.0; points.len()];
    let mut total = 0.0;

    for i in 1..points.len() {
        total += v3::distance(points[i - 1], points[i]).sqrt();
        cumulative[i] = total;
    }

    if total <= f64::EPSILON {
        return (0..points.len())
            .map(|idx| idx as f64 / (points.len() - 1) as f64)
            .collect();
    }

    cumulative.into_iter().map(|v| v / total).collect()
}

fn chord_length_params(points: &[RgmPoint3]) -> Vec<f64> {
    if points.len() <= 1 {
        return vec![0.0; points.len()];
    }

    let mut cumulative = vec![0.0; points.len()];
    let mut total = 0.0;

    for i in 1..points.len() {
        total += v3::distance(points[i - 1], points[i]);
        cumulative[i] = total;
    }

    if total <= f64::EPSILON {
        return (0..points.len())
            .map(|idx| idx as f64 / (points.len() - 1) as f64)
            .collect();
    }

    cumulative.into_iter().map(|v| v / total).collect()
}

fn clamped_open_knots(point_count: usize, degree: usize, params: &[f64]) -> Vec<f64> {
    let knot_count = point_count + degree + 1;
    let mut knots = vec![0.0; knot_count];

    for k in 0..=degree {
        knots[k] = 0.0;
        knots[knot_count - 1 - k] = 1.0;
    }

    if point_count > degree + 1 {
        let n = point_count - 1;
        let interior_count = n - degree;

        for j in 1..=interior_count {
            let mut sum = 0.0;
            for i in j..(j + degree) {
                sum += params[i];
            }
            knots[j + degree] = sum / degree as f64;
        }
    }

    knots
}

fn uniform_periodic_knots(control_count: usize, degree: usize) -> Vec<f64> {
    let knot_count = control_count + degree + 1;
    (0..knot_count).map(|idx| idx as f64).collect()
}

/// Solve the B-spline interpolation problem: find control points P such that
/// the B-spline defined by (P, knots, degree) passes through `data_points`
/// at the given `params`.
///
/// Algorithm from Piegl & Tiller, *The NURBS Book*, Section 9.2.1.
fn solve_bspline_interpolation(
    data_points: &[RgmPoint3],
    params: &[f64],
    knots: &[f64],
    degree: usize,
) -> Result<Vec<RgmPoint3>, RgmStatus> {
    let n = data_points.len();
    if n < 2 || params.len() != n {
        return Err(RgmStatus::InvalidInput);
    }

    let n_last = n - 1;

    let mut mat = vec![vec![0.0f64; n]; n];
    for k in 0..n {
        let span = math::basis::find_span(n_last, degree, params[k], knots)?;
        let basis = math::basis::basis_funs(span, params[k], degree, knots)?;
        for j in 0..=degree {
            let col = span - degree + j;
            if col < n {
                mat[k][col] = basis[j];
            }
        }
    }

    let mut rhs_x: Vec<f64> = data_points.iter().map(|p| p.x).collect();
    let mut rhs_y: Vec<f64> = data_points.iter().map(|p| p.y).collect();
    let mut rhs_z: Vec<f64> = data_points.iter().map(|p| p.z).collect();

    for col in 0..n {
        let mut max_val = mat[col][col].abs();
        let mut max_row = col;
        for row in (col + 1)..n.min(col + degree + 2) {
            if mat[row][col].abs() > max_val {
                max_val = mat[row][col].abs();
                max_row = row;
            }
        }
        if max_val < 1e-15 {
            return Err(RgmStatus::NumericalFailure);
        }
        if max_row != col {
            mat.swap(col, max_row);
            rhs_x.swap(col, max_row);
            rhs_y.swap(col, max_row);
            rhs_z.swap(col, max_row);
        }
        for row in (col + 1)..n.min(col + degree + 2) {
            let factor = mat[row][col] / mat[col][col];
            if factor.abs() < 1e-18 {
                continue;
            }
            for j in col..n.min(col + degree + 2) {
                mat[row][j] -= factor * mat[col][j];
            }
            rhs_x[row] -= factor * rhs_x[col];
            rhs_y[row] -= factor * rhs_y[col];
            rhs_z[row] -= factor * rhs_z[col];
        }
    }

    let mut result = vec![RgmPoint3 { x: 0.0, y: 0.0, z: 0.0 }; n];
    for i in (0..n).rev() {
        let mut sx = rhs_x[i];
        let mut sy = rhs_y[i];
        let mut sz = rhs_z[i];
        for j in (i + 1)..n.min(i + degree + 2) {
            sx -= mat[i][j] * result[j].x;
            sy -= mat[i][j] * result[j].y;
            sz -= mat[i][j] * result[j].z;
        }
        if mat[i][i].abs() < 1e-15 {
            return Err(RgmStatus::NumericalFailure);
        }
        result[i] = RgmPoint3 {
            x: sx / mat[i][i],
            y: sy / mat[i][i],
            z: sz / mat[i][i],
        };
    }

    Ok(result)
}

fn build_nurbs_from_core(
    degree: usize,
    periodic: bool,
    closed: bool,
    control_points: Vec<RgmPoint3>,
    weights: Vec<f64>,
    knots: Vec<f64>,
    tol: RgmToleranceContext,
    fit_points: Vec<RgmPoint3>,
) -> Result<NurbsCurveData, RgmStatus> {
    let core = NurbsCurveCore {
        degree,
        periodic,
        control_points,
        weights,
        knots,
        u_start: 0.0,
        u_end: 0.0,
        tol,
    };

    build_nurbs_from_core_auto_domain(core, closed, fit_points)
}

fn build_nurbs_from_core_auto_domain(
    mut core: NurbsCurveCore,
    closed: bool,
    fit_points: Vec<RgmPoint3>,
) -> Result<NurbsCurveData, RgmStatus> {
    let ctrl_count = core.control_points.len();
    if ctrl_count == 0 || ctrl_count <= core.degree {
        return Err(RgmStatus::InvalidInput);
    }

    core.u_start = core.knots[core.degree];
    core.u_end = core.knots[ctrl_count];

    validate_curve(&core)?;
    let arc_length = build_arc_length_cache(&core)?;

    Ok(NurbsCurveData {
        core,
        closed,
        fit_points,
        arc_length,
    })
}

fn build_periodic_nurbs_from_base(
    base_points: &[RgmPoint3],
    base_weights: &[f64],
    degree: usize,
    tol: RgmToleranceContext,
    closed: bool,
    fit_points: Vec<RgmPoint3>,
) -> Result<NurbsCurveData, RgmStatus> {
    if base_points.len() != base_weights.len() || base_points.len() <= degree {
        return Err(RgmStatus::InvalidInput);
    }

    let mut control_points = base_points.to_vec();
    let mut weights = base_weights.to_vec();

    for idx in 0..degree {
        control_points.push(base_points[idx]);
        weights.push(base_weights[idx]);
    }

    let knots = uniform_periodic_knots(control_points.len(), degree);
    build_nurbs_from_core(
        degree,
        true,
        closed,
        control_points,
        weights,
        knots,
        tol,
        fit_points,
    )
}

fn build_open_nurbs_from_points(
    points: &[RgmPoint3],
    degree: usize,
    tol: RgmToleranceContext,
    fit_points: Vec<RgmPoint3>,
) -> Result<NurbsCurveData, RgmStatus> {
    if points.len() <= degree {
        return Err(RgmStatus::InvalidInput);
    }

    let params = chord_length_params(points);
    let knots = clamped_open_knots(points.len(), degree, &params);
    let weights = vec![1.0; points.len()];

    build_nurbs_from_core(
        degree,
        false,
        false,
        points.to_vec(),
        weights,
        knots,
        tol,
        fit_points,
    )
}

fn build_nurbs_from_fit_points(
    points: &[RgmPoint3],
    degree: u32,
    closed: bool,
    tol: RgmToleranceContext,
) -> Result<NurbsCurveData, RgmStatus> {
    if points.is_empty() {
        return Err(RgmStatus::InvalidInput);
    }
    if degree == 0 {
        return Err(RgmStatus::InvalidInput);
    }

    let mut fit_points = points.to_vec();
    if closed {
        fit_points = dedup_closed_points(fit_points, tol.abs_tol.max(0.0));
    }

    let degree = degree as usize;
    if fit_points.len() <= degree {
        return Err(RgmStatus::InvalidInput);
    }

    if closed {
        let weights = vec![1.0; fit_points.len()];
        return build_periodic_nurbs_from_base(
            &fit_points,
            &weights,
            degree,
            tol,
            true,
            fit_points.clone(),
        );
    }

    build_open_nurbs_from_points(&fit_points, degree, tol, fit_points.clone())
}

fn build_line_nurbs(line: RgmLine3, tol: RgmToleranceContext) -> Result<NurbsCurveData, RgmStatus> {
    let points = vec![line.start, line.end];
    build_open_nurbs_from_points(&points, 1, tol, points.clone())
}

fn build_polyline_nurbs(
    points: &[RgmPoint3],
    closed: bool,
    tol: RgmToleranceContext,
) -> Result<NurbsCurveData, RgmStatus> {
    if points.len() < 2 {
        return Err(RgmStatus::InvalidInput);
    }

    let mut data = points.to_vec();
    if closed {
        data = dedup_closed_points(data, tol.abs_tol.max(0.0));
        if data.len() < 2 {
            return Err(RgmStatus::InvalidInput);
        }
        let weights = vec![1.0; data.len()];
        return build_periodic_nurbs_from_base(&data, &weights, 1, tol, true, data.clone());
    }

    build_open_nurbs_from_points(&data, 1, tol, data.clone())
}

fn build_arc_nurbs(arc: RgmArc3, tol: RgmToleranceContext) -> Result<NurbsCurveData, RgmStatus> {
    if arc.radius <= tol.abs_tol.max(1e-12) {
        return Err(RgmStatus::InvalidInput);
    }
    if arc.sweep_angle.abs() <= tol.angle_tol.max(1e-12) {
        return Err(RgmStatus::InvalidInput);
    }

    let (x_axis, y_axis) = orthonormalize_plane_axes(arc.plane)?;
    let center = arc.plane.origin;

    let segments = (arc.sweep_angle.abs() / FRAC_PI_2).ceil().max(1.0) as usize;
    let delta = arc.sweep_angle / segments as f64;

    let mut points = Vec::with_capacity(2 * segments + 1);
    let mut weights = Vec::with_capacity(2 * segments + 1);

    for seg in 0..segments {
        let a0 = arc.start_angle + seg as f64 * delta;
        let a1 = a0 + delta;
        let am = 0.5 * (a0 + a1);
        let w_mid = (0.5 * delta).cos();
        if w_mid.abs() <= 1e-12 {
            return Err(RgmStatus::NumericalFailure);
        }

        let p0 = point_from_frame(
            center,
            x_axis,
            y_axis,
            arc.radius * a0.cos(),
            arc.radius * a0.sin(),
        );
        let p1 = point_from_frame(
            center,
            x_axis,
            y_axis,
            (arc.radius / w_mid) * am.cos(),
            (arc.radius / w_mid) * am.sin(),
        );
        let p2 = point_from_frame(
            center,
            x_axis,
            y_axis,
            arc.radius * a1.cos(),
            arc.radius * a1.sin(),
        );

        if seg == 0 {
            points.push(p0);
            weights.push(1.0);
        }

        points.push(p1);
        weights.push(w_mid);
        points.push(p2);
        weights.push(1.0);
    }

    let degree = 2;
    let mut knots = vec![0.0; points.len() + degree + 1];
    let mut cursor = 0_usize;
    for _ in 0..=degree {
        knots[cursor] = 0.0;
        cursor += 1;
    }
    for idx in 1..segments {
        knots[cursor] = idx as f64;
        cursor += 1;
        knots[cursor] = idx as f64;
        cursor += 1;
    }
    for _ in 0..=degree {
        knots[cursor] = segments as f64;
        cursor += 1;
    }

    for knot in &mut knots {
        *knot /= segments as f64;
    }

    build_nurbs_from_core(
        degree,
        false,
        false,
        points,
        weights,
        knots,
        tol,
        Vec::new(),
    )
}

fn build_circle_nurbs(
    circle: RgmCircle3,
    tol: RgmToleranceContext,
) -> Result<NurbsCurveData, RgmStatus> {
    let arc = RgmArc3 {
        plane: circle.plane,
        radius: circle.radius,
        start_angle: 0.0,
        sweep_angle: 2.0 * PI,
    };
    let mut nurbs = build_arc_nurbs(arc, tol)?;
    nurbs.closed = true;
    Ok(nurbs)
}

fn reverse_open_nurbs(curve: &NurbsCurveData) -> Result<NurbsCurveData, RgmStatus> {
    if curve.core.periodic {
        return Err(RgmStatus::InvalidInput);
    }

    let mut control_points = curve.core.control_points.clone();
    control_points.reverse();

    let mut weights = curve.core.weights.clone();
    weights.reverse();

    let mut knots = vec![0.0; curve.core.knots.len()];
    for (idx, value) in curve.core.knots.iter().enumerate() {
        knots[curve.core.knots.len() - 1 - idx] = curve.core.u_start + curve.core.u_end - value;
    }

    build_nurbs_from_core(
        curve.core.degree,
        false,
        curve.closed,
        control_points,
        weights,
        knots,
        curve.core.tol,
        curve.fit_points.clone(),
    )
}

fn periodic_to_open_nurbs(curve: &NurbsCurveData) -> Result<NurbsCurveData, RgmStatus> {
    if !curve.core.periodic {
        return Ok(curve.clone());
    }

    let control_count = curve.core.control_points.len();
    let degree = curve.core.degree.max(1);
    let sample_count = (control_count * 12).max(degree * 18 + 16);
    let span = curve.core.u_end - curve.core.u_start;
    if span <= curve.core.tol.abs_tol.max(1e-12) {
        return Err(RgmStatus::DegenerateGeometry);
    }

    let mut points = Vec::with_capacity(sample_count + 1);
    for idx in 0..=sample_count {
        let t = idx as f64 / sample_count as f64;
        let mut u = curve.core.u_start + t * span;
        if idx == sample_count {
            u = curve.core.u_end - span * 1e-9;
        }
        let eval = eval_nurbs_u(&curve.core, u)?;
        points.push(eval.point);
    }

    build_open_nurbs_from_points(
        &points,
        curve.core.degree,
        curve.core.tol,
        curve.fit_points.clone(),
    )
}

#[derive(Clone, Copy, Debug)]
struct HomogeneousPoint {
    x: f64,
    y: f64,
    z: f64,
    w: f64,
}

impl HomogeneousPoint {
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
            w: self.w + other.w,
        }
    }

    fn scale(self, value: f64) -> Self {
        Self {
            x: self.x * value,
            y: self.y * value,
            z: self.z * value,
            w: self.w * value,
        }
    }

    fn blend(left: Self, right: Self, right_weight: f64) -> Self {
        left.scale(1.0 - right_weight)
            .add(right.scale(right_weight))
    }
}

fn to_homogeneous(point: RgmPoint3, weight: f64) -> HomogeneousPoint {
    HomogeneousPoint {
        x: point.x * weight,
        y: point.y * weight,
        z: point.z * weight,
        w: weight,
    }
}

fn from_homogeneous(value: HomogeneousPoint, eps: f64) -> Result<(RgmPoint3, f64), RgmStatus> {
    if value.w.abs() <= eps {
        return Err(RgmStatus::NumericalFailure);
    }

    Ok((
        RgmPoint3 {
            x: value.x / value.w,
            y: value.y / value.w,
            z: value.z / value.w,
        },
        value.w,
    ))
}

fn knot_multiplicity(knots: &[f64], knot: f64, eps: f64) -> usize {
    knots
        .iter()
        .filter(|value| (**value - knot).abs() <= eps)
        .count()
}

fn insert_knot_once_homogeneous(
    degree: usize,
    knots: &[f64],
    control: &[HomogeneousPoint],
    knot: f64,
    eps: f64,
) -> Result<(Vec<f64>, Vec<HomogeneousPoint>), RgmStatus> {
    if control.is_empty() || degree == 0 {
        return Err(RgmStatus::InvalidInput);
    }

    let n = control.len() - 1;
    let expected_knot_count = n + degree + 2;
    if knots.len() != expected_knot_count {
        return Err(RgmStatus::InvalidInput);
    }

    let m = expected_knot_count - 1;
    let span = math::basis::find_span(n, degree, knot, knots)?;
    let multiplicity = knot_multiplicity(knots, knot, eps);
    if multiplicity >= degree + 1 {
        return Err(RgmStatus::InvalidInput);
    }

    let mut next_knots = vec![0.0; knots.len() + 1];
    let mut next_control = vec![
        HomogeneousPoint {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 0.0,
        };
        control.len() + 1
    ];

    next_knots[..=span].copy_from_slice(&knots[..=span]);
    next_knots[span + 1] = knot;
    next_knots[span + 2..=m + 1].copy_from_slice(&knots[span + 1..=m]);

    let left_static_end = span.saturating_sub(degree);
    next_control[..=left_static_end].copy_from_slice(&control[..=left_static_end]);

    let right_start = span.saturating_sub(multiplicity);
    next_control[right_start + 1..=n + 1].copy_from_slice(&control[right_start..=n]);

    let blend_start = span.saturating_sub(degree) + 1;
    let blend_end = span.saturating_sub(multiplicity);
    if blend_start <= blend_end {
        for i in blend_start..=blend_end {
            let denom = knots[i + degree] - knots[i];
            let alpha = if denom.abs() <= eps {
                0.0
            } else {
                (knot - knots[i]) / denom
            };
            next_control[i] = HomogeneousPoint::blend(control[i - 1], control[i], alpha);
        }
    }

    Ok((next_knots, next_control))
}

fn elevate_bezier_homogeneous(
    control: &[HomogeneousPoint],
    target_degree: usize,
) -> Result<Vec<HomogeneousPoint>, RgmStatus> {
    if control.is_empty() {
        return Err(RgmStatus::InvalidInput);
    }

    let mut current = control.to_vec();
    let mut degree = current.len() - 1;
    if target_degree < degree {
        return Err(RgmStatus::InvalidInput);
    }

    while degree < target_degree {
        let mut elevated = vec![
            HomogeneousPoint {
                x: 0.0,
                y: 0.0,
                z: 0.0,
                w: 0.0,
            };
            degree + 2
        ];
        elevated[0] = current[0];
        elevated[degree + 1] = current[degree];
        for i in 1..=degree {
            let alpha = i as f64 / (degree + 1) as f64;
            elevated[i] = current[i - 1]
                .scale(alpha)
                .add(current[i].scale(1.0 - alpha));
        }

        current = elevated;
        degree += 1;
    }

    Ok(current)
}

fn elevate_open_nurbs_to_degree(
    curve: &NurbsCurveData,
    target_degree: usize,
) -> Result<NurbsCurveData, RgmStatus> {
    if curve.core.periodic || target_degree < curve.core.degree {
        return Err(RgmStatus::InvalidInput);
    }
    if target_degree == curve.core.degree {
        return Ok(curve.clone());
    }

    let degree = curve.core.degree;
    let control_count = curve.core.control_points.len();
    if control_count <= degree {
        return Err(RgmStatus::InvalidInput);
    }

    let expected_knot_count = control_count + degree + 1;
    if curve.core.knots.len() != expected_knot_count {
        return Err(RgmStatus::InvalidInput);
    }

    let mut knots = curve.core.knots.clone();
    let mut control: Vec<HomogeneousPoint> = curve
        .core
        .control_points
        .iter()
        .copied()
        .zip(curve.core.weights.iter().copied())
        .map(|(point, weight)| to_homogeneous(point, weight))
        .collect();

    let eps = curve.core.tol.abs_tol.max(1e-12);
    let n = control_count - 1;
    let u_start = curve.core.knots[degree];
    let u_end = curve.core.knots[n + 1];

    let mut internal_knots = Vec::new();
    let mut idx = degree + 1;
    while idx <= n {
        let knot = curve.core.knots[idx];
        if knot > u_start + eps && knot < u_end - eps {
            internal_knots.push(knot);
        }
        idx += 1;
        while idx <= n && (curve.core.knots[idx] - knot).abs() <= eps {
            idx += 1;
        }
    }

    for knot in internal_knots {
        loop {
            let multiplicity = knot_multiplicity(&knots, knot, eps);
            if multiplicity >= degree {
                break;
            }
            let (next_knots, next_control) =
                insert_knot_once_homogeneous(degree, &knots, &control, knot, eps)?;
            knots = next_knots;
            control = next_control;
        }
    }

    let n_after = control.len() - 1;
    let mut span_indices = Vec::new();
    for i in degree..=n_after {
        if knots[i + 1] - knots[i] > eps {
            span_indices.push(i);
        }
    }
    if span_indices.is_empty() {
        return Err(RgmStatus::DegenerateGeometry);
    }

    let mut boundaries = Vec::with_capacity(span_indices.len() + 1);
    let mut boundary_shared = Vec::with_capacity(span_indices.len().saturating_sub(1));
    boundaries.push(knots[span_indices[0]]);

    let mut elevated_control = Vec::new();
    let mut prev_end_idx: Option<usize> = None;
    for span_idx in span_indices {
        let segment_start = span_idx - degree;
        let segment_end = span_idx;
        let segment = &control[segment_start..=segment_end];
        let elevated_segment = elevate_bezier_homogeneous(segment, target_degree)?;

        if let Some(prev_end) = prev_end_idx {
            let shared = segment_start == prev_end;
            boundary_shared.push(shared);
            if shared {
                elevated_control.extend_from_slice(&elevated_segment[1..]);
            } else {
                elevated_control.extend_from_slice(&elevated_segment);
            }
        } else {
            elevated_control.extend_from_slice(&elevated_segment);
        }

        prev_end_idx = Some(segment_end);
        boundaries.push(knots[span_idx + 1]);
    }

    let mut elevated_knots = Vec::with_capacity(elevated_control.len() + target_degree + 1);
    for _ in 0..=target_degree {
        elevated_knots.push(boundaries[0]);
    }
    for (idx, boundary) in boundaries
        .iter()
        .take(boundaries.len().saturating_sub(1))
        .skip(1)
        .enumerate()
    {
        let mult = if boundary_shared[idx] {
            target_degree
        } else {
            target_degree + 1
        };
        for _ in 0..mult {
            elevated_knots.push(*boundary);
        }
    }
    for _ in 0..=target_degree {
        elevated_knots.push(boundaries[boundaries.len() - 1]);
    }

    let denom_eps = curve.core.tol.abs_tol.max(1e-14);
    let mut elevated_points = Vec::with_capacity(elevated_control.len());
    let mut elevated_weights = Vec::with_capacity(elevated_control.len());
    for value in elevated_control {
        let (point, weight) = from_homogeneous(value, denom_eps)?;
        elevated_points.push(point);
        elevated_weights.push(weight);
    }

    build_nurbs_from_core(
        target_degree,
        false,
        curve.closed,
        elevated_points,
        elevated_weights,
        elevated_knots,
        curve.core.tol,
        curve.fit_points.clone(),
    )
}

fn reparameterize_open_nurbs(
    curve: &NurbsCurveData,
    new_start: f64,
    new_end: f64,
) -> Result<NurbsCurveData, RgmStatus> {
    if curve.core.periodic {
        return Err(RgmStatus::InvalidInput);
    }

    let old_start = curve.core.u_start;
    let old_end = curve.core.u_end;
    let old_span = old_end - old_start;
    let new_span = new_end - new_start;
    let eps = curve.core.tol.abs_tol.max(1e-12);

    if old_span <= eps || new_span <= eps {
        return Err(RgmStatus::DegenerateGeometry);
    }

    let scale = new_span / old_span;
    let offset = new_start - scale * old_start;
    let knots: Vec<f64> = curve
        .core
        .knots
        .iter()
        .map(|value| scale * *value + offset)
        .collect();

    build_nurbs_from_core(
        curve.core.degree,
        false,
        curve.closed,
        curve.core.control_points.clone(),
        curve.core.weights.clone(),
        knots,
        curve.core.tol,
        curve.fit_points.clone(),
    )
}


