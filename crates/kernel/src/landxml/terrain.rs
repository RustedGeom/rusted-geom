use super::types::TerrainTin;

pub fn terrain_vertex_count(terrain: &TerrainTin) -> usize {
    terrain.vertices_m.len()
}

pub fn terrain_triangle_count(terrain: &TerrainTin) -> usize {
    terrain.triangles.len() / 3
}
