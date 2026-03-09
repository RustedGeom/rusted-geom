#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr;
    use std::time::Instant;

    include!("../tests/helpers.rs");
    include!("../tests/test_session.rs");
    include!("../tests/test_curves.rs");
    include!("../tests/test_surfaces.rs");
    include!("../tests/test_meshes.rs");
    include!("../tests/test_intersections.rs");
    include!("../tests/test_brep.rs");
    include!("../tests/test_bounds.rs");
    include!("../tests/test_sweep_loft.rs");
}
