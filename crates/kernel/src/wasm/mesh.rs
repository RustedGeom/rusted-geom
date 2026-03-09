//! Mesh constructor and query methods on `KernelSession`.

use super::{flat_to_points, points_to_flat, KernelSession, MeshHandle};
use crate::{
    rgm_intersect_mesh_mesh, rgm_intersect_mesh_plane, rgm_mesh_boolean, rgm_mesh_copy_indices,
    rgm_mesh_copy_vertices, rgm_mesh_create_box, rgm_mesh_create_indexed, rgm_mesh_create_torus,
    rgm_mesh_create_uv_sphere, rgm_mesh_rotate, rgm_mesh_scale, rgm_mesh_bake_transform,
    rgm_mesh_translate, rgm_mesh_triangle_count, rgm_mesh_vertex_count,
    rgm_mesh_volume,
    RgmObjectHandle, RgmPlane, RgmPoint3, RgmVec3,
};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl KernelSession {
    // ── Constructors ──────────────────────────────────────────────────────────

    /// Create a mesh from indexed geometry.
    ///
    /// `vertices`: flat `[x,y,z, …]`.  `indices`: triangle indices (triplets).
    pub fn create_indexed_mesh(
        &self,
        vertices: Vec<f64>,
        indices: Vec<u32>,
    ) -> Result<MeshHandle, JsValue> {
        let pts = flat_to_points(&vertices);
        let mut out = RgmObjectHandle(0);
        super::error::check(rgm_mesh_create_indexed(
            self.handle(), pts.as_ptr(), pts.len(),
            indices.as_ptr(), indices.len(), &mut out,
        ))?;
        Ok(MeshHandle::new(self.session_id, out.0))
    }

    /// Create an axis-aligned box mesh.
    ///
    /// `center` = [cx, cy, cz], `size` = [sx, sy, sz].
    pub fn create_box_mesh(
        &self,
        cx: f64, cy: f64, cz: f64,
        sx: f64, sy: f64, sz: f64,
    ) -> Result<MeshHandle, JsValue> {
        let center = RgmPoint3 { x: cx, y: cy, z: cz };
        let size   = RgmVec3   { x: sx, y: sy, z: sz };
        let mut out = RgmObjectHandle(0);
        super::error::check(rgm_mesh_create_box(
            self.handle(), &center, &size, &mut out,
        ))?;
        Ok(MeshHandle::new(self.session_id, out.0))
    }

    /// Create a UV-sphere mesh.
    pub fn create_uv_sphere_mesh(
        &self,
        cx: f64, cy: f64, cz: f64,
        radius: f64,
        u_steps: u32,
        v_steps: u32,
    ) -> Result<MeshHandle, JsValue> {
        let center = RgmPoint3 { x: cx, y: cy, z: cz };
        let mut out = RgmObjectHandle(0);
        super::error::check(rgm_mesh_create_uv_sphere(
            self.handle(), &center, radius, u_steps, v_steps, &mut out,
        ))?;
        Ok(MeshHandle::new(self.session_id, out.0))
    }

    /// Create a torus mesh.
    pub fn create_torus_mesh(
        &self,
        cx: f64, cy: f64, cz: f64,
        major_radius: f64,
        minor_radius: f64,
        major_steps: u32,
        minor_steps: u32,
    ) -> Result<MeshHandle, JsValue> {
        let center = RgmPoint3 { x: cx, y: cy, z: cz };
        let mut out = RgmObjectHandle(0);
        super::error::check(rgm_mesh_create_torus(
            self.handle(), &center, major_radius, minor_radius,
            major_steps, minor_steps, &mut out,
        ))?;
        Ok(MeshHandle::new(self.session_id, out.0))
    }

    // ── Transforms ────────────────────────────────────────────────────────────

    /// Translate a mesh.  Returns a new handle.
    pub fn mesh_translate(
        &self,
        mesh: &MeshHandle,
        dx: f64, dy: f64, dz: f64,
    ) -> Result<MeshHandle, JsValue> {
        let delta = RgmVec3 { x: dx, y: dy, z: dz };
        let mut out = RgmObjectHandle(0);
        super::error::check(rgm_mesh_translate(
            self.handle(), RgmObjectHandle(mesh.object_id), &delta, &mut out,
        ))?;
        Ok(MeshHandle::new(self.session_id, out.0))
    }

    /// Rotate a mesh around `axis` by `angle_rad` about `pivot`.  Returns a new handle.
    pub fn mesh_rotate(
        &self,
        mesh: &MeshHandle,
        axis_x: f64, axis_y: f64, axis_z: f64,
        angle_rad: f64,
        pivot_x: f64, pivot_y: f64, pivot_z: f64,
    ) -> Result<MeshHandle, JsValue> {
        let axis  = RgmVec3   { x: axis_x,  y: axis_y,  z: axis_z  };
        let pivot = crate::RgmPoint3 { x: pivot_x, y: pivot_y, z: pivot_z };
        let mut out = RgmObjectHandle(0);
        super::error::check(rgm_mesh_rotate(
            self.handle(), RgmObjectHandle(mesh.object_id),
            &axis, angle_rad, &pivot, &mut out,
        ))?;
        Ok(MeshHandle::new(self.session_id, out.0))
    }

    /// Scale a mesh non-uniformly about `pivot`.  Returns a new handle.
    pub fn mesh_scale(
        &self,
        mesh: &MeshHandle,
        sx: f64, sy: f64, sz: f64,
        pivot_x: f64, pivot_y: f64, pivot_z: f64,
    ) -> Result<MeshHandle, JsValue> {
        let scale = RgmVec3   { x: sx, y: sy, z: sz };
        let pivot = crate::RgmPoint3 { x: pivot_x, y: pivot_y, z: pivot_z };
        let mut out = RgmObjectHandle(0);
        super::error::check(rgm_mesh_scale(
            self.handle(), RgmObjectHandle(mesh.object_id),
            &scale, &pivot, &mut out,
        ))?;
        Ok(MeshHandle::new(self.session_id, out.0))
    }

    /// Bake the transform of a mesh.  Returns a new handle.
    pub fn mesh_bake_transform(&self, mesh: &MeshHandle) -> Result<MeshHandle, JsValue> {
        let mut out = RgmObjectHandle(0);
        super::error::check(rgm_mesh_bake_transform(
            self.handle(), RgmObjectHandle(mesh.object_id), &mut out,
        ))?;
        Ok(MeshHandle::new(self.session_id, out.0))
    }

    // ── Queries ───────────────────────────────────────────────────────────────

    /// Number of vertices in a mesh.
    pub fn mesh_vertex_count(&self, mesh: &MeshHandle) -> Result<u32, JsValue> {
        let mut count = 0u32;
        super::error::check(rgm_mesh_vertex_count(
            self.handle(), RgmObjectHandle(mesh.object_id), &mut count,
        ))?;
        Ok(count)
    }

    /// Number of triangles in a mesh.
    pub fn mesh_triangle_count(&self, mesh: &MeshHandle) -> Result<u32, JsValue> {
        let mut count = 0u32;
        super::error::check(rgm_mesh_triangle_count(
            self.handle(), RgmObjectHandle(mesh.object_id), &mut count,
        ))?;
        Ok(count)
    }

    /// Copy all vertices as a flat `[x,y,z, …]` array.
    pub fn mesh_copy_vertices(&self, mesh: &MeshHandle) -> Result<Vec<f64>, JsValue> {
        // Phase 1: query count
        let mut count = 0u32;
        super::error::check(rgm_mesh_vertex_count(
            self.handle(), RgmObjectHandle(mesh.object_id), &mut count,
        ))?;
        if count == 0 {
            return Ok(Vec::new());
        }
        // Phase 2: fill
        let mut pts = vec![RgmPoint3 { x: 0., y: 0., z: 0. }; count as usize];
        let mut actual = 0u32;
        super::error::check(rgm_mesh_copy_vertices(
            self.handle(), RgmObjectHandle(mesh.object_id),
            pts.as_mut_ptr(), count, &mut actual,
        ))?;
        pts.truncate(actual as usize);
        Ok(points_to_flat(&pts))
    }

    /// Copy all vertex positions as a flat `[x,y,z, …]` `Float32Array`.
    ///
    /// Preferred over `mesh_copy_vertices` when feeding Three.js `BufferAttribute`,
    /// as it avoids a `Float64Array → RgmPoint3[] → Float32Array` double-copy.
    pub fn mesh_copy_positions_f32(&self, mesh: &MeshHandle) -> Result<Vec<f32>, JsValue> {
        let mut count = 0u32;
        super::error::check(rgm_mesh_vertex_count(
            self.handle(), RgmObjectHandle(mesh.object_id), &mut count,
        ))?;
        if count == 0 {
            return Ok(Vec::new());
        }
        let mut pts = vec![RgmPoint3 { x: 0., y: 0., z: 0. }; count as usize];
        let mut actual = 0u32;
        super::error::check(rgm_mesh_copy_vertices(
            self.handle(), RgmObjectHandle(mesh.object_id),
            pts.as_mut_ptr(), count, &mut actual,
        ))?;
        pts.truncate(actual as usize);
        let mut out = Vec::with_capacity(actual as usize * 3);
        for p in &pts {
            out.push(p.x as f32);
            out.push(p.y as f32);
            out.push(p.z as f32);
        }
        Ok(out)
    }

    /// Copy all triangle indices as a flat `[i0,i1,i2, …]` array.
    pub fn mesh_copy_indices(&self, mesh: &MeshHandle) -> Result<Vec<u32>, JsValue> {
        // Phase 1: query count (triangles × 3)
        let mut tri_count = 0u32;
        super::error::check(rgm_mesh_triangle_count(
            self.handle(), RgmObjectHandle(mesh.object_id), &mut tri_count,
        ))?;
        if tri_count == 0 {
            return Ok(Vec::new());
        }
        let capacity = tri_count * 3;
        // Phase 2: fill
        let mut indices = vec![0u32; capacity as usize];
        let mut actual = 0u32;
        super::error::check(rgm_mesh_copy_indices(
            self.handle(), RgmObjectHandle(mesh.object_id),
            indices.as_mut_ptr(), capacity, &mut actual,
        ))?;
        indices.truncate(actual as usize);
        Ok(indices)
    }

    // ── Boolean & intersection ────────────────────────────────────────────────

    /// Mesh boolean operation.  `op`: 0 = Union, 1 = Intersection, 2 = Difference.
    pub fn mesh_boolean(
        &self,
        mesh_a: &MeshHandle,
        mesh_b: &MeshHandle,
        op: i32,
    ) -> Result<MeshHandle, JsValue> {
        let mut out = RgmObjectHandle(0);
        super::error::check(rgm_mesh_boolean(
            self.handle(),
            RgmObjectHandle(mesh_a.object_id),
            RgmObjectHandle(mesh_b.object_id),
            op, &mut out,
        ))?;
        Ok(MeshHandle::new(self.session_id, out.0))
    }

    /// Intersect a mesh with a plane.
    ///
    /// `plane` is flat: `[ox,oy,oz, xx,xy,xz, yx,yy,yz, zx,zy,zz]`.
    /// Returns a flat `[x,y,z, …]` array of intersection points.
    pub fn intersect_mesh_plane(
        &self,
        mesh: &MeshHandle,
        plane: Vec<f64>,
    ) -> Result<Vec<f64>, JsValue> {
        let p = parse_plane(&plane)?;
        // Phase 1: count
        let mut count = 0u32;
        rgm_intersect_mesh_plane(
            self.handle(), RgmObjectHandle(mesh.object_id),
            &p, std::ptr::null_mut(), 0, &mut count,
        );
        if count == 0 {
            return Ok(Vec::new());
        }
        // Phase 2: fill
        let mut pts = vec![RgmPoint3 { x: 0., y: 0., z: 0. }; count as usize];
        let mut actual = 0u32;
        super::error::check(rgm_intersect_mesh_plane(
            self.handle(), RgmObjectHandle(mesh.object_id),
            &p, pts.as_mut_ptr(), count, &mut actual,
        ))?;
        pts.truncate(actual as usize);
        Ok(points_to_flat(&pts))
    }

    /// Intersect two meshes.
    /// Returns a flat `[x,y,z, …]` array of intersection points.
    pub fn intersect_mesh_mesh(
        &self,
        mesh_a: &MeshHandle,
        mesh_b: &MeshHandle,
    ) -> Result<Vec<f64>, JsValue> {
        // Phase 1: count
        let mut count = 0u32;
        rgm_intersect_mesh_mesh(
            self.handle(),
            RgmObjectHandle(mesh_a.object_id),
            RgmObjectHandle(mesh_b.object_id),
            std::ptr::null_mut(), 0, &mut count,
        );
        if count == 0 {
            return Ok(Vec::new());
        }
        // Phase 2: fill
        let mut pts = vec![RgmPoint3 { x: 0., y: 0., z: 0. }; count as usize];
        let mut actual = 0u32;
        super::error::check(rgm_intersect_mesh_mesh(
            self.handle(),
            RgmObjectHandle(mesh_a.object_id),
            RgmObjectHandle(mesh_b.object_id),
            pts.as_mut_ptr(), count, &mut actual,
        ))?;
        pts.truncate(actual as usize);
        Ok(points_to_flat(&pts))
    }

    /// Compute the volume enclosed by a closed mesh using the divergence theorem.
    pub fn mesh_volume(&self, mesh: &MeshHandle) -> Result<f64, JsValue> {
        let mut vol = 0.0_f64;
        super::error::check(rgm_mesh_volume(
            self.handle(),
            RgmObjectHandle(mesh.object_id),
            &mut vol,
        ))?;
        Ok(vol)
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

fn parse_plane(flat: &[f64]) -> Result<RgmPlane, JsValue> {
    if flat.len() < 12 {
        return Err(JsValue::from_str("plane array must have 12 values [ox,oy,oz, xx,xy,xz, yx,yy,yz, zx,zy,zz]"));
    }
    Ok(RgmPlane {
        origin: crate::RgmPoint3 { x: flat[0],  y: flat[1],  z: flat[2]  },
        x_axis: crate::RgmVec3   { x: flat[3],  y: flat[4],  z: flat[5]  },
        y_axis: crate::RgmVec3   { x: flat[6],  y: flat[7],  z: flat[8]  },
        z_axis: crate::RgmVec3   { x: flat[9],  y: flat[10], z: flat[11] },
    })
}

