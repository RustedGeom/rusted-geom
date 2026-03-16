use std::fmt;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[repr(C)]
pub struct GfVec3d {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[repr(C)]
pub struct GfVec3f {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[repr(C)]
pub struct GfVec2d {
    pub x: f64,
    pub y: f64,
}

pub type GfMatrix4d = [[f64; 4]; 4];

pub type VtArray<T> = Vec<T>;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TfToken(pub Arc<str>);

impl TfToken {
    pub fn new(s: &str) -> Self {
        TfToken(Arc::from(s))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for TfToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TfToken({:?})", &*self.0)
    }
}

impl fmt::Display for TfToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl Default for TfToken {
    fn default() -> Self {
        TfToken(Arc::from(""))
    }
}

impl From<&str> for TfToken {
    fn from(s: &str) -> Self {
        TfToken::new(s)
    }
}

impl From<String> for TfToken {
    fn from(s: String) -> Self {
        TfToken(Arc::from(s.as_str()))
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct SdfPath(Arc<str>);

impl SdfPath {
    pub fn new(s: &str) -> Self {
        SdfPath(Arc::from(s))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn parent(&self) -> Option<SdfPath> {
        let s = self.as_str();
        if s == "/" {
            return None;
        }
        match s.rfind('/') {
            Some(0) => Some(SdfPath::new("/")),
            Some(i) => Some(SdfPath::new(&s[..i])),
            None => None,
        }
    }

    pub fn child(&self, name: &str) -> SdfPath {
        let s = self.as_str();
        if s == "/" {
            SdfPath::new(&format!("/{name}"))
        } else {
            SdfPath::new(&format!("{s}/{name}"))
        }
    }

    pub fn name(&self) -> &str {
        let s = self.as_str();
        match s.rfind('/') {
            Some(i) => &s[i + 1..],
            None => s,
        }
    }
}

impl fmt::Debug for SdfPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SdfPath({:?})", &*self.0)
    }
}

impl fmt::Display for SdfPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&str> for SdfPath {
    fn from(s: &str) -> Self {
        SdfPath::new(s)
    }
}

impl From<String> for SdfPath {
    fn from(s: String) -> Self {
        SdfPath(Arc::from(s.as_str()))
    }
}

impl GfVec3d {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }
}

impl GfVec3f {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    pub fn to_f64(self) -> GfVec3d {
        GfVec3d {
            x: self.x as f64,
            y: self.y as f64,
            z: self.z as f64,
        }
    }
}

impl GfVec2d {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

impl From<GfVec3f> for GfVec3d {
    fn from(v: GfVec3f) -> Self {
        v.to_f64()
    }
}
