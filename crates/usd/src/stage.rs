use std::collections::HashMap;

use crate::foundation::SdfPath;
use crate::schema::generated::{SchemaData, UsdSchema};

/// A single prim in the scene graph.
#[derive(Clone, Debug)]
pub struct UsdPrim {
    pub path: SdfPath,
    pub schema: SchemaData,
    pub active: bool,
}

/// Lightweight in-memory USD scene graph.
///
/// Flat scene only — no composition, no references, no variants.
/// Prims are addressed by `SdfPath` and store a `SchemaData` payload.
#[derive(Clone, Debug)]
pub struct UsdStage {
    prims: HashMap<SdfPath, UsdPrim>,
    children: HashMap<SdfPath, Vec<SdfPath>>,
}

impl Default for UsdStage {
    fn default() -> Self {
        Self::new()
    }
}

impl UsdStage {
    pub fn new() -> Self {
        Self {
            prims: HashMap::new(),
            children: HashMap::new(),
        }
    }

    /// Define a new prim at `path` with the given schema data.
    /// Creates any missing ancestor Scope prims automatically.
    pub fn define_prim(&mut self, path: SdfPath, schema: SchemaData) -> &mut UsdPrim {
        self.ensure_ancestors(&path);

        if let Some(parent) = path.parent() {
            let siblings = self.children.entry(parent).or_default();
            if !siblings.contains(&path) {
                siblings.push(path.clone());
            }
        }

        let prim = self.prims.entry(path.clone()).or_insert_with(|| UsdPrim {
            path: path.clone(),
            schema: schema.clone(),
            active: true,
        });
        prim.schema = schema;
        prim
    }

    /// Get an immutable reference to a typed prim.
    pub fn get<T: UsdSchema>(&self, path: &SdfPath) -> Option<&T> {
        let prim = self.prims.get(path)?;
        T::from_schema_data(&prim.schema)
    }

    /// Get a mutable reference to a typed prim.
    pub fn get_mut<T: UsdSchema>(&mut self, path: &SdfPath) -> Option<&mut T> {
        let prim = self.prims.get_mut(path)?;
        T::from_schema_data_mut(&mut prim.schema)
    }

    /// Get the raw prim at a path.
    pub fn prim(&self, path: &SdfPath) -> Option<&UsdPrim> {
        self.prims.get(path)
    }

    /// Get the raw prim mutably.
    pub fn prim_mut(&mut self, path: &SdfPath) -> Option<&mut UsdPrim> {
        self.prims.get_mut(path)
    }

    /// Returns child paths of the given parent, in insertion order.
    pub fn children(&self, parent: &SdfPath) -> &[SdfPath] {
        self.children.get(parent).map_or(&[], Vec::as_slice)
    }

    /// Remove a prim and all its descendants.
    pub fn remove_prim(&mut self, path: &SdfPath) {
        let child_paths: Vec<SdfPath> = self
            .children
            .get(path)
            .cloned()
            .unwrap_or_default();
        for child in child_paths {
            self.remove_prim(&child);
        }
        self.prims.remove(path);
        self.children.remove(path);
        if let Some(parent) = path.parent() {
            if let Some(siblings) = self.children.get_mut(&parent) {
                siblings.retain(|p| p != path);
            }
        }
    }

    /// Iterate over all root-level prims.
    pub fn root_prims(&self) -> Vec<&UsdPrim> {
        let root = SdfPath::new("/");
        self.children(&root)
            .iter()
            .filter_map(|p| self.prims.get(p))
            .collect()
    }

    /// Total number of prims on the stage.
    pub fn prim_count(&self) -> usize {
        self.prims.len()
    }

    /// Check whether a prim exists at the given path.
    pub fn has_prim(&self, path: &SdfPath) -> bool {
        self.prims.contains_key(path)
    }

    /// Iterate all prims in no guaranteed order.
    pub fn all_prims(&self) -> impl Iterator<Item = &UsdPrim> {
        self.prims.values()
    }

    /// Ensure all ancestor paths exist as Scope prims.
    fn ensure_ancestors(&mut self, path: &SdfPath) {
        let mut ancestors = Vec::new();
        let mut current = path.parent();
        while let Some(p) = current {
            if p.as_str() == "/" {
                break;
            }
            if self.prims.contains_key(&p) {
                break;
            }
            ancestors.push(p.clone());
            current = p.parent();
        }

        for anc in ancestors.into_iter().rev() {
            if let Some(parent) = anc.parent() {
                let siblings = self.children.entry(parent).or_default();
                if !siblings.contains(&anc) {
                    siblings.push(anc.clone());
                }
            }
            self.prims.entry(anc.clone()).or_insert_with(|| UsdPrim {
                path: anc,
                schema: SchemaData::default(),
                active: true,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::generated::{UsdGeomMesh, UsdGeomNurbsCurves, UsdGeomScope};

    #[test]
    fn define_and_retrieve_prim() {
        let mut stage = UsdStage::new();
        let path = SdfPath::new("/Meshes/Body0");
        let mesh = UsdGeomMesh::default();
        stage.define_prim(path.clone(), SchemaData::Mesh(mesh));

        assert!(stage.has_prim(&path));
        assert_eq!(stage.prim_count(), 2); // /Meshes (scope) + /Meshes/Body0

        let retrieved = stage.get::<UsdGeomMesh>(&path);
        assert!(retrieved.is_some());
    }

    #[test]
    fn children_ordering() {
        let mut stage = UsdStage::new();
        stage.define_prim(SdfPath::new("/A"), SchemaData::Scope(UsdGeomScope));
        stage.define_prim(SdfPath::new("/B"), SchemaData::Scope(UsdGeomScope));
        stage.define_prim(SdfPath::new("/C"), SchemaData::Scope(UsdGeomScope));

        let root = SdfPath::new("/");
        let children: Vec<&str> = stage.children(&root).iter().map(|p| p.as_str()).collect();
        assert_eq!(children, vec!["/A", "/B", "/C"]);
    }

    #[test]
    fn auto_creates_ancestor_scopes() {
        let mut stage = UsdStage::new();
        let path = SdfPath::new("/A/B/C/D");
        let curves = UsdGeomNurbsCurves::default();
        stage.define_prim(path.clone(), SchemaData::NurbsCurves(curves));

        assert!(stage.has_prim(&SdfPath::new("/A")));
        assert!(stage.has_prim(&SdfPath::new("/A/B")));
        assert!(stage.has_prim(&SdfPath::new("/A/B/C")));
        assert!(stage.has_prim(&path));
    }

    #[test]
    fn remove_prim_recursive() {
        let mut stage = UsdStage::new();
        stage.define_prim(SdfPath::new("/A/B"), SchemaData::Scope(UsdGeomScope));
        stage.define_prim(SdfPath::new("/A/C"), SchemaData::Scope(UsdGeomScope));

        stage.remove_prim(&SdfPath::new("/A"));
        assert!(!stage.has_prim(&SdfPath::new("/A")));
        assert!(!stage.has_prim(&SdfPath::new("/A/B")));
        assert!(!stage.has_prim(&SdfPath::new("/A/C")));
    }
}
