//! DO NOT EDIT — generated from usd/schema.usda by `cargo xtask codegen`.
//! Schema version: OpenUSD 24.11
//!
//! Re-run `cargo xtask codegen` to regenerate.
#![allow(non_camel_case_types)]
/// Attribute metadata for generic serialization.
#[derive(Clone, Debug)]
pub struct AttributeMetadata {
    pub usd_name: &'static str,
    pub usd_type: &'static str,
    pub is_uniform: bool,
}
/// Trait providing schema metadata for generic serialization.
pub trait UsdSchemaInfo {
    fn schema_name(&self) -> &'static str;
    fn attribute_metadata(&self) -> &'static [AttributeMetadata];
}
/// Trait for type-safe prim access.
pub trait UsdSchema: Sized {
    fn from_schema_data(d: &SchemaData) -> Option<&Self>;
    fn from_schema_data_mut(d: &mut SchemaData) -> Option<&mut Self>;
}
///Concrete prim schema for a transform, which implements Xformable
#[derive(Clone, Debug)]
pub struct UsdGeomXform {
    /**Visibility is meant to be the simplest form of "pruning"
visibility that is supported by most DCC apps.  Visibility is
animatable, allowing a sub-tree of geometry to be present for some
segment of a shot, and absent from others; unlike the action of
deactivating geometry prims, invisible geometry is still
available for inspection, for positioning, for defining volumes, etc.*/
    pub visibility: crate::foundation::TfToken,
    /**Purpose is a classification of geometry into categories that
can each be independently included or excluded from traversals of prims
on a stage, such as rendering or bounding-box computation traversals.

    See \UsdGeom_ImageablePurpose for more detail about how
\*purpose is computed and used.*/
    pub purpose: crate::foundation::TfToken,
    /**A full affine matrix transform op. When authored, this prim should
list "xformOp:transform" in xformOpOrder so consumers can compose the
local-to-parent matrix correctly.*/
    pub xform_op_transform: Option<crate::foundation::GfMatrix4d>,
    /**Encodes the sequence of transformation operations in the
order in which they should be pushed onto a transform stack while
visiting a UsdStage's prims in a graph traversal that will effect
the desired positioning for this prim and its descendant prims.

You should rarely, if ever, need to manipulate this attribute directly.
It is managed by the AddXformOp(), SetResetXformStack(), and
SetXformOpOrder(), and consulted by GetOrderedXformOps() and
GetLocalTransformation().*/
    pub xform_op_order: crate::foundation::VtArray<crate::foundation::TfToken>,
}
impl Default for UsdGeomXform {
    fn default() -> Self {
        Self {
            visibility: crate::foundation::TfToken::new("inherited"),
            purpose: crate::foundation::TfToken::new("default"),
            xform_op_transform: None,
            xform_op_order: Vec::new(),
        }
    }
}
impl crate::schema::generated::UsdSchemaInfo for UsdGeomXform {
    fn schema_name(&self) -> &'static str {
        "UsdGeomXform"
    }
    fn attribute_metadata(
        &self,
    ) -> &'static [crate::schema::generated::AttributeMetadata] {
        static META: &[crate::schema::generated::AttributeMetadata] = &[
            crate::schema::generated::AttributeMetadata {
                usd_name: "visibility",
                usd_type: "token",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "purpose",
                usd_type: "token",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "xformOp:transform",
                usd_type: "matrix4d",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "xformOpOrder",
                usd_type: "token[]",
                is_uniform: true,
            },
        ];
        META
    }
}
/**Defines a primitive rectilinear cube centered at the origin.

The fallback values for Cube, Sphere, Cone, and Cylinder are set so that
they all pack into the same volume/bounds.*/
#[derive(Clone, Debug)]
pub struct UsdGeomCube {
    /**Visibility is meant to be the simplest form of "pruning"
visibility that is supported by most DCC apps.  Visibility is
animatable, allowing a sub-tree of geometry to be present for some
segment of a shot, and absent from others; unlike the action of
deactivating geometry prims, invisible geometry is still
available for inspection, for positioning, for defining volumes, etc.*/
    pub visibility: crate::foundation::TfToken,
    /**Purpose is a classification of geometry into categories that
can each be independently included or excluded from traversals of prims
on a stage, such as rendering or bounding-box computation traversals.

See \UsdGeom_ImageablePurpose for more detail about how
\*purpose is computed and used.*/
    pub purpose: crate::foundation::TfToken,
    /**Encodes the sequence of transformation operations in the
order in which they should be pushed onto a transform stack while
visiting a UsdStage's prims in a graph traversal that will effect
the desired positioning for this prim and its descendant prims.

You should rarely, if ever, need to manipulate this attribute directly.
It is managed by the AddXformOp(), SetResetXformStack(), and
SetXformOpOrder(), and consulted by GetOrderedXformOps() and
GetLocalTransformation().*/
    pub xform_op_order: crate::foundation::VtArray<crate::foundation::TfToken>,
    /**Extent is a three dimensional range measuring the geometric
extent of the authored gprim in its own local space (i.e. its own
transform not applied), \*without accounting for any shader-induced
displacement. If __any__ extent value has been authored for a given
Boundable, then it should be authored at every timeSample at which
geometry-affecting properties are authored, to ensure correct
evaluation via ComputeExtent(). If __no__ extent value has been
authored, then ComputeExtent() will call the Boundable's registered
ComputeExtentFunction(), which may be expensive, which is why we
strongly encourage proper authoring of extent.
\See: ComputeExtent()
\See: \UsdGeom_Boundable_Extent.

An authored extent on a prim which has children is expected to include
the extent of all children, as they will be pruned from BBox computation
during traversal.*/
    pub extent: crate::foundation::VtArray<crate::foundation::GfVec3f>,
    /**It is useful to have an "official" colorSet that can be used
as a display or modeling color, even in the absence of any specified
shader for a gprim.  DisplayColor serves this role; because it is a
UsdGeomPrimvar, it can also be used as a gprim override for any shader
that consumes a \*displayColor parameter.*/
    pub primvars_display_color: crate::foundation::VtArray<crate::foundation::GfVec3f>,
    /**Companion to \*displayColor that specifies opacity, broken
out as an independent attribute rather than an rgba color, both so that
each can be independently overridden, and because shaders rarely consume
rgba parameters.*/
    pub primvars_display_opacity: crate::foundation::VtArray<f32>,
    /**Although some renderers treat all parametric or polygonal
surfaces as if they were effectively laminae with outward-facing
normals on both sides, some renderers derive significant optimizations
by considering these surfaces to have only a single outward side,
typically determined by control-point winding order and/or
\*orientation.  By doing so they can perform "backface culling" to
avoid drawing the many polygons of most closed surfaces that face away
from the viewer.

However, it is often advantageous to model thin objects such as paper
and cloth as single, open surfaces that must be viewable from both
sides, always.  Setting a gprim's \*doubleSided attribute to
\\c true instructs all renderers to disable optimizations such as
backface culling for the gprim, and attempt (not all renderers are able
to do so, but the USD reference GL renderer always will) to provide
forward-facing normals on each side of the surface for lighting
calculations.*/
    pub double_sided: bool,
    /**Orientation specifies whether the gprim's surface normal
should be computed using the right hand rule, or the left hand rule.
Please see \UsdGeom_WindingOrder for a deeper explanation and
generalization of orientation to composed scenes with transformation
hierarchies.*/
    pub orientation: crate::foundation::TfToken,
    /**Indicates the length of each edge of the cube.  If you
author \*size you must also author \*extent.

\See: GetExtentAttr()*/
    pub size: f64,
}
impl Default for UsdGeomCube {
    fn default() -> Self {
        Self {
            visibility: crate::foundation::TfToken::new("inherited"),
            purpose: crate::foundation::TfToken::new("default"),
            xform_op_order: Vec::new(),
            extent: Vec::new(),
            primvars_display_color: Vec::new(),
            primvars_display_opacity: Vec::new(),
            double_sided: false,
            orientation: crate::foundation::TfToken::new("rightHanded"),
            size: 2f64,
        }
    }
}
impl crate::schema::generated::UsdSchemaInfo for UsdGeomCube {
    fn schema_name(&self) -> &'static str {
        "UsdGeomCube"
    }
    fn attribute_metadata(
        &self,
    ) -> &'static [crate::schema::generated::AttributeMetadata] {
        static META: &[crate::schema::generated::AttributeMetadata] = &[
            crate::schema::generated::AttributeMetadata {
                usd_name: "visibility",
                usd_type: "token",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "purpose",
                usd_type: "token",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "xformOpOrder",
                usd_type: "token[]",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "extent",
                usd_type: "float3[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "primvars:displayColor",
                usd_type: "color3f[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "primvars:displayOpacity",
                usd_type: "float[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "doubleSided",
                usd_type: "bool",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "orientation",
                usd_type: "token",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "size",
                usd_type: "double",
                is_uniform: false,
            },
        ];
        META
    }
}
/**Defines a primitive sphere centered at the origin.

The fallback values for Cube, Sphere, Cone, and Cylinder are set so that
they all pack into the same volume/bounds.*/
#[derive(Clone, Debug)]
pub struct UsdGeomSphere {
    /**Visibility is meant to be the simplest form of "pruning"
visibility that is supported by most DCC apps.  Visibility is
animatable, allowing a sub-tree of geometry to be present for some
segment of a shot, and absent from others; unlike the action of
deactivating geometry prims, invisible geometry is still
available for inspection, for positioning, for defining volumes, etc.*/
    pub visibility: crate::foundation::TfToken,
    /**Purpose is a classification of geometry into categories that
can each be independently included or excluded from traversals of prims
on a stage, such as rendering or bounding-box computation traversals.

See \UsdGeom_ImageablePurpose for more detail about how
\*purpose is computed and used.*/
    pub purpose: crate::foundation::TfToken,
    /**Encodes the sequence of transformation operations in the
order in which they should be pushed onto a transform stack while
visiting a UsdStage's prims in a graph traversal that will effect
the desired positioning for this prim and its descendant prims.

You should rarely, if ever, need to manipulate this attribute directly.
It is managed by the AddXformOp(), SetResetXformStack(), and
SetXformOpOrder(), and consulted by GetOrderedXformOps() and
GetLocalTransformation().*/
    pub xform_op_order: crate::foundation::VtArray<crate::foundation::TfToken>,
    /**Extent is a three dimensional range measuring the geometric
extent of the authored gprim in its own local space (i.e. its own
transform not applied), \*without accounting for any shader-induced
displacement. If __any__ extent value has been authored for a given
Boundable, then it should be authored at every timeSample at which
geometry-affecting properties are authored, to ensure correct
evaluation via ComputeExtent(). If __no__ extent value has been
authored, then ComputeExtent() will call the Boundable's registered
ComputeExtentFunction(), which may be expensive, which is why we
strongly encourage proper authoring of extent.
\See: ComputeExtent()
\See: \UsdGeom_Boundable_Extent.

An authored extent on a prim which has children is expected to include
the extent of all children, as they will be pruned from BBox computation
during traversal.*/
    pub extent: crate::foundation::VtArray<crate::foundation::GfVec3f>,
    /**It is useful to have an "official" colorSet that can be used
as a display or modeling color, even in the absence of any specified
shader for a gprim.  DisplayColor serves this role; because it is a
UsdGeomPrimvar, it can also be used as a gprim override for any shader
that consumes a \*displayColor parameter.*/
    pub primvars_display_color: crate::foundation::VtArray<crate::foundation::GfVec3f>,
    /**Companion to \*displayColor that specifies opacity, broken
out as an independent attribute rather than an rgba color, both so that
each can be independently overridden, and because shaders rarely consume
rgba parameters.*/
    pub primvars_display_opacity: crate::foundation::VtArray<f32>,
    /**Although some renderers treat all parametric or polygonal
surfaces as if they were effectively laminae with outward-facing
normals on both sides, some renderers derive significant optimizations
by considering these surfaces to have only a single outward side,
typically determined by control-point winding order and/or
\*orientation.  By doing so they can perform "backface culling" to
avoid drawing the many polygons of most closed surfaces that face away
from the viewer.

However, it is often advantageous to model thin objects such as paper
and cloth as single, open surfaces that must be viewable from both
sides, always.  Setting a gprim's \*doubleSided attribute to
\\c true instructs all renderers to disable optimizations such as
backface culling for the gprim, and attempt (not all renderers are able
to do so, but the USD reference GL renderer always will) to provide
forward-facing normals on each side of the surface for lighting
calculations.*/
    pub double_sided: bool,
    /**Orientation specifies whether the gprim's surface normal
should be computed using the right hand rule, or the left hand rule.
Please see \UsdGeom_WindingOrder for a deeper explanation and
generalization of orientation to composed scenes with transformation
hierarchies.*/
    pub orientation: crate::foundation::TfToken,
    /**Indicates the sphere's radius.  If you
author \*radius you must also author \*extent.

\See: GetExtentAttr()*/
    pub radius: f64,
}
impl Default for UsdGeomSphere {
    fn default() -> Self {
        Self {
            visibility: crate::foundation::TfToken::new("inherited"),
            purpose: crate::foundation::TfToken::new("default"),
            xform_op_order: Vec::new(),
            extent: Vec::new(),
            primvars_display_color: Vec::new(),
            primvars_display_opacity: Vec::new(),
            double_sided: false,
            orientation: crate::foundation::TfToken::new("rightHanded"),
            radius: 1f64,
        }
    }
}
impl crate::schema::generated::UsdSchemaInfo for UsdGeomSphere {
    fn schema_name(&self) -> &'static str {
        "UsdGeomSphere"
    }
    fn attribute_metadata(
        &self,
    ) -> &'static [crate::schema::generated::AttributeMetadata] {
        static META: &[crate::schema::generated::AttributeMetadata] = &[
            crate::schema::generated::AttributeMetadata {
                usd_name: "visibility",
                usd_type: "token",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "purpose",
                usd_type: "token",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "xformOpOrder",
                usd_type: "token[]",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "extent",
                usd_type: "float3[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "primvars:displayColor",
                usd_type: "color3f[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "primvars:displayOpacity",
                usd_type: "float[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "doubleSided",
                usd_type: "bool",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "orientation",
                usd_type: "token",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "radius",
                usd_type: "double",
                is_uniform: false,
            },
        ];
        META
    }
}
/**Defines a primitive cylinder with closed ends, centered at the
origin, whose spine is along the specified \*axis.

The fallback values for Cube, Sphere, Cone, and Cylinder are set so that
they all pack into the same volume/bounds.*/
#[derive(Clone, Debug)]
pub struct UsdGeomCylinder {
    /**Visibility is meant to be the simplest form of "pruning"
visibility that is supported by most DCC apps.  Visibility is
animatable, allowing a sub-tree of geometry to be present for some
segment of a shot, and absent from others; unlike the action of
deactivating geometry prims, invisible geometry is still
available for inspection, for positioning, for defining volumes, etc.*/
    pub visibility: crate::foundation::TfToken,
    /**Purpose is a classification of geometry into categories that
can each be independently included or excluded from traversals of prims
on a stage, such as rendering or bounding-box computation traversals.

See \UsdGeom_ImageablePurpose for more detail about how
\*purpose is computed and used.*/
    pub purpose: crate::foundation::TfToken,
    /**Encodes the sequence of transformation operations in the
order in which they should be pushed onto a transform stack while
visiting a UsdStage's prims in a graph traversal that will effect
the desired positioning for this prim and its descendant prims.

You should rarely, if ever, need to manipulate this attribute directly.
It is managed by the AddXformOp(), SetResetXformStack(), and
SetXformOpOrder(), and consulted by GetOrderedXformOps() and
GetLocalTransformation().*/
    pub xform_op_order: crate::foundation::VtArray<crate::foundation::TfToken>,
    /**Extent is a three dimensional range measuring the geometric
extent of the authored gprim in its own local space (i.e. its own
transform not applied), \*without accounting for any shader-induced
displacement. If __any__ extent value has been authored for a given
Boundable, then it should be authored at every timeSample at which
geometry-affecting properties are authored, to ensure correct
evaluation via ComputeExtent(). If __no__ extent value has been
authored, then ComputeExtent() will call the Boundable's registered
ComputeExtentFunction(), which may be expensive, which is why we
strongly encourage proper authoring of extent.
\See: ComputeExtent()
\See: \UsdGeom_Boundable_Extent.

An authored extent on a prim which has children is expected to include
the extent of all children, as they will be pruned from BBox computation
during traversal.*/
    pub extent: crate::foundation::VtArray<crate::foundation::GfVec3f>,
    /**It is useful to have an "official" colorSet that can be used
as a display or modeling color, even in the absence of any specified
shader for a gprim.  DisplayColor serves this role; because it is a
UsdGeomPrimvar, it can also be used as a gprim override for any shader
that consumes a \*displayColor parameter.*/
    pub primvars_display_color: crate::foundation::VtArray<crate::foundation::GfVec3f>,
    /**Companion to \*displayColor that specifies opacity, broken
out as an independent attribute rather than an rgba color, both so that
each can be independently overridden, and because shaders rarely consume
rgba parameters.*/
    pub primvars_display_opacity: crate::foundation::VtArray<f32>,
    /**Although some renderers treat all parametric or polygonal
surfaces as if they were effectively laminae with outward-facing
normals on both sides, some renderers derive significant optimizations
by considering these surfaces to have only a single outward side,
typically determined by control-point winding order and/or
\*orientation.  By doing so they can perform "backface culling" to
avoid drawing the many polygons of most closed surfaces that face away
from the viewer.

However, it is often advantageous to model thin objects such as paper
and cloth as single, open surfaces that must be viewable from both
sides, always.  Setting a gprim's \*doubleSided attribute to
\\c true instructs all renderers to disable optimizations such as
backface culling for the gprim, and attempt (not all renderers are able
to do so, but the USD reference GL renderer always will) to provide
forward-facing normals on each side of the surface for lighting
calculations.*/
    pub double_sided: bool,
    /**Orientation specifies whether the gprim's surface normal
should be computed using the right hand rule, or the left hand rule.
Please see \UsdGeom_WindingOrder for a deeper explanation and
generalization of orientation to composed scenes with transformation
hierarchies.*/
    pub orientation: crate::foundation::TfToken,
    /**The size of the cylinder's spine along the specified
\*axis.  If you author \*height you must also author \*extent.

\See: GetExtentAttr()*/
    pub height: f64,
    /**The radius of the cylinder. If you author \*radius
you must also author \*extent.

\See: GetExtentAttr()*/
    pub radius: f64,
    ///The axis along which the spine of the cylinder is aligned
    pub axis: crate::foundation::TfToken,
}
impl Default for UsdGeomCylinder {
    fn default() -> Self {
        Self {
            visibility: crate::foundation::TfToken::new("inherited"),
            purpose: crate::foundation::TfToken::new("default"),
            xform_op_order: Vec::new(),
            extent: Vec::new(),
            primvars_display_color: Vec::new(),
            primvars_display_opacity: Vec::new(),
            double_sided: false,
            orientation: crate::foundation::TfToken::new("rightHanded"),
            height: 2f64,
            radius: 1f64,
            axis: crate::foundation::TfToken::new("Z"),
        }
    }
}
impl crate::schema::generated::UsdSchemaInfo for UsdGeomCylinder {
    fn schema_name(&self) -> &'static str {
        "UsdGeomCylinder"
    }
    fn attribute_metadata(
        &self,
    ) -> &'static [crate::schema::generated::AttributeMetadata] {
        static META: &[crate::schema::generated::AttributeMetadata] = &[
            crate::schema::generated::AttributeMetadata {
                usd_name: "visibility",
                usd_type: "token",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "purpose",
                usd_type: "token",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "xformOpOrder",
                usd_type: "token[]",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "extent",
                usd_type: "float3[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "primvars:displayColor",
                usd_type: "color3f[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "primvars:displayOpacity",
                usd_type: "float[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "doubleSided",
                usd_type: "bool",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "orientation",
                usd_type: "token",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "height",
                usd_type: "double",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "radius",
                usd_type: "double",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "axis",
                usd_type: "token",
                is_uniform: true,
            },
        ];
        META
    }
}
/**Defines a primitive capsule, i.e. a cylinder capped by two half
spheres, centered at the origin, whose spine is along the specified
\*axis.
The spherical cap heights (sagitta) of the two endcaps are a function of
the relative radii of the endcaps, such that cylinder tangent and sphere
tangent are coincident and maintain C1 continuity.*/
#[derive(Clone, Debug)]
pub struct UsdGeomCapsule {
    /**Visibility is meant to be the simplest form of "pruning"
visibility that is supported by most DCC apps.  Visibility is
animatable, allowing a sub-tree of geometry to be present for some
segment of a shot, and absent from others; unlike the action of
deactivating geometry prims, invisible geometry is still
available for inspection, for positioning, for defining volumes, etc.*/
    pub visibility: crate::foundation::TfToken,
    /**Purpose is a classification of geometry into categories that
can each be independently included or excluded from traversals of prims
on a stage, such as rendering or bounding-box computation traversals.

See \UsdGeom_ImageablePurpose for more detail about how
\*purpose is computed and used.*/
    pub purpose: crate::foundation::TfToken,
    /**Encodes the sequence of transformation operations in the
order in which they should be pushed onto a transform stack while
visiting a UsdStage's prims in a graph traversal that will effect
the desired positioning for this prim and its descendant prims.

You should rarely, if ever, need to manipulate this attribute directly.
It is managed by the AddXformOp(), SetResetXformStack(), and
SetXformOpOrder(), and consulted by GetOrderedXformOps() and
GetLocalTransformation().*/
    pub xform_op_order: crate::foundation::VtArray<crate::foundation::TfToken>,
    /**Extent is a three dimensional range measuring the geometric
extent of the authored gprim in its own local space (i.e. its own
transform not applied), \*without accounting for any shader-induced
displacement. If __any__ extent value has been authored for a given
Boundable, then it should be authored at every timeSample at which
geometry-affecting properties are authored, to ensure correct
evaluation via ComputeExtent(). If __no__ extent value has been
authored, then ComputeExtent() will call the Boundable's registered
ComputeExtentFunction(), which may be expensive, which is why we
strongly encourage proper authoring of extent.
\See: ComputeExtent()
\See: \UsdGeom_Boundable_Extent.

An authored extent on a prim which has children is expected to include
the extent of all children, as they will be pruned from BBox computation
during traversal.*/
    pub extent: crate::foundation::VtArray<crate::foundation::GfVec3f>,
    /**It is useful to have an "official" colorSet that can be used
as a display or modeling color, even in the absence of any specified
shader for a gprim.  DisplayColor serves this role; because it is a
UsdGeomPrimvar, it can also be used as a gprim override for any shader
that consumes a \*displayColor parameter.*/
    pub primvars_display_color: crate::foundation::VtArray<crate::foundation::GfVec3f>,
    /**Companion to \*displayColor that specifies opacity, broken
out as an independent attribute rather than an rgba color, both so that
each can be independently overridden, and because shaders rarely consume
rgba parameters.*/
    pub primvars_display_opacity: crate::foundation::VtArray<f32>,
    /**Although some renderers treat all parametric or polygonal
surfaces as if they were effectively laminae with outward-facing
normals on both sides, some renderers derive significant optimizations
by considering these surfaces to have only a single outward side,
typically determined by control-point winding order and/or
\*orientation.  By doing so they can perform "backface culling" to
avoid drawing the many polygons of most closed surfaces that face away
from the viewer.

However, it is often advantageous to model thin objects such as paper
and cloth as single, open surfaces that must be viewable from both
sides, always.  Setting a gprim's \*doubleSided attribute to
\\c true instructs all renderers to disable optimizations such as
backface culling for the gprim, and attempt (not all renderers are able
to do so, but the USD reference GL renderer always will) to provide
forward-facing normals on each side of the surface for lighting
calculations.*/
    pub double_sided: bool,
    /**Orientation specifies whether the gprim's surface normal
should be computed using the right hand rule, or the left hand rule.
Please see \UsdGeom_WindingOrder for a deeper explanation and
generalization of orientation to composed scenes with transformation
hierarchies.*/
    pub orientation: crate::foundation::TfToken,
    /**The length of the capsule's spine along the specified
\*axis excluding the size of the two half spheres, i.e.
the length of the cylinder portion of the capsule.
If you author \*height you must also author \*extent.
\See: GetExtentAttr()*/
    pub height: f64,
    /**The radius of the capsule.  If you
author \*radius you must also author \*extent.

\See: GetExtentAttr()*/
    pub radius: f64,
    ///The axis along which the spine of the capsule is aligned
    pub axis: crate::foundation::TfToken,
}
impl Default for UsdGeomCapsule {
    fn default() -> Self {
        Self {
            visibility: crate::foundation::TfToken::new("inherited"),
            purpose: crate::foundation::TfToken::new("default"),
            xform_op_order: Vec::new(),
            extent: Vec::new(),
            primvars_display_color: Vec::new(),
            primvars_display_opacity: Vec::new(),
            double_sided: false,
            orientation: crate::foundation::TfToken::new("rightHanded"),
            height: 1f64,
            radius: 0.5f64,
            axis: crate::foundation::TfToken::new("Z"),
        }
    }
}
impl crate::schema::generated::UsdSchemaInfo for UsdGeomCapsule {
    fn schema_name(&self) -> &'static str {
        "UsdGeomCapsule"
    }
    fn attribute_metadata(
        &self,
    ) -> &'static [crate::schema::generated::AttributeMetadata] {
        static META: &[crate::schema::generated::AttributeMetadata] = &[
            crate::schema::generated::AttributeMetadata {
                usd_name: "visibility",
                usd_type: "token",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "purpose",
                usd_type: "token",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "xformOpOrder",
                usd_type: "token[]",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "extent",
                usd_type: "float3[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "primvars:displayColor",
                usd_type: "color3f[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "primvars:displayOpacity",
                usd_type: "float[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "doubleSided",
                usd_type: "bool",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "orientation",
                usd_type: "token",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "height",
                usd_type: "double",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "radius",
                usd_type: "double",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "axis",
                usd_type: "token",
                is_uniform: true,
            },
        ];
        META
    }
}
/**Defines a primitive cone, centered at the origin, whose spine
is along the specified \*axis, with the apex of the cone pointing
in the direction of the positive axis.

The fallback values for Cube, Sphere, Cone, and Cylinder are set so that
they all pack into the same volume/bounds.*/
#[derive(Clone, Debug)]
pub struct UsdGeomCone {
    /**Visibility is meant to be the simplest form of "pruning"
visibility that is supported by most DCC apps.  Visibility is
animatable, allowing a sub-tree of geometry to be present for some
segment of a shot, and absent from others; unlike the action of
deactivating geometry prims, invisible geometry is still
available for inspection, for positioning, for defining volumes, etc.*/
    pub visibility: crate::foundation::TfToken,
    /**Purpose is a classification of geometry into categories that
can each be independently included or excluded from traversals of prims
on a stage, such as rendering or bounding-box computation traversals.

See \UsdGeom_ImageablePurpose for more detail about how
\*purpose is computed and used.*/
    pub purpose: crate::foundation::TfToken,
    /**Encodes the sequence of transformation operations in the
order in which they should be pushed onto a transform stack while
visiting a UsdStage's prims in a graph traversal that will effect
the desired positioning for this prim and its descendant prims.

You should rarely, if ever, need to manipulate this attribute directly.
It is managed by the AddXformOp(), SetResetXformStack(), and
SetXformOpOrder(), and consulted by GetOrderedXformOps() and
GetLocalTransformation().*/
    pub xform_op_order: crate::foundation::VtArray<crate::foundation::TfToken>,
    /**Extent is a three dimensional range measuring the geometric
extent of the authored gprim in its own local space (i.e. its own
transform not applied), \*without accounting for any shader-induced
displacement. If __any__ extent value has been authored for a given
Boundable, then it should be authored at every timeSample at which
geometry-affecting properties are authored, to ensure correct
evaluation via ComputeExtent(). If __no__ extent value has been
authored, then ComputeExtent() will call the Boundable's registered
ComputeExtentFunction(), which may be expensive, which is why we
strongly encourage proper authoring of extent.
\See: ComputeExtent()
\See: \UsdGeom_Boundable_Extent.

An authored extent on a prim which has children is expected to include
the extent of all children, as they will be pruned from BBox computation
during traversal.*/
    pub extent: crate::foundation::VtArray<crate::foundation::GfVec3f>,
    /**It is useful to have an "official" colorSet that can be used
as a display or modeling color, even in the absence of any specified
shader for a gprim.  DisplayColor serves this role; because it is a
UsdGeomPrimvar, it can also be used as a gprim override for any shader
that consumes a \*displayColor parameter.*/
    pub primvars_display_color: crate::foundation::VtArray<crate::foundation::GfVec3f>,
    /**Companion to \*displayColor that specifies opacity, broken
out as an independent attribute rather than an rgba color, both so that
each can be independently overridden, and because shaders rarely consume
rgba parameters.*/
    pub primvars_display_opacity: crate::foundation::VtArray<f32>,
    /**Although some renderers treat all parametric or polygonal
surfaces as if they were effectively laminae with outward-facing
normals on both sides, some renderers derive significant optimizations
by considering these surfaces to have only a single outward side,
typically determined by control-point winding order and/or
\*orientation.  By doing so they can perform "backface culling" to
avoid drawing the many polygons of most closed surfaces that face away
from the viewer.

However, it is often advantageous to model thin objects such as paper
and cloth as single, open surfaces that must be viewable from both
sides, always.  Setting a gprim's \*doubleSided attribute to
\\c true instructs all renderers to disable optimizations such as
backface culling for the gprim, and attempt (not all renderers are able
to do so, but the USD reference GL renderer always will) to provide
forward-facing normals on each side of the surface for lighting
calculations.*/
    pub double_sided: bool,
    /**Orientation specifies whether the gprim's surface normal
should be computed using the right hand rule, or the left hand rule.
Please see \UsdGeom_WindingOrder for a deeper explanation and
generalization of orientation to composed scenes with transformation
hierarchies.*/
    pub orientation: crate::foundation::TfToken,
    /**The length of the cone's spine along the specified
\*axis.  If you author \*height you must also author \*extent.

\See: GetExtentAttr()*/
    pub height: f64,
    /**The radius of the cone.  If you
author \*radius you must also author \*extent.

\See: GetExtentAttr()*/
    pub radius: f64,
    ///The axis along which the spine of the cone is aligned
    pub axis: crate::foundation::TfToken,
}
impl Default for UsdGeomCone {
    fn default() -> Self {
        Self {
            visibility: crate::foundation::TfToken::new("inherited"),
            purpose: crate::foundation::TfToken::new("default"),
            xform_op_order: Vec::new(),
            extent: Vec::new(),
            primvars_display_color: Vec::new(),
            primvars_display_opacity: Vec::new(),
            double_sided: false,
            orientation: crate::foundation::TfToken::new("rightHanded"),
            height: 2f64,
            radius: 1f64,
            axis: crate::foundation::TfToken::new("Z"),
        }
    }
}
impl crate::schema::generated::UsdSchemaInfo for UsdGeomCone {
    fn schema_name(&self) -> &'static str {
        "UsdGeomCone"
    }
    fn attribute_metadata(
        &self,
    ) -> &'static [crate::schema::generated::AttributeMetadata] {
        static META: &[crate::schema::generated::AttributeMetadata] = &[
            crate::schema::generated::AttributeMetadata {
                usd_name: "visibility",
                usd_type: "token",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "purpose",
                usd_type: "token",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "xformOpOrder",
                usd_type: "token[]",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "extent",
                usd_type: "float3[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "primvars:displayColor",
                usd_type: "color3f[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "primvars:displayOpacity",
                usd_type: "float[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "doubleSided",
                usd_type: "bool",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "orientation",
                usd_type: "token",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "height",
                usd_type: "double",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "radius",
                usd_type: "double",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "axis",
                usd_type: "token",
                is_uniform: true,
            },
        ];
        META
    }
}
/**Defines a primitive cylinder with closed ends, centered at the
origin, whose spine is along the specified \*axis, with a pair of radii
describing the size of the end points.

The fallback values for Cube, Sphere, Cone, and Cylinder are set so that
they all pack into the same volume/bounds.*/
#[derive(Clone, Debug)]
pub struct UsdGeomCylinder_1 {
    /**Visibility is meant to be the simplest form of "pruning"
visibility that is supported by most DCC apps.  Visibility is
animatable, allowing a sub-tree of geometry to be present for some
segment of a shot, and absent from others; unlike the action of
deactivating geometry prims, invisible geometry is still
available for inspection, for positioning, for defining volumes, etc.*/
    pub visibility: crate::foundation::TfToken,
    /**Purpose is a classification of geometry into categories that
can each be independently included or excluded from traversals of prims
on a stage, such as rendering or bounding-box computation traversals.

See \UsdGeom_ImageablePurpose for more detail about how
\*purpose is computed and used.*/
    pub purpose: crate::foundation::TfToken,
    /**Encodes the sequence of transformation operations in the
order in which they should be pushed onto a transform stack while
visiting a UsdStage's prims in a graph traversal that will effect
the desired positioning for this prim and its descendant prims.

You should rarely, if ever, need to manipulate this attribute directly.
It is managed by the AddXformOp(), SetResetXformStack(), and
SetXformOpOrder(), and consulted by GetOrderedXformOps() and
GetLocalTransformation().*/
    pub xform_op_order: crate::foundation::VtArray<crate::foundation::TfToken>,
    /**Extent is a three dimensional range measuring the geometric
extent of the authored gprim in its own local space (i.e. its own
transform not applied), \*without accounting for any shader-induced
displacement. If __any__ extent value has been authored for a given
Boundable, then it should be authored at every timeSample at which
geometry-affecting properties are authored, to ensure correct
evaluation via ComputeExtent(). If __no__ extent value has been
authored, then ComputeExtent() will call the Boundable's registered
ComputeExtentFunction(), which may be expensive, which is why we
strongly encourage proper authoring of extent.
\See: ComputeExtent()
\See: \UsdGeom_Boundable_Extent.

An authored extent on a prim which has children is expected to include
the extent of all children, as they will be pruned from BBox computation
during traversal.*/
    pub extent: crate::foundation::VtArray<crate::foundation::GfVec3f>,
    /**It is useful to have an "official" colorSet that can be used
as a display or modeling color, even in the absence of any specified
shader for a gprim.  DisplayColor serves this role; because it is a
UsdGeomPrimvar, it can also be used as a gprim override for any shader
that consumes a \*displayColor parameter.*/
    pub primvars_display_color: crate::foundation::VtArray<crate::foundation::GfVec3f>,
    /**Companion to \*displayColor that specifies opacity, broken
out as an independent attribute rather than an rgba color, both so that
each can be independently overridden, and because shaders rarely consume
rgba parameters.*/
    pub primvars_display_opacity: crate::foundation::VtArray<f32>,
    /**Although some renderers treat all parametric or polygonal
surfaces as if they were effectively laminae with outward-facing
normals on both sides, some renderers derive significant optimizations
by considering these surfaces to have only a single outward side,
typically determined by control-point winding order and/or
\*orientation.  By doing so they can perform "backface culling" to
avoid drawing the many polygons of most closed surfaces that face away
from the viewer.

However, it is often advantageous to model thin objects such as paper
and cloth as single, open surfaces that must be viewable from both
sides, always.  Setting a gprim's \*doubleSided attribute to
\\c true instructs all renderers to disable optimizations such as
backface culling for the gprim, and attempt (not all renderers are able
to do so, but the USD reference GL renderer always will) to provide
forward-facing normals on each side of the surface for lighting
calculations.*/
    pub double_sided: bool,
    /**Orientation specifies whether the gprim's surface normal
should be computed using the right hand rule, or the left hand rule.
Please see \UsdGeom_WindingOrder for a deeper explanation and
generalization of orientation to composed scenes with transformation
hierarchies.*/
    pub orientation: crate::foundation::TfToken,
    /**The length of the cylinder's spine along the specified
\*axis.  If you author \*height you must also author \*extent.

\See: GetExtentAttr()*/
    pub height: f64,
    /**The radius of the top of the cylinder - i.e. the face located
along the positive \*axis. If you author \*radiusTop you must also
author \*extent.

\See: GetExtentAttr()*/
    pub radius_top: f64,
    /**The radius of the bottom of the cylinder - i.e. the face
point located along the negative \*axis. If you author
\*radiusBottom you must also author \*extent.

\See: GetExtentAttr()*/
    pub radius_bottom: f64,
    ///The axis along which the spine of the cylinder is aligned
    pub axis: crate::foundation::TfToken,
}
impl Default for UsdGeomCylinder_1 {
    fn default() -> Self {
        Self {
            visibility: crate::foundation::TfToken::new("inherited"),
            purpose: crate::foundation::TfToken::new("default"),
            xform_op_order: Vec::new(),
            extent: Vec::new(),
            primvars_display_color: Vec::new(),
            primvars_display_opacity: Vec::new(),
            double_sided: false,
            orientation: crate::foundation::TfToken::new("rightHanded"),
            height: 2f64,
            radius_top: 1f64,
            radius_bottom: 1f64,
            axis: crate::foundation::TfToken::new("Z"),
        }
    }
}
impl crate::schema::generated::UsdSchemaInfo for UsdGeomCylinder_1 {
    fn schema_name(&self) -> &'static str {
        "UsdGeomCylinder_1"
    }
    fn attribute_metadata(
        &self,
    ) -> &'static [crate::schema::generated::AttributeMetadata] {
        static META: &[crate::schema::generated::AttributeMetadata] = &[
            crate::schema::generated::AttributeMetadata {
                usd_name: "visibility",
                usd_type: "token",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "purpose",
                usd_type: "token",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "xformOpOrder",
                usd_type: "token[]",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "extent",
                usd_type: "float3[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "primvars:displayColor",
                usd_type: "color3f[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "primvars:displayOpacity",
                usd_type: "float[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "doubleSided",
                usd_type: "bool",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "orientation",
                usd_type: "token",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "height",
                usd_type: "double",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "radiusTop",
                usd_type: "double",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "radiusBottom",
                usd_type: "double",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "axis",
                usd_type: "token",
                is_uniform: true,
            },
        ];
        META
    }
}
/**Defines a primitive capsule, i.e. a cylinder capped by two half
spheres, with potentially different radii, centered at the origin, and whose
spine is along the specified \*axis.
The spherical cap heights (sagitta) of the two endcaps are a function of
the relative radii of the endcaps, such that cylinder tangent and sphere
tangent are coincident and maintain C1 continuity.*/
#[derive(Clone, Debug)]
pub struct UsdGeomCapsule_1 {
    /**Visibility is meant to be the simplest form of "pruning"
visibility that is supported by most DCC apps.  Visibility is
animatable, allowing a sub-tree of geometry to be present for some
segment of a shot, and absent from others; unlike the action of
deactivating geometry prims, invisible geometry is still
available for inspection, for positioning, for defining volumes, etc.*/
    pub visibility: crate::foundation::TfToken,
    /**Purpose is a classification of geometry into categories that
can each be independently included or excluded from traversals of prims
on a stage, such as rendering or bounding-box computation traversals.

See \UsdGeom_ImageablePurpose for more detail about how
\*purpose is computed and used.*/
    pub purpose: crate::foundation::TfToken,
    /**Encodes the sequence of transformation operations in the
order in which they should be pushed onto a transform stack while
visiting a UsdStage's prims in a graph traversal that will effect
the desired positioning for this prim and its descendant prims.

You should rarely, if ever, need to manipulate this attribute directly.
It is managed by the AddXformOp(), SetResetXformStack(), and
SetXformOpOrder(), and consulted by GetOrderedXformOps() and
GetLocalTransformation().*/
    pub xform_op_order: crate::foundation::VtArray<crate::foundation::TfToken>,
    /**Extent is a three dimensional range measuring the geometric
extent of the authored gprim in its own local space (i.e. its own
transform not applied), \*without accounting for any shader-induced
displacement. If __any__ extent value has been authored for a given
Boundable, then it should be authored at every timeSample at which
geometry-affecting properties are authored, to ensure correct
evaluation via ComputeExtent(). If __no__ extent value has been
authored, then ComputeExtent() will call the Boundable's registered
ComputeExtentFunction(), which may be expensive, which is why we
strongly encourage proper authoring of extent.
\See: ComputeExtent()
\See: \UsdGeom_Boundable_Extent.

An authored extent on a prim which has children is expected to include
the extent of all children, as they will be pruned from BBox computation
during traversal.*/
    pub extent: crate::foundation::VtArray<crate::foundation::GfVec3f>,
    /**It is useful to have an "official" colorSet that can be used
as a display or modeling color, even in the absence of any specified
shader for a gprim.  DisplayColor serves this role; because it is a
UsdGeomPrimvar, it can also be used as a gprim override for any shader
that consumes a \*displayColor parameter.*/
    pub primvars_display_color: crate::foundation::VtArray<crate::foundation::GfVec3f>,
    /**Companion to \*displayColor that specifies opacity, broken
out as an independent attribute rather than an rgba color, both so that
each can be independently overridden, and because shaders rarely consume
rgba parameters.*/
    pub primvars_display_opacity: crate::foundation::VtArray<f32>,
    /**Although some renderers treat all parametric or polygonal
surfaces as if they were effectively laminae with outward-facing
normals on both sides, some renderers derive significant optimizations
by considering these surfaces to have only a single outward side,
typically determined by control-point winding order and/or
\*orientation.  By doing so they can perform "backface culling" to
avoid drawing the many polygons of most closed surfaces that face away
from the viewer.

However, it is often advantageous to model thin objects such as paper
and cloth as single, open surfaces that must be viewable from both
sides, always.  Setting a gprim's \*doubleSided attribute to
\\c true instructs all renderers to disable optimizations such as
backface culling for the gprim, and attempt (not all renderers are able
to do so, but the USD reference GL renderer always will) to provide
forward-facing normals on each side of the surface for lighting
calculations.*/
    pub double_sided: bool,
    /**Orientation specifies whether the gprim's surface normal
should be computed using the right hand rule, or the left hand rule.
Please see \UsdGeom_WindingOrder for a deeper explanation and
generalization of orientation to composed scenes with transformation
hierarchies.*/
    pub orientation: crate::foundation::TfToken,
    /**The length of the capsule's spine along the specified
\*axis excluding the size of the two half spheres, i.e.
the length of the cylinder portion of the capsule.
If you author \*height you must also author \*extent.
\See: GetExtentAttr()*/
    pub height: f64,
    /**The radius of the capping sphere at the top of the capsule -
i.e. the sphere in the direction of the positive \*axis. If you
author \*radius you must also author \*extent.

\See: GetExtentAttr()*/
    pub radius_top: f64,
    /**The radius of the capping sphere at the bottom of the capsule -
i.e. the sphere located in the direction of the negative \*axis. If
you author \*radius you must also author \*extent.

\See: GetExtentAttr()*/
    pub radius_bottom: f64,
    ///The axis along which the spine of the capsule is aligned
    pub axis: crate::foundation::TfToken,
}
impl Default for UsdGeomCapsule_1 {
    fn default() -> Self {
        Self {
            visibility: crate::foundation::TfToken::new("inherited"),
            purpose: crate::foundation::TfToken::new("default"),
            xform_op_order: Vec::new(),
            extent: Vec::new(),
            primvars_display_color: Vec::new(),
            primvars_display_opacity: Vec::new(),
            double_sided: false,
            orientation: crate::foundation::TfToken::new("rightHanded"),
            height: 1f64,
            radius_top: 0.5f64,
            radius_bottom: 0.5f64,
            axis: crate::foundation::TfToken::new("Z"),
        }
    }
}
impl crate::schema::generated::UsdSchemaInfo for UsdGeomCapsule_1 {
    fn schema_name(&self) -> &'static str {
        "UsdGeomCapsule_1"
    }
    fn attribute_metadata(
        &self,
    ) -> &'static [crate::schema::generated::AttributeMetadata] {
        static META: &[crate::schema::generated::AttributeMetadata] = &[
            crate::schema::generated::AttributeMetadata {
                usd_name: "visibility",
                usd_type: "token",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "purpose",
                usd_type: "token",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "xformOpOrder",
                usd_type: "token[]",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "extent",
                usd_type: "float3[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "primvars:displayColor",
                usd_type: "color3f[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "primvars:displayOpacity",
                usd_type: "float[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "doubleSided",
                usd_type: "bool",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "orientation",
                usd_type: "token",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "height",
                usd_type: "double",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "radiusTop",
                usd_type: "double",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "radiusBottom",
                usd_type: "double",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "axis",
                usd_type: "token",
                is_uniform: true,
            },
        ];
        META
    }
}
/**Defines a primitive plane, centered at the origin, and is defined by
a cardinal axis, width, and length. The plane is double-sided by default.

The axis of width and length are perpendicular to the plane's \*axis:

axis  | width  | length
----- | ------ | -------
X     | z-axis | y-axis
Y     | x-axis | z-axis
Z     | x-axis | y-axis*/
#[derive(Clone, Debug)]
pub struct UsdGeomPlane {
    /**Visibility is meant to be the simplest form of "pruning"
visibility that is supported by most DCC apps.  Visibility is
animatable, allowing a sub-tree of geometry to be present for some
segment of a shot, and absent from others; unlike the action of
deactivating geometry prims, invisible geometry is still
available for inspection, for positioning, for defining volumes, etc.*/
    pub visibility: crate::foundation::TfToken,
    /**Purpose is a classification of geometry into categories that
can each be independently included or excluded from traversals of prims
on a stage, such as rendering or bounding-box computation traversals.

See \UsdGeom_ImageablePurpose for more detail about how
\*purpose is computed and used.*/
    pub purpose: crate::foundation::TfToken,
    /**Encodes the sequence of transformation operations in the
order in which they should be pushed onto a transform stack while
visiting a UsdStage's prims in a graph traversal that will effect
the desired positioning for this prim and its descendant prims.

You should rarely, if ever, need to manipulate this attribute directly.
It is managed by the AddXformOp(), SetResetXformStack(), and
SetXformOpOrder(), and consulted by GetOrderedXformOps() and
GetLocalTransformation().*/
    pub xform_op_order: crate::foundation::VtArray<crate::foundation::TfToken>,
    /**Extent is a three dimensional range measuring the geometric
extent of the authored gprim in its own local space (i.e. its own
transform not applied), \*without accounting for any shader-induced
displacement. If __any__ extent value has been authored for a given
Boundable, then it should be authored at every timeSample at which
geometry-affecting properties are authored, to ensure correct
evaluation via ComputeExtent(). If __no__ extent value has been
authored, then ComputeExtent() will call the Boundable's registered
ComputeExtentFunction(), which may be expensive, which is why we
strongly encourage proper authoring of extent.
\See: ComputeExtent()
\See: \UsdGeom_Boundable_Extent.

An authored extent on a prim which has children is expected to include
the extent of all children, as they will be pruned from BBox computation
during traversal.*/
    pub extent: crate::foundation::VtArray<crate::foundation::GfVec3f>,
    /**It is useful to have an "official" colorSet that can be used
as a display or modeling color, even in the absence of any specified
shader for a gprim.  DisplayColor serves this role; because it is a
UsdGeomPrimvar, it can also be used as a gprim override for any shader
that consumes a \*displayColor parameter.*/
    pub primvars_display_color: crate::foundation::VtArray<crate::foundation::GfVec3f>,
    /**Companion to \*displayColor that specifies opacity, broken
out as an independent attribute rather than an rgba color, both so that
each can be independently overridden, and because shaders rarely consume
rgba parameters.*/
    pub primvars_display_opacity: crate::foundation::VtArray<f32>,
    /**Although some renderers treat all parametric or polygonal
surfaces as if they were effectively laminae with outward-facing
normals on both sides, some renderers derive significant optimizations
by considering these surfaces to have only a single outward side,
typically determined by control-point winding order and/or
\*orientation.  By doing so they can perform "backface culling" to
avoid drawing the many polygons of most closed surfaces that face away
from the viewer.

However, it is often advantageous to model thin objects such as paper
and cloth as single, open surfaces that must be viewable from both
sides, always.  Setting a gprim's \*doubleSided attribute to
\\c true instructs all renderers to disable optimizations such as
backface culling for the gprim, and attempt (not all renderers are able
to do so, but the USD reference GL renderer always will) to provide
forward-facing normals on each side of the surface for lighting
calculations.*/
    pub double_sided: bool,
    /**Orientation specifies whether the gprim's surface normal
should be computed using the right hand rule, or the left hand rule.
Please see \UsdGeom_WindingOrder for a deeper explanation and
generalization of orientation to composed scenes with transformation
hierarchies.*/
    pub orientation: crate::foundation::TfToken,
    /**The width of the plane, which aligns to the x-axis when \*axis is
'Z' or 'Y', or to the z-axis when \*axis is 'X'.  If you author \*width
you must also author \*extent.

\See: UsdGeomGprim::GetExtentAttr()*/
    pub width: f64,
    /**The length of the plane, which aligns to the y-axis when \*axis is
'Z' or 'X', or to the z-axis when \*axis is 'Y'.  If you author \*length
you must also author \*extent.

\See: UsdGeomGprim::GetExtentAttr()*/
    pub length: f64,
    /**The axis along which the surface of the plane is aligned. When set
to 'Z' the plane is in the xy-plane; when \*axis is 'X' the plane is in
the yz-plane, and when \*axis is 'Y' the plane is in the xz-plane.

\See: UsdGeomGprim::GetAxisAttr().*/
    pub axis: crate::foundation::TfToken,
}
impl Default for UsdGeomPlane {
    fn default() -> Self {
        Self {
            visibility: crate::foundation::TfToken::new("inherited"),
            purpose: crate::foundation::TfToken::new("default"),
            xform_op_order: Vec::new(),
            extent: Vec::new(),
            primvars_display_color: Vec::new(),
            primvars_display_opacity: Vec::new(),
            double_sided: false,
            orientation: crate::foundation::TfToken::new("rightHanded"),
            width: 2f64,
            length: 2f64,
            axis: crate::foundation::TfToken::new("Z"),
        }
    }
}
impl crate::schema::generated::UsdSchemaInfo for UsdGeomPlane {
    fn schema_name(&self) -> &'static str {
        "UsdGeomPlane"
    }
    fn attribute_metadata(
        &self,
    ) -> &'static [crate::schema::generated::AttributeMetadata] {
        static META: &[crate::schema::generated::AttributeMetadata] = &[
            crate::schema::generated::AttributeMetadata {
                usd_name: "visibility",
                usd_type: "token",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "purpose",
                usd_type: "token",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "xformOpOrder",
                usd_type: "token[]",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "extent",
                usd_type: "float3[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "primvars:displayColor",
                usd_type: "color3f[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "primvars:displayOpacity",
                usd_type: "float[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "doubleSided",
                usd_type: "bool",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "orientation",
                usd_type: "token",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "width",
                usd_type: "double",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "length",
                usd_type: "double",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "axis",
                usd_type: "token",
                is_uniform: true,
            },
        ];
        META
    }
}
/**Encodes a mesh with optional subdivision properties and features.

As a point-based primitive, meshes are defined in terms of points that
are connected into edges and faces. Many references to meshes use the
term 'vertex' in place of or interchangeably with 'points', while some
use 'vertex' to refer to the 'face-vertices' that define a face.  To
avoid confusion, the term 'vertex' is intentionally avoided in favor of
'points' or 'face-vertices'.

The connectivity between points, edges and faces is encoded using a
common minimal topological description of the faces of the mesh.  Each
face is defined by a set of face-vertices using indices into the Mesh's
_points_ array (inherited from UsdGeomPointBased) and laid out in a
single linear _faceVertexIndices_ array for efficiency.  A companion
_faceVertexCounts_ array provides, for each face, the number of
consecutive face-vertices in _faceVertexIndices_ that define the face.
No additional connectivity information is required or constructed, so
no adjacency or neighborhood queries are available.

A key property of this mesh schema is that it encodes both subdivision
surfaces and simpler polygonal meshes. This is achieved by varying the
_subdivisionScheme_ attribute, which is set to specify Catmull-Clark
subdivision by default, so polygonal meshes must always be explicitly
declared. The available subdivision schemes and additional subdivision
features encoded in optional attributes conform to the feature set of
OpenSubdiv
(https://graphics.pixar.com/opensubdiv/docs/subdivision_surfaces.html).

\UsdGeom_Mesh_Primvars
__A Note About Primvars__

The following list clarifies the number of elements for and the
interpolation behavior of the different primvar interpolation types
for meshes:

- __constant__: One element for the entire mesh; no interpolation.
- __uniform__: One element for each face of the mesh; elements are
typically not interpolated but are inherited by other faces derived
from a given face (via subdivision, tessellation, etc.).
- __varying__: One element for each point of the mesh;
interpolation of point data is always linear.
- __vertex__: One element for each point of the mesh;
interpolation of point data is applied according to the
_subdivisionScheme_ attribute.
- __faceVarying__: One element for each of the face-vertices that
define the mesh topology; interpolation of face-vertex data may
be smooth or linear, according to the _subdivisionScheme_ and
_faceVaryingLinearInterpolation_ attributes.

Primvar interpolation types and related utilities are described more
generally in \Usd_InterpolationVals.

\UsdGeom_Mesh_Normals
__A Note About Normals__

Normals should not be authored on a subdivision mesh, since subdivision
algorithms define their own normals. They should only be authored for
polygonal meshes (_subdivisionScheme_ = "none").

The _normals_ attribute inherited from UsdGeomPointBased is not a generic
primvar, but the number of elements in this attribute will be determined by
its _interpolation_.  See \UsdGeomPointBased::GetNormalsInterpolation() .
If _normals_ and _primvars:normals_ are both specified, the latter has
precedence.  If a polygonal mesh specifies __neither__ _normals_ nor
_primvars:normals_, then it should be treated and rendered as faceted,
with no attempt to compute smooth normals.

The normals generated for smooth subdivision schemes, e.g. Catmull-Clark
and Loop, will likewise be smooth, but others, e.g. Bilinear, may be
discontinuous between faces and/or within non-planar irregular faces.*/
#[derive(Clone, Debug)]
pub struct UsdGeomMesh {
    /**Visibility is meant to be the simplest form of "pruning"
visibility that is supported by most DCC apps.  Visibility is
animatable, allowing a sub-tree of geometry to be present for some
segment of a shot, and absent from others; unlike the action of
deactivating geometry prims, invisible geometry is still
available for inspection, for positioning, for defining volumes, etc.*/
    pub visibility: crate::foundation::TfToken,
    /**Purpose is a classification of geometry into categories that
can each be independently included or excluded from traversals of prims
on a stage, such as rendering or bounding-box computation traversals.

See \UsdGeom_ImageablePurpose for more detail about how
\*purpose is computed and used.*/
    pub purpose: crate::foundation::TfToken,
    /**Encodes the sequence of transformation operations in the
order in which they should be pushed onto a transform stack while
visiting a UsdStage's prims in a graph traversal that will effect
the desired positioning for this prim and its descendant prims.

You should rarely, if ever, need to manipulate this attribute directly.
It is managed by the AddXformOp(), SetResetXformStack(), and
SetXformOpOrder(), and consulted by GetOrderedXformOps() and
GetLocalTransformation().*/
    pub xform_op_order: crate::foundation::VtArray<crate::foundation::TfToken>,
    /**Extent is a three dimensional range measuring the geometric
extent of the authored gprim in its own local space (i.e. its own
transform not applied), \*without accounting for any shader-induced
displacement. If __any__ extent value has been authored for a given
Boundable, then it should be authored at every timeSample at which
geometry-affecting properties are authored, to ensure correct
evaluation via ComputeExtent(). If __no__ extent value has been
authored, then ComputeExtent() will call the Boundable's registered
ComputeExtentFunction(), which may be expensive, which is why we
strongly encourage proper authoring of extent.
\See: ComputeExtent()
\See: \UsdGeom_Boundable_Extent.

An authored extent on a prim which has children is expected to include
the extent of all children, as they will be pruned from BBox computation
during traversal.*/
    pub extent: crate::foundation::VtArray<crate::foundation::GfVec3f>,
    /**It is useful to have an "official" colorSet that can be used
as a display or modeling color, even in the absence of any specified
shader for a gprim.  DisplayColor serves this role; because it is a
UsdGeomPrimvar, it can also be used as a gprim override for any shader
that consumes a \*displayColor parameter.*/
    pub primvars_display_color: crate::foundation::VtArray<crate::foundation::GfVec3f>,
    /**Companion to \*displayColor that specifies opacity, broken
out as an independent attribute rather than an rgba color, both so that
each can be independently overridden, and because shaders rarely consume
rgba parameters.*/
    pub primvars_display_opacity: crate::foundation::VtArray<f32>,
    /**Although some renderers treat all parametric or polygonal
surfaces as if they were effectively laminae with outward-facing
normals on both sides, some renderers derive significant optimizations
by considering these surfaces to have only a single outward side,
typically determined by control-point winding order and/or
\*orientation.  By doing so they can perform "backface culling" to
avoid drawing the many polygons of most closed surfaces that face away
from the viewer.

However, it is often advantageous to model thin objects such as paper
and cloth as single, open surfaces that must be viewable from both
sides, always.  Setting a gprim's \*doubleSided attribute to
\\c true instructs all renderers to disable optimizations such as
backface culling for the gprim, and attempt (not all renderers are able
to do so, but the USD reference GL renderer always will) to provide
forward-facing normals on each side of the surface for lighting
calculations.*/
    pub double_sided: bool,
    /**Orientation specifies whether the gprim's surface normal
should be computed using the right hand rule, or the left hand rule.
Please see \UsdGeom_WindingOrder for a deeper explanation and
generalization of orientation to composed scenes with transformation
hierarchies.*/
    pub orientation: crate::foundation::TfToken,
    /**The primary geometry attribute for all PointBased
primitives, describes points in (local) space.*/
    pub points: crate::foundation::VtArray<crate::foundation::GfVec3f>,
    /**If provided, 'velocities' should be used by renderers to

compute positions between samples for the 'points' attribute, rather
than interpolating between neighboring 'points' samples.  This is the
only reasonable means of computing motion blur for topologically
varying PointBased primitives.  It follows that the length of each
'velocities' sample must match the length of the corresponding
'points' sample.  Velocity is measured in position units per second,
as per most simulation software. To convert to position units per
UsdTimeCode, divide by UsdStage::GetTimeCodesPerSecond().

See also \UsdGeom_VelocityInterpolation .*/
    pub velocities: crate::foundation::VtArray<crate::foundation::GfVec3f>,
    /**If provided, 'accelerations' should be used with
velocities to compute positions between samples for the 'points'
attribute rather than interpolating between neighboring 'points'
samples. Acceleration is measured in position units per second-squared.
To convert to position units per squared UsdTimeCode, divide by the
square of UsdStage::GetTimeCodesPerSecond().*/
    pub accelerations: crate::foundation::VtArray<crate::foundation::GfVec3f>,
    /**Provide an object-space orientation for individual points,
which, depending on subclass, may define a surface, curve, or free
points.  Note that 'normals' should not be authored on any Mesh that
is subdivided, since the subdivision algorithm will define its own
normals. 'normals' is not a generic primvar, but the number of elements
in this attribute will be determined by its 'interpolation'.  See
\SetNormalsInterpolation() . If 'normals' and 'primvars:normals'
are both specified, the latter has precedence.*/
    pub normals: crate::foundation::VtArray<crate::foundation::GfVec3f>,
    /**Flat list of the index (into the _points_ attribute) of each
vertex of each face in the mesh.  If this attribute has more than
one timeSample, the mesh is considered to be topologically varying.*/
    pub face_vertex_indices: crate::foundation::VtArray<i32>,
    /**Provides the number of vertices in each face of the mesh,
which is also the number of consecutive indices in _faceVertexIndices_
that define the face.  The length of this attribute is the number of
faces in the mesh.  If this attribute has more than
one timeSample, the mesh is considered to be topologically varying.*/
    pub face_vertex_counts: crate::foundation::VtArray<i32>,
    /**The subdivision scheme to be applied to the surface.
Valid values are:

- __catmullClark__: The default, Catmull-Clark subdivision; preferred
for quad-dominant meshes (generalizes B-splines); interpolation
of point data is smooth (non-linear)
- __loop__: Loop subdivision; preferred for purely triangular meshes;
interpolation of point data is smooth (non-linear)
- __bilinear__: Subdivision reduces all faces to quads (topologically
similar to "catmullClark"); interpolation of point data is bilinear
- __none__: No subdivision, i.e. a simple polygonal mesh; interpolation
of point data is linear

Polygonal meshes are typically lighter weight and faster to render,
depending on renderer and render mode.  Use of "bilinear" will produce
a similar shape to a polygonal mesh and may offer additional guarantees
of watertightness and additional subdivision features (e.g. holes) but
may also not respect authored normals.*/
    pub subdivision_scheme: crate::foundation::TfToken,
    /**Specifies how subdivision is applied for faces adjacent to
boundary edges and boundary points. Valid values correspond to choices
available in OpenSubdiv:

- __none__: No boundary interpolation is applied and boundary faces are
effectively treated as holes
- __edgeOnly__: A sequence of boundary edges defines a smooth curve to
which the edges of subdivided boundary faces converge
- __edgeAndCorner__: The default, similar to "edgeOnly" but the smooth
boundary curve is made sharp at corner points

These are illustrated and described in more detail in the OpenSubdiv
documentation:
https://graphics.pixar.com/opensubdiv/docs/subdivision_surfaces.html#boundary-interpolation-rules*/
    pub interpolate_boundary: crate::foundation::TfToken,
    /**Specifies how elements of a primvar of interpolation type
"faceVarying" are interpolated for subdivision surfaces. Interpolation
can be as smooth as a "vertex" primvar or constrained to be linear at
features specified by several options.  Valid values correspond to
choices available in OpenSubdiv:

- __none__: No linear constraints or sharpening, smooth everywhere
- __cornersOnly__: Sharpen corners of discontinuous boundaries only,
smooth everywhere else
- __cornersPlus1__: The default, same as "cornersOnly" plus additional
sharpening at points where three or more distinct face-varying
values occur
- __cornersPlus2__: Same as "cornersPlus1" plus additional sharpening
at points with at least one discontinuous boundary corner or
only one discontinuous boundary edge (a dart)
- __boundaries__: Piecewise linear along discontinuous boundaries,
smooth interior
- __all__: Piecewise linear everywhere

These are illustrated and described in more detail in the OpenSubdiv
documentation:
https://graphics.pixar.com/opensubdiv/docs/subdivision_surfaces.html#face-varying-interpolation-rules*/
    pub face_varying_linear_interpolation: crate::foundation::TfToken,
    /**Specifies an option to the subdivision rules for the
Catmull-Clark scheme to try and improve undesirable artifacts when
subdividing triangles.  Valid values are "catmullClark" for the
standard rules (the default) and "smooth" for the improvement.

See https://graphics.pixar.com/opensubdiv/docs/subdivision_surfaces.html#triangle-subdivision-rule*/
    pub triangle_subdivision_rule: crate::foundation::TfToken,
    /**The indices of all faces that should be treated as holes,
i.e. made invisible. This is traditionally a feature of subdivision
surfaces and not generally applied to polygonal meshes.*/
    pub hole_indices: crate::foundation::VtArray<i32>,
    /**The indices of points for which a corresponding sharpness
value is specified in _cornerSharpnesses_ (so the size of this array
must match that of _cornerSharpnesses_).*/
    pub corner_indices: crate::foundation::VtArray<i32>,
    /**The sharpness values associated with a corresponding set of
points specified in _cornerIndices_ (so the size of this array must
match that of _cornerIndices_). Use the constant `SHARPNESS_INFINITE`
for a perfectly sharp corner.*/
    pub corner_sharpnesses: crate::foundation::VtArray<f32>,
    /**The indices of points grouped into sets of successive pairs
that identify edges to be creased. The size of this array must be
equal to the sum of all elements of the _creaseLengths_ attribute.*/
    pub crease_indices: crate::foundation::VtArray<i32>,
    /**The length of this array specifies the number of creases
(sets of adjacent sharpened edges) on the mesh. Each element gives
the number of points of each crease, whose indices are successively
laid out in the _creaseIndices_ attribute. Since each crease must
be at least one edge long, each element of this array must be at
least two.*/
    pub crease_lengths: crate::foundation::VtArray<i32>,
    /**The per-crease or per-edge sharpness values for all creases.
Since _creaseLengths_ encodes the number of points in each crease,
the number of elements in this array will be either len(creaseLengths)
or the sum over all X of (creaseLengths[X] - 1). Note that while
the RI spec allows each crease to have either a single sharpness
or a value per-edge, USD will encode either a single sharpness
per crease on a mesh, or sharpnesses for all edges making up
the creases on a mesh.  Use the constant `SHARPNESS_INFINITE` for a
perfectly sharp crease.*/
    pub crease_sharpnesses: crate::foundation::VtArray<f32>,
}
impl Default for UsdGeomMesh {
    fn default() -> Self {
        Self {
            visibility: crate::foundation::TfToken::new("inherited"),
            purpose: crate::foundation::TfToken::new("default"),
            xform_op_order: Vec::new(),
            extent: Vec::new(),
            primvars_display_color: Vec::new(),
            primvars_display_opacity: Vec::new(),
            double_sided: false,
            orientation: crate::foundation::TfToken::new("rightHanded"),
            points: Vec::new(),
            velocities: Vec::new(),
            accelerations: Vec::new(),
            normals: Vec::new(),
            face_vertex_indices: Vec::new(),
            face_vertex_counts: Vec::new(),
            subdivision_scheme: crate::foundation::TfToken::new("catmullClark"),
            interpolate_boundary: crate::foundation::TfToken::new("edgeAndCorner"),
            face_varying_linear_interpolation: crate::foundation::TfToken::new(
                "cornersPlus1",
            ),
            triangle_subdivision_rule: crate::foundation::TfToken::new("catmullClark"),
            hole_indices: Vec::new(),
            corner_indices: Vec::new(),
            corner_sharpnesses: Vec::new(),
            crease_indices: Vec::new(),
            crease_lengths: Vec::new(),
            crease_sharpnesses: Vec::new(),
        }
    }
}
impl crate::schema::generated::UsdSchemaInfo for UsdGeomMesh {
    fn schema_name(&self) -> &'static str {
        "UsdGeomMesh"
    }
    fn attribute_metadata(
        &self,
    ) -> &'static [crate::schema::generated::AttributeMetadata] {
        static META: &[crate::schema::generated::AttributeMetadata] = &[
            crate::schema::generated::AttributeMetadata {
                usd_name: "visibility",
                usd_type: "token",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "purpose",
                usd_type: "token",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "xformOpOrder",
                usd_type: "token[]",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "extent",
                usd_type: "float3[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "primvars:displayColor",
                usd_type: "color3f[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "primvars:displayOpacity",
                usd_type: "float[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "doubleSided",
                usd_type: "bool",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "orientation",
                usd_type: "token",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "points",
                usd_type: "point3f[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "velocities",
                usd_type: "vector3f[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "accelerations",
                usd_type: "vector3f[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "normals",
                usd_type: "normal3f[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "faceVertexIndices",
                usd_type: "int[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "faceVertexCounts",
                usd_type: "int[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "subdivisionScheme",
                usd_type: "token",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "interpolateBoundary",
                usd_type: "token",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "faceVaryingLinearInterpolation",
                usd_type: "token",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "triangleSubdivisionRule",
                usd_type: "token",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "holeIndices",
                usd_type: "int[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "cornerIndices",
                usd_type: "int[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "cornerSharpnesses",
                usd_type: "float[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "creaseIndices",
                usd_type: "int[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "creaseLengths",
                usd_type: "int[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "creaseSharpnesses",
                usd_type: "float[]",
                is_uniform: false,
            },
        ];
        META
    }
}
/**Encodes a tetrahedral mesh. A tetrahedral mesh is defined as a set of
tetrahedra. Each tetrahedron is defined by a set of 4 points, with the
triangles of the tetrahedron determined from these 4 points as described in
the <b>tetVertexIndices</b> attribute description. The mesh surface faces
are encoded as triangles. Surface faces must be provided for consumers
that need to do surface calculations, such as renderers or consumers using
physics attachments. Both tetrahedra and surface face definitions use
indices into the TetMesh's <b>points</b> attribute, inherited from
UsdGeomPointBased.*/
#[derive(Clone, Debug)]
pub struct UsdGeomTetMesh {
    /**Visibility is meant to be the simplest form of "pruning"
visibility that is supported by most DCC apps.  Visibility is
animatable, allowing a sub-tree of geometry to be present for some
segment of a shot, and absent from others; unlike the action of
deactivating geometry prims, invisible geometry is still
available for inspection, for positioning, for defining volumes, etc.*/
    pub visibility: crate::foundation::TfToken,
    /**Purpose is a classification of geometry into categories that
can each be independently included or excluded from traversals of prims
on a stage, such as rendering or bounding-box computation traversals.

See \UsdGeom_ImageablePurpose for more detail about how
\*purpose is computed and used.*/
    pub purpose: crate::foundation::TfToken,
    /**Encodes the sequence of transformation operations in the
order in which they should be pushed onto a transform stack while
visiting a UsdStage's prims in a graph traversal that will effect
the desired positioning for this prim and its descendant prims.

You should rarely, if ever, need to manipulate this attribute directly.
It is managed by the AddXformOp(), SetResetXformStack(), and
SetXformOpOrder(), and consulted by GetOrderedXformOps() and
GetLocalTransformation().*/
    pub xform_op_order: crate::foundation::VtArray<crate::foundation::TfToken>,
    /**Extent is a three dimensional range measuring the geometric
extent of the authored gprim in its own local space (i.e. its own
transform not applied), \*without accounting for any shader-induced
displacement. If __any__ extent value has been authored for a given
Boundable, then it should be authored at every timeSample at which
geometry-affecting properties are authored, to ensure correct
evaluation via ComputeExtent(). If __no__ extent value has been
authored, then ComputeExtent() will call the Boundable's registered
ComputeExtentFunction(), which may be expensive, which is why we
strongly encourage proper authoring of extent.
\See: ComputeExtent()
\See: \UsdGeom_Boundable_Extent.

An authored extent on a prim which has children is expected to include
the extent of all children, as they will be pruned from BBox computation
during traversal.*/
    pub extent: crate::foundation::VtArray<crate::foundation::GfVec3f>,
    /**It is useful to have an "official" colorSet that can be used
as a display or modeling color, even in the absence of any specified
shader for a gprim.  DisplayColor serves this role; because it is a
UsdGeomPrimvar, it can also be used as a gprim override for any shader
that consumes a \*displayColor parameter.*/
    pub primvars_display_color: crate::foundation::VtArray<crate::foundation::GfVec3f>,
    /**Companion to \*displayColor that specifies opacity, broken
out as an independent attribute rather than an rgba color, both so that
each can be independently overridden, and because shaders rarely consume
rgba parameters.*/
    pub primvars_display_opacity: crate::foundation::VtArray<f32>,
    /**Although some renderers treat all parametric or polygonal
surfaces as if they were effectively laminae with outward-facing
normals on both sides, some renderers derive significant optimizations
by considering these surfaces to have only a single outward side,
typically determined by control-point winding order and/or
\*orientation.  By doing so they can perform "backface culling" to
avoid drawing the many polygons of most closed surfaces that face away
from the viewer.

However, it is often advantageous to model thin objects such as paper
and cloth as single, open surfaces that must be viewable from both
sides, always.  Setting a gprim's \*doubleSided attribute to
\\c true instructs all renderers to disable optimizations such as
backface culling for the gprim, and attempt (not all renderers are able
to do so, but the USD reference GL renderer always will) to provide
forward-facing normals on each side of the surface for lighting
calculations.*/
    pub double_sided: bool,
    /**Orientation specifies whether the gprim's surface normal
should be computed using the right hand rule, or the left hand rule.
Please see \UsdGeom_WindingOrder for a deeper explanation and
generalization of orientation to composed scenes with transformation
hierarchies.*/
    pub orientation: crate::foundation::TfToken,
    /**The primary geometry attribute for all PointBased
primitives, describes points in (local) space.*/
    pub points: crate::foundation::VtArray<crate::foundation::GfVec3f>,
    /**If provided, 'velocities' should be used by renderers to

compute positions between samples for the 'points' attribute, rather
than interpolating between neighboring 'points' samples.  This is the
only reasonable means of computing motion blur for topologically
varying PointBased primitives.  It follows that the length of each
'velocities' sample must match the length of the corresponding
'points' sample.  Velocity is measured in position units per second,
as per most simulation software. To convert to position units per
UsdTimeCode, divide by UsdStage::GetTimeCodesPerSecond().

See also \UsdGeom_VelocityInterpolation .*/
    pub velocities: crate::foundation::VtArray<crate::foundation::GfVec3f>,
    /**If provided, 'accelerations' should be used with
velocities to compute positions between samples for the 'points'
attribute rather than interpolating between neighboring 'points'
samples. Acceleration is measured in position units per second-squared.
To convert to position units per squared UsdTimeCode, divide by the
square of UsdStage::GetTimeCodesPerSecond().*/
    pub accelerations: crate::foundation::VtArray<crate::foundation::GfVec3f>,
    /**Provide an object-space orientation for individual points,
which, depending on subclass, may define a surface, curve, or free
points.  Note that 'normals' should not be authored on any Mesh that
is subdivided, since the subdivision algorithm will define its own
normals. 'normals' is not a generic primvar, but the number of elements
in this attribute will be determined by its 'interpolation'.  See
\SetNormalsInterpolation() . If 'normals' and 'primvars:normals'
are both specified, the latter has precedence.*/
    pub normals: crate::foundation::VtArray<crate::foundation::GfVec3f>,
    /**Flat list of the index (into the <b>points</b> attribute) of
each vertex of each tetrahedron in the mesh. Each int4 corresponds to the
indices of a single tetrahedron. Users should set the <b>orientation</b>
attribute of UsdGeomPrim accordingly. That is if the <b>orientation</b>
is "rightHanded", the CCW face ordering of a tetrahedron is
[123],[032],[013],[021] with respect to the int4. This results in the
normals facing outward from the center of the tetrahedron. The following
diagram shows the face ordering of an unwrapped tetrahedron with
"rightHanded" orientation.

\[image] USDTetMeshRightHanded.svg

If the <b>orientation</b> attribute is set to "leftHanded" the face
ordering of the tetrahedron is [321],[230],[310],[120] and the
leftHanded CW face normals point outward from the center of the
tetrahedron. The following diagram shows the face ordering of an
unwrapped tetrahedron with "leftHanded" orientation.

\[image] USDTetMeshLeftHanded.svg

Setting the <b>orientation</b> attribute to align with the
ordering of the int4 for the tetrahedrons is the responsibility of the
user.*/
    pub tet_vertex_indices: crate::foundation::VtArray<[i32; 4]>,
    /**<b>surfaceFaceVertexIndices</b> defines the triangle
surface faces indices wrt. <b>points</b> of the tetmesh surface. Again
the <b>orientation</b> attribute inherited from UsdGeomPrim should be
set accordingly. The <b>orientation</b> for faces of tetrahedra and
surface faces must match.*/
    pub surface_face_vertex_indices: crate::foundation::VtArray<[i32; 3]>,
}
impl Default for UsdGeomTetMesh {
    fn default() -> Self {
        Self {
            visibility: crate::foundation::TfToken::new("inherited"),
            purpose: crate::foundation::TfToken::new("default"),
            xform_op_order: Vec::new(),
            extent: Vec::new(),
            primvars_display_color: Vec::new(),
            primvars_display_opacity: Vec::new(),
            double_sided: false,
            orientation: crate::foundation::TfToken::new("rightHanded"),
            points: Vec::new(),
            velocities: Vec::new(),
            accelerations: Vec::new(),
            normals: Vec::new(),
            tet_vertex_indices: Vec::new(),
            surface_face_vertex_indices: Vec::new(),
        }
    }
}
impl crate::schema::generated::UsdSchemaInfo for UsdGeomTetMesh {
    fn schema_name(&self) -> &'static str {
        "UsdGeomTetMesh"
    }
    fn attribute_metadata(
        &self,
    ) -> &'static [crate::schema::generated::AttributeMetadata] {
        static META: &[crate::schema::generated::AttributeMetadata] = &[
            crate::schema::generated::AttributeMetadata {
                usd_name: "visibility",
                usd_type: "token",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "purpose",
                usd_type: "token",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "xformOpOrder",
                usd_type: "token[]",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "extent",
                usd_type: "float3[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "primvars:displayColor",
                usd_type: "color3f[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "primvars:displayOpacity",
                usd_type: "float[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "doubleSided",
                usd_type: "bool",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "orientation",
                usd_type: "token",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "points",
                usd_type: "point3f[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "velocities",
                usd_type: "vector3f[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "accelerations",
                usd_type: "vector3f[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "normals",
                usd_type: "normal3f[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "tetVertexIndices",
                usd_type: "int4[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "surfaceFaceVertexIndices",
                usd_type: "int3[]",
                is_uniform: false,
            },
        ];
        META
    }
}
/**Encodes a subset of a piece of geometry (i.e. a UsdGeomImageable)
as a set of indices. Currently supports encoding subsets of faces,
points, edges, segments, and tetrahedrons.

To apply to a geometric prim, a GeomSubset prim must be the prim's direct
child in namespace, and possess a concrete defining specifier (i.e. def).
This restriction makes it easy and efficient to discover subsets of a prim.
We might want to relax this restriction if it's common to have multiple
<b>families</b> of subsets on a gprim and if it's useful to be able to
organize subsets belonging to a <b>family</b> under a common scope. See
'familyName' attribute for more info on defining a family of subsets.

Note that a GeomSubset isn't an imageable (i.e. doesn't derive from
UsdGeomImageable). So, you can't author <b>visibility</b> for it or
override its <b>purpose</b>.

Materials are bound to GeomSubsets just as they are for regular
geometry using API available in UsdShade (UsdShadeMaterial::Bind).*/
#[derive(Clone, Debug)]
pub struct UsdGeomGeomSubset {
    /**The type of element that the indices target. "elementType" can
have one of the following values:
<ul><li><b>face</b>: Identifies faces on a Gprim's surface. For a
UsdGeomMesh, each element of the _indices_ attribute would refer to
an element of the Mesh's _faceCounts_ attribute. For a UsdGeomTetMesh,
each element of the _indices_ attribute would refer to an element of
the Mesh's _surfaceFaceVertexIndices_ attribute.</li>
<li><b>point</b>: for any UsdGeomPointBased, each
element of the _indices_ attribute would refer to an element of the
Mesh's _points_ attribute</li>
<li><b>edge</b>: for any UsdGeomMesh, each pair of elements
in the _indices_ attribute would refer to a pair of points of the
Mesh's _points_ attribute that are connected as an implicit edge on the
Mesh. These edges are derived from the Mesh's _faceVertexIndices_
attribute. Edges are not currently defined for a UsdGeomTetMesh, but
could be derived from all tetrahedron edges or surface face edges only
if a specific use-case arises.</li>
<li><b>segment</b>: for any Curve, each pair of elements
in the _indices_ attribute would refer to a pair of indices
(_curveIndex_, _segmentIndex_) where _curveIndex_ is the position of
the specified curve in the Curve's _curveVertexCounts_ attribute, and
_segmentIndex_ is the index of the segment within that curve.</li>
<li><b>tetrahedron</b>: for any UsdGeomTetMesh, each element of the
_indices_ attribute would refer to an element of the TetMesh's
_tetVertexIndices_ attribute.
</li></ul>*/
    pub element_type: crate::foundation::TfToken,
    /**The set of indices included in this subset. The indices need not
be sorted, but the same index should not appear more than once. Indices
are invalid if outside the range [0, elementCount) for the given time on
the parent geometric prim.*/
    pub indices: crate::foundation::VtArray<i32>,
    /**The name of the family of subsets that this subset belongs to.
This is optional and is primarily useful when there are multiple
families of subsets under a geometric prim. In some cases, this could
also be used for achieving proper roundtripping of subset data between
DCC apps.
When multiple subsets belonging to a prim have the same familyName, they
are said to belong to the family. A <i>familyType</i> value can be
encoded on the owner of a family of subsets as a token using the static
method UsdGeomSubset::SetFamilyType(). "familyType" can have one of the
following values:
<ul><li><b>UsdGeomTokens->partition</b>: implies that every element of
the whole geometry appears exactly once in only one of the subsets
belonging to the family.</li>
<li><b>UsdGeomTokens->nonOverlapping</b>: an element that appears in one
subset may not appear in any other subset belonging to the family, and
appears only once in the subset in which it appears.</li>
<li><b>UsdGeomTokens->unrestricted</b>: implies that there are no
restrictions w.r.t. the membership of elements in the subsets. They
could be overlapping and the union of all subsets in the family may
not represent the whole.</li>
</ul>
\Note: The validity of subset data is not enforced by the authoring
APIs, however they can be checked using UsdGeomSubset::ValidateFamily().*/
    pub family_name: crate::foundation::TfToken,
}
impl Default for UsdGeomGeomSubset {
    fn default() -> Self {
        Self {
            element_type: crate::foundation::TfToken::new("face"),
            indices: Vec::new(),
            family_name: crate::foundation::TfToken::new(""),
        }
    }
}
impl crate::schema::generated::UsdSchemaInfo for UsdGeomGeomSubset {
    fn schema_name(&self) -> &'static str {
        "UsdGeomGeomSubset"
    }
    fn attribute_metadata(
        &self,
    ) -> &'static [crate::schema::generated::AttributeMetadata] {
        static META: &[crate::schema::generated::AttributeMetadata] = &[
            crate::schema::generated::AttributeMetadata {
                usd_name: "elementType",
                usd_type: "token",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "indices",
                usd_type: "int[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "familyName",
                usd_type: "token",
                is_uniform: true,
            },
        ];
        META
    }
}
/**Encodes a rational or polynomial non-uniform B-spline
surface, with optional trim curves.

The encoding mostly follows that of RiNuPatch and RiTrimCurve:
https://renderman.pixar.com/resources/RenderMan_20/geometricPrimitives.html#rinupatch , with some minor renaming and coalescing for clarity.

The layout of control vertices in the \*points attribute inherited
from UsdGeomPointBased is row-major with U considered rows, and V columns.

\UsdGeom_NurbsPatch_Form
<b>NurbsPatch Form</b>

The authored points, orders, knots, weights, and ranges are all that is
required to render the nurbs patch.  However, the only way to model closed
surfaces with nurbs is to ensure that the first and last control points
along the given axis are coincident.  Similarly, to ensure the surface is
not only closed but also C2 continuous, the last \*order - 1 control
points must be (correspondingly) coincident with the first \*order - 1
control points, and also the spacing of the last corresponding knots
must be the same as the first corresponding knots.

<b>Form</b> is provided as an aid to interchange between modeling and
animation applications so that they can robustly identify the intent with
which the surface was modelled, and take measures (if they are able) to
preserve the continuity/concidence constraints as the surface may be rigged
or deformed.
\- An \*open-form NurbsPatch has no continuity constraints.
\- A \*closed-form NurbsPatch expects the first and last control points
to overlap
\- A \*periodic-form NurbsPatch expects the first and last
\*order - 1 control points to overlap.

<b>Nurbs vs Subdivision Surfaces</b>

Nurbs are an important modeling primitive in CAD/CAM tools and early
computer graphics DCC's.  Because they have a natural UV parameterization
they easily support "trim curves", which allow smooth shapes to be
carved out of the surface.

However, the topology of the patch is always rectangular, and joining two
nurbs patches together (especially when they have differing numbers of
spans) is difficult to do smoothly.  Also, nurbs are not supported by
the Ptex texturing technology (http://ptex.us).

Neither of these limitations are shared by subdivision surfaces; therefore,
although they do not subscribe to trim-curve-based shaping, subdivs are
often considered a more flexible modeling primitive.*/
#[derive(Clone, Debug)]
pub struct UsdGeomNurbsPatch {
    /**Visibility is meant to be the simplest form of "pruning"
visibility that is supported by most DCC apps.  Visibility is
animatable, allowing a sub-tree of geometry to be present for some
segment of a shot, and absent from others; unlike the action of
deactivating geometry prims, invisible geometry is still
available for inspection, for positioning, for defining volumes, etc.*/
    pub visibility: crate::foundation::TfToken,
    /**Purpose is a classification of geometry into categories that
can each be independently included or excluded from traversals of prims
on a stage, such as rendering or bounding-box computation traversals.

See \UsdGeom_ImageablePurpose for more detail about how
\*purpose is computed and used.*/
    pub purpose: crate::foundation::TfToken,
    /**Encodes the sequence of transformation operations in the
order in which they should be pushed onto a transform stack while
visiting a UsdStage's prims in a graph traversal that will effect
the desired positioning for this prim and its descendant prims.

You should rarely, if ever, need to manipulate this attribute directly.
It is managed by the AddXformOp(), SetResetXformStack(), and
SetXformOpOrder(), and consulted by GetOrderedXformOps() and
GetLocalTransformation().*/
    pub xform_op_order: crate::foundation::VtArray<crate::foundation::TfToken>,
    /**Extent is a three dimensional range measuring the geometric
extent of the authored gprim in its own local space (i.e. its own
transform not applied), \*without accounting for any shader-induced
displacement. If __any__ extent value has been authored for a given
Boundable, then it should be authored at every timeSample at which
geometry-affecting properties are authored, to ensure correct
evaluation via ComputeExtent(). If __no__ extent value has been
authored, then ComputeExtent() will call the Boundable's registered
ComputeExtentFunction(), which may be expensive, which is why we
strongly encourage proper authoring of extent.
\See: ComputeExtent()
\See: \UsdGeom_Boundable_Extent.

An authored extent on a prim which has children is expected to include
the extent of all children, as they will be pruned from BBox computation
during traversal.*/
    pub extent: crate::foundation::VtArray<crate::foundation::GfVec3f>,
    /**It is useful to have an "official" colorSet that can be used
as a display or modeling color, even in the absence of any specified
shader for a gprim.  DisplayColor serves this role; because it is a
UsdGeomPrimvar, it can also be used as a gprim override for any shader
that consumes a \*displayColor parameter.*/
    pub primvars_display_color: crate::foundation::VtArray<crate::foundation::GfVec3f>,
    /**Companion to \*displayColor that specifies opacity, broken
out as an independent attribute rather than an rgba color, both so that
each can be independently overridden, and because shaders rarely consume
rgba parameters.*/
    pub primvars_display_opacity: crate::foundation::VtArray<f32>,
    /**Although some renderers treat all parametric or polygonal
surfaces as if they were effectively laminae with outward-facing
normals on both sides, some renderers derive significant optimizations
by considering these surfaces to have only a single outward side,
typically determined by control-point winding order and/or
\*orientation.  By doing so they can perform "backface culling" to
avoid drawing the many polygons of most closed surfaces that face away
from the viewer.

However, it is often advantageous to model thin objects such as paper
and cloth as single, open surfaces that must be viewable from both
sides, always.  Setting a gprim's \*doubleSided attribute to
\\c true instructs all renderers to disable optimizations such as
backface culling for the gprim, and attempt (not all renderers are able
to do so, but the USD reference GL renderer always will) to provide
forward-facing normals on each side of the surface for lighting
calculations.*/
    pub double_sided: bool,
    /**Orientation specifies whether the gprim's surface normal
should be computed using the right hand rule, or the left hand rule.
Please see \UsdGeom_WindingOrder for a deeper explanation and
generalization of orientation to composed scenes with transformation
hierarchies.*/
    pub orientation: crate::foundation::TfToken,
    /**The primary geometry attribute for all PointBased
primitives, describes points in (local) space.*/
    pub points: crate::foundation::VtArray<crate::foundation::GfVec3f>,
    /**If provided, 'velocities' should be used by renderers to

compute positions between samples for the 'points' attribute, rather
than interpolating between neighboring 'points' samples.  This is the
only reasonable means of computing motion blur for topologically
varying PointBased primitives.  It follows that the length of each
'velocities' sample must match the length of the corresponding
'points' sample.  Velocity is measured in position units per second,
as per most simulation software. To convert to position units per
UsdTimeCode, divide by UsdStage::GetTimeCodesPerSecond().

See also \UsdGeom_VelocityInterpolation .*/
    pub velocities: crate::foundation::VtArray<crate::foundation::GfVec3f>,
    /**If provided, 'accelerations' should be used with
velocities to compute positions between samples for the 'points'
attribute rather than interpolating between neighboring 'points'
samples. Acceleration is measured in position units per second-squared.
To convert to position units per squared UsdTimeCode, divide by the
square of UsdStage::GetTimeCodesPerSecond().*/
    pub accelerations: crate::foundation::VtArray<crate::foundation::GfVec3f>,
    /**Provide an object-space orientation for individual points,
which, depending on subclass, may define a surface, curve, or free
points.  Note that 'normals' should not be authored on any Mesh that
is subdivided, since the subdivision algorithm will define its own
normals. 'normals' is not a generic primvar, but the number of elements
in this attribute will be determined by its 'interpolation'.  See
\SetNormalsInterpolation() . If 'normals' and 'primvars:normals'
are both specified, the latter has precedence.*/
    pub normals: crate::foundation::VtArray<crate::foundation::GfVec3f>,
    /**Number of vertices in the U direction.  Should be at least as
large as uOrder.*/
    pub u_vertex_count: Option<i32>,
    /**Number of vertices in the V direction.  Should be at least as
large as vOrder.*/
    pub v_vertex_count: Option<i32>,
    /**Order in the U direction.  Order must be positive and is
equal to the degree of the polynomial basis to be evaluated, plus 1.*/
    pub u_order: Option<i32>,
    /**Order in the V direction.  Order must be positive and is
equal to the degree of the polynomial basis to be evaluated, plus 1.*/
    pub v_order: Option<i32>,
    /**Knot vector for U direction providing U parameterization.
The length of this array must be ( uVertexCount + uOrder ), and its
entries must take on monotonically increasing values.*/
    pub u_knots: crate::foundation::VtArray<f64>,
    /**Knot vector for V direction providing U parameterization.
The length of this array must be ( vVertexCount + vOrder ), and its
entries must take on monotonically increasing values.*/
    pub v_knots: crate::foundation::VtArray<f64>,
    /**Interpret the control grid and knot vectors as representing
an open, geometrically closed, or geometrically closed and C2 continuous
surface along the U dimension.
\See: \UsdGeom_NurbsPatch_Form "NurbsPatch Form"*/
    pub u_form: crate::foundation::TfToken,
    /**Interpret the control grid and knot vectors as representing
an open, geometrically closed, or geometrically closed and C2 continuous
surface along the V dimension.
\See: \UsdGeom_NurbsPatch_Form "NurbsPatch Form"*/
    pub v_form: crate::foundation::TfToken,
    /**Provides the minimum and maximum parametric values (as defined
by uKnots) over which the surface is actually defined.  The minimum
must be less than the maximum, and greater than or equal to the
value of uKnots[uOrder-1].  The maxium must be less than or equal
to the last element's value in uKnots.*/
    pub u_range: Option<crate::foundation::GfVec2d>,
    /**Provides the minimum and maximum parametric values (as defined
by vKnots) over which the surface is actually defined.  The minimum
must be less than the maximum, and greater than or equal to the
value of vKnots[vOrder-1].  The maxium must be less than or equal
to the last element's value in vKnots.*/
    pub v_range: Option<crate::foundation::GfVec2d>,
    /**Optionally provides "w" components for each control point,
thus must be the same length as the points attribute.  If authored,
the patch will be rational.  If unauthored, the patch will be
polynomial, i.e. weight for all points is 1.0.
\Note: Some DCC's pre-weight the \*points, but in this schema,
\*points are not pre-weighted.*/
    pub point_weights: crate::foundation::VtArray<f64>,
    /**Each element specifies how many curves are present in each
"loop" of the trimCurve, and the length of the array determines how
many loops the trimCurve contains.  The sum of all elements is the
total nuber of curves in the trim, to which we will refer as
\*nCurves in describing the other trim attributes.*/
    pub trim_curve_counts: crate::foundation::VtArray<i32>,
    ///Flat list of orders for each of the \*nCurves curves.
    pub trim_curve_orders: crate::foundation::VtArray<i32>,
    /**Flat list of number of vertices for each of the
\*nCurves curves.*/
    pub trim_curve_vertex_counts: crate::foundation::VtArray<i32>,
    /**Flat list of parametric values for each of the
\*nCurves curves.  There will be as many knots as the sum over
all elements of \*vertexCounts plus the sum over all elements of
\*orders.*/
    pub trim_curve_knots: crate::foundation::VtArray<f64>,
    /**Flat list of minimum and maximum parametric values
(as defined by \*knots) for each of the \*nCurves curves.*/
    pub trim_curve_ranges: crate::foundation::VtArray<crate::foundation::GfVec2d>,
    /**Flat list of homogeneous 2D points (u, v, w) that comprise
the \*nCurves curves.  The number of points should be equal to the
um over all elements of \*vertexCounts.*/
    pub trim_curve_points: crate::foundation::VtArray<crate::foundation::GfVec3d>,
}
impl Default for UsdGeomNurbsPatch {
    fn default() -> Self {
        Self {
            visibility: crate::foundation::TfToken::new("inherited"),
            purpose: crate::foundation::TfToken::new("default"),
            xform_op_order: Vec::new(),
            extent: Vec::new(),
            primvars_display_color: Vec::new(),
            primvars_display_opacity: Vec::new(),
            double_sided: false,
            orientation: crate::foundation::TfToken::new("rightHanded"),
            points: Vec::new(),
            velocities: Vec::new(),
            accelerations: Vec::new(),
            normals: Vec::new(),
            u_vertex_count: None,
            v_vertex_count: None,
            u_order: None,
            v_order: None,
            u_knots: Vec::new(),
            v_knots: Vec::new(),
            u_form: crate::foundation::TfToken::new("open"),
            v_form: crate::foundation::TfToken::new("open"),
            u_range: None,
            v_range: None,
            point_weights: Vec::new(),
            trim_curve_counts: Vec::new(),
            trim_curve_orders: Vec::new(),
            trim_curve_vertex_counts: Vec::new(),
            trim_curve_knots: Vec::new(),
            trim_curve_ranges: Vec::new(),
            trim_curve_points: Vec::new(),
        }
    }
}
impl crate::schema::generated::UsdSchemaInfo for UsdGeomNurbsPatch {
    fn schema_name(&self) -> &'static str {
        "UsdGeomNurbsPatch"
    }
    fn attribute_metadata(
        &self,
    ) -> &'static [crate::schema::generated::AttributeMetadata] {
        static META: &[crate::schema::generated::AttributeMetadata] = &[
            crate::schema::generated::AttributeMetadata {
                usd_name: "visibility",
                usd_type: "token",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "purpose",
                usd_type: "token",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "xformOpOrder",
                usd_type: "token[]",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "extent",
                usd_type: "float3[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "primvars:displayColor",
                usd_type: "color3f[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "primvars:displayOpacity",
                usd_type: "float[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "doubleSided",
                usd_type: "bool",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "orientation",
                usd_type: "token",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "points",
                usd_type: "point3f[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "velocities",
                usd_type: "vector3f[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "accelerations",
                usd_type: "vector3f[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "normals",
                usd_type: "normal3f[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "uVertexCount",
                usd_type: "int",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "vVertexCount",
                usd_type: "int",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "uOrder",
                usd_type: "int",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "vOrder",
                usd_type: "int",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "uKnots",
                usd_type: "double[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "vKnots",
                usd_type: "double[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "uForm",
                usd_type: "token",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "vForm",
                usd_type: "token",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "uRange",
                usd_type: "double2",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "vRange",
                usd_type: "double2",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "pointWeights",
                usd_type: "double[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "trimCurve:counts",
                usd_type: "int[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "trimCurve:orders",
                usd_type: "int[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "trimCurve:vertexCounts",
                usd_type: "int[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "trimCurve:knots",
                usd_type: "double[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "trimCurve:ranges",
                usd_type: "double2[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "trimCurve:points",
                usd_type: "double3[]",
                is_uniform: false,
            },
        ];
        META
    }
}
impl UsdGeomNurbsPatch {
    #[inline]
    pub fn degree_u(&self) -> usize {
        self.u_order.map(|o| (o as usize).saturating_sub(1)).unwrap_or(0)
    }
    #[inline]
    pub fn degree_v(&self) -> usize {
        self.v_order.map(|o| (o as usize).saturating_sub(1)).unwrap_or(0)
    }
}
/**BasisCurves are a batched curve representation analogous to the
classic RIB definition via Basis and Curves statements. BasisCurves are
often used to render dense aggregate geometry like hair or grass.

A 'matrix' and 'vstep' associated with the \*basis are used to
interpolate the vertices of a cubic BasisCurves. (The basis attribute
is unused for linear BasisCurves.)

A single prim may have many curves whose count is determined implicitly by
the length of the \*curveVertexCounts vector.  Each individual curve is
composed of one or more segments. Each segment is defined by four vertices
for cubic curves and two vertices for linear curves. See the next section
for more information on how to map curve vertex counts to segment counts.

\## UsdGeomBasisCurves_Segment Segment Indexing
Interpolating a curve requires knowing how to decompose it into its
individual segments.

The segments of a cubic curve are determined by the vertex count,
the \*wrap (periodicity), and the vstep of the basis. For linear
curves, the basis token is ignored and only the vertex count and
wrap are needed.

cubic basis   | vstep
------------- | ------
bezier        | 3
catmullRom    | 1
bspline       | 1

The first segment of a cubic (nonperiodic) curve is always defined by its
first four points. The vstep is the increment used to determine what
vertex indices define the next segment.  For a two segment (nonperiodic)
bspline basis curve (vstep = 1), the first segment will be defined by
interpolating vertices [0, 1, 2, 3] and the second segment will be defined
by [1, 2, 3, 4].  For a two segment bezier basis curve (vstep = 3), the
first segment will be defined by interpolating vertices [0, 1, 2, 3] and
the second segment will be defined by [3, 4, 5, 6].  If the vstep is not
one, then you must take special care to make sure that the number of cvs
properly divides by your vstep. (The indices described are relative to
the initial vertex index for a batched curve.)

For periodic curves, at least one of the curve's initial vertices are
repeated to close the curve. For cubic curves, the number of vertices
repeated is '4 - vstep'. For linear curves, only one vertex is repeated
to close the loop.

Pinned curves are a special case of nonperiodic curves that only affects
the behavior of cubic Bspline and Catmull-Rom curves. To evaluate or render
pinned curves, a client must effectively add 'phantom points' at the
beginning and end of every curve in a batch.  These phantom points
are injected to ensure that the interpolated curve begins at P[0] and
ends at P[n-1].

For a curve with initial point P[0] and last point P[n-1], the phantom
points are defined as.
P[-1]  = 2 * P[0] - P[1]
P[n] = 2 * P[n-1] - P[n-2]

Pinned cubic curves will (usually) have to be unpacked into the standard
nonperiodic representation before rendering. This unpacking can add some
additional overhead. However, using pinned curves reduces the amount of
data recorded in a scene and (more importantly) better records the
authors' intent for interchange.

\Note: The additional phantom points mean that the minimum curve vertex
count for cubic bspline and catmullRom curves is 2.

Linear curve segments are defined by two vertices.
A two segment linear curve's first segment would be defined by
interpolating vertices [0, 1]. The second segment would be defined by
vertices [1, 2]. (Again, for a batched curve, indices are relative to
the initial vertex index.)

When validating curve topology, each renderable entry in the
curveVertexCounts vector must pass this check.

type    | wrap                        | validitity
------- | --------------------------- | ----------------
linear  | nonperiodic                 | curveVertexCounts[i] > 2
linear  | periodic                    | curveVertexCounts[i] > 3
cubic   | nonperiodic                 | (curveVertexCounts[i] - 4) % vstep == 0
cubic   | periodic                    | (curveVertexCounts[i]) % vstep == 0
cubic   | pinned (catmullRom/bspline) | (curveVertexCounts[i] - 2) >= 0

\## UsdGeomBasisCurves_BasisMatrix Cubic Vertex Interpolation

\[image] USDCurveBasisMatrix.png width=750

\## UsdGeomBasisCurves_Linear Linear Vertex Interpolation

Linear interpolation is always used on curves of type linear.
't' with domain [0, 1], the curve is defined by the equation
P0 * (1-t) + P1 * t. t at 0 describes the first point and t at 1 describes
the end point.

\## UsdGeomBasisCurves_PrimvarInterpolation Primvar Interpolation

For cubic curves, primvar data can be either interpolated cubically between
vertices or linearly across segments.  The corresponding token
for cubic interpolation is 'vertex' and for linear interpolation is
'varying'.  Per vertex data should be the same size as the number
of vertices in your curve.  Segment varying data is dependent on the
wrap (periodicity) and number of segments in your curve.  For linear curves,
varying and vertex data would be interpolated the same way.  By convention
varying is the preferred interpolation because of the association of
varying with linear interpolation.

\[image] USDCurvePrimvars.png

To convert an entry in the curveVertexCounts vector into a segment count
for an individual curve, apply these rules.  Sum up all the results in
order to compute how many total segments all curves have.

The following tables describe the expected segment count for the 'i'th
curve in a curve batch as well as the entire batch. Python syntax
like '[:]' (to describe all members of an array) and 'len(...)'
(to describe the length of an array) are used.

type    | wrap                        | curve segment count                    | batch segment count
------- | --------------------------- | -------------------------------------- | --------------------------
linear  | nonperiodic                 | curveVertexCounts[i] - 1               | sum(curveVertexCounts[:]) - len(curveVertexCounts)
linear  | periodic                    | curveVertexCounts[i]                   | sum(curveVertexCounts[:])
cubic   | nonperiodic                 | (curveVertexCounts[i] - 4) / vstep + 1 | sum(curveVertexCounts[:] - 4) / vstep + len(curveVertexCounts)
cubic   | periodic                    | curveVertexCounts[i] / vstep           | sum(curveVertexCounts[:]) / vstep
cubic   | pinned (catmullRom/bspline) | (curveVertexCounts[i] - 2) + 1         | sum(curveVertexCounts[:] - 2) + len(curveVertexCounts)

The following table descrives the expected size of varying
(linearly interpolated) data, derived from the segment counts computed
above.

wrap                | curve varying count          | batch varying count
------------------- | ---------------------------- | ------------------------------------------------
nonperiodic/pinned  | segmentCounts[i] + 1         | sum(segmentCounts[:]) + len(curveVertexCounts)
periodic            | segmentCounts[i]             | sum(segmentCounts[:])

Both curve types additionally define 'constant' interpolation for the
entire prim and 'uniform' interpolation as per curve data.


\Note: Take care when providing support for linearly interpolated data for
cubic curves. Its shape doesn't provide a one to one mapping with either
the number of curves (like 'uniform') or the number of vertices (like
'vertex') and so it is often overlooked. This is the only primitive in
UsdGeom (as of this writing) where this is true. For meshes, while they
use different interpolation methods, 'varying' and 'vertex' are both
specified per point. It's common to assume that curves follow a similar
pattern and build in structures and language for per primitive, per
element, and per point data only to come upon these arrays that don't
quite fit into either of those categories. It is
also common to conflate 'varying' with being per segment data and use the
segmentCount rules table instead of its neighboring varying data table
rules. We suspect that this is because for the common case of
nonperiodic cubic curves, both the provided segment count and varying data
size formula end with '+ 1'. While debugging, users may look at the double
'+ 1' as a mistake and try to remove it.  We take this time to enumerate
these issues because we've fallen into them before and hope that we save
others time in their own implementations.

As an example of deriving per curve segment and varying primvar data counts from
the wrap, type, basis, and curveVertexCount, the following table is provided.

wrap          | type    | basis   | curveVertexCount  | curveSegmentCount  | varyingDataCount
------------- | ------- | ------- | ----------------- | ------------------ | -------------------------
nonperiodic   | linear  | N/A     | [2 3 2 5]         | [1 2 1 4]          | [2 3 2 5]
nonperiodic   | cubic   | bezier  | [4 7 10 4 7]      | [1 2 3 1 2]        | [2 3 4 2 3]
nonperiodic   | cubic   | bspline | [5 4 6 7]         | [2 1 3 4]          | [3 2 4 5]
periodic      | cubic   | bezier  | [6 9 6]           | [2 3 2]            | [2 3 2]
periodic      | linear  | N/A     | [3 7]             | [3 7]              | [3 7]

\## UsdGeomBasisCurves_TubesAndRibbons Tubes and Ribbons

The strictest definition of a curve as an infinitely thin wire is not
particularly useful for describing production scenes. The additional
\*widths and \*normals attributes can be used to describe cylindrical
tubes and or flat oriented ribbons.

Curves with only widths defined are imaged as tubes with radius
'width / 2'. Curves with both widths and normals are imaged as ribbons
oriented in the direction of the interpolated normal vectors.

While not technically UsdGeomPrimvars, widths and normals
also have interpolation metadata. It's common for authored widths to have
constant, varying, or vertex interpolation
(see UsdGeomCurves::GetWidthsInterpolation()).  It's common for
authored normals to have varying interpolation
(see UsdGeomPointBased::GetNormalsInterpolation()).

\[image] USDCurveHydra.png

The file used to generate these curves can be found in
extras/usd/examples/usdGeomExamples/basisCurves.usda.  It's provided
as a reference on how to properly image both tubes and ribbons. The first
row of curves are linear; the second are cubic bezier. (We aim in future
releases of HdSt to fix the discontinuity seen with broken tangents to
better match offline renderers like RenderMan.) The yellow and violet
cubic curves represent cubic vertex width interpolation for which there is
no equivalent for linear curves.

\Note: How did this prim type get its name?  This prim is a portmanteau of
two different statements in the original RenderMan specification:
'Basis' and 'Curves'.*/
#[derive(Clone, Debug)]
pub struct UsdGeomBasisCurves {
    /**Visibility is meant to be the simplest form of "pruning"
visibility that is supported by most DCC apps.  Visibility is
animatable, allowing a sub-tree of geometry to be present for some
segment of a shot, and absent from others; unlike the action of
deactivating geometry prims, invisible geometry is still
available for inspection, for positioning, for defining volumes, etc.*/
    pub visibility: crate::foundation::TfToken,
    /**Purpose is a classification of geometry into categories that
can each be independently included or excluded from traversals of prims
on a stage, such as rendering or bounding-box computation traversals.

See \UsdGeom_ImageablePurpose for more detail about how
\*purpose is computed and used.*/
    pub purpose: crate::foundation::TfToken,
    /**Encodes the sequence of transformation operations in the
order in which they should be pushed onto a transform stack while
visiting a UsdStage's prims in a graph traversal that will effect
the desired positioning for this prim and its descendant prims.

You should rarely, if ever, need to manipulate this attribute directly.
It is managed by the AddXformOp(), SetResetXformStack(), and
SetXformOpOrder(), and consulted by GetOrderedXformOps() and
GetLocalTransformation().*/
    pub xform_op_order: crate::foundation::VtArray<crate::foundation::TfToken>,
    /**Extent is a three dimensional range measuring the geometric
extent of the authored gprim in its own local space (i.e. its own
transform not applied), \*without accounting for any shader-induced
displacement. If __any__ extent value has been authored for a given
Boundable, then it should be authored at every timeSample at which
geometry-affecting properties are authored, to ensure correct
evaluation via ComputeExtent(). If __no__ extent value has been
authored, then ComputeExtent() will call the Boundable's registered
ComputeExtentFunction(), which may be expensive, which is why we
strongly encourage proper authoring of extent.
\See: ComputeExtent()
\See: \UsdGeom_Boundable_Extent.

An authored extent on a prim which has children is expected to include
the extent of all children, as they will be pruned from BBox computation
during traversal.*/
    pub extent: crate::foundation::VtArray<crate::foundation::GfVec3f>,
    /**It is useful to have an "official" colorSet that can be used
as a display or modeling color, even in the absence of any specified
shader for a gprim.  DisplayColor serves this role; because it is a
UsdGeomPrimvar, it can also be used as a gprim override for any shader
that consumes a \*displayColor parameter.*/
    pub primvars_display_color: crate::foundation::VtArray<crate::foundation::GfVec3f>,
    /**Companion to \*displayColor that specifies opacity, broken
out as an independent attribute rather than an rgba color, both so that
each can be independently overridden, and because shaders rarely consume
rgba parameters.*/
    pub primvars_display_opacity: crate::foundation::VtArray<f32>,
    /**Although some renderers treat all parametric or polygonal
surfaces as if they were effectively laminae with outward-facing
normals on both sides, some renderers derive significant optimizations
by considering these surfaces to have only a single outward side,
typically determined by control-point winding order and/or
\*orientation.  By doing so they can perform "backface culling" to
avoid drawing the many polygons of most closed surfaces that face away
from the viewer.

However, it is often advantageous to model thin objects such as paper
and cloth as single, open surfaces that must be viewable from both
sides, always.  Setting a gprim's \*doubleSided attribute to
\\c true instructs all renderers to disable optimizations such as
backface culling for the gprim, and attempt (not all renderers are able
to do so, but the USD reference GL renderer always will) to provide
forward-facing normals on each side of the surface for lighting
calculations.*/
    pub double_sided: bool,
    /**Orientation specifies whether the gprim's surface normal
should be computed using the right hand rule, or the left hand rule.
Please see \UsdGeom_WindingOrder for a deeper explanation and
generalization of orientation to composed scenes with transformation
hierarchies.*/
    pub orientation: crate::foundation::TfToken,
    /**The primary geometry attribute for all PointBased
primitives, describes points in (local) space.*/
    pub points: crate::foundation::VtArray<crate::foundation::GfVec3f>,
    /**If provided, 'velocities' should be used by renderers to

compute positions between samples for the 'points' attribute, rather
than interpolating between neighboring 'points' samples.  This is the
only reasonable means of computing motion blur for topologically
varying PointBased primitives.  It follows that the length of each
'velocities' sample must match the length of the corresponding
'points' sample.  Velocity is measured in position units per second,
as per most simulation software. To convert to position units per
UsdTimeCode, divide by UsdStage::GetTimeCodesPerSecond().

See also \UsdGeom_VelocityInterpolation .*/
    pub velocities: crate::foundation::VtArray<crate::foundation::GfVec3f>,
    /**If provided, 'accelerations' should be used with
velocities to compute positions between samples for the 'points'
attribute rather than interpolating between neighboring 'points'
samples. Acceleration is measured in position units per second-squared.
To convert to position units per squared UsdTimeCode, divide by the
square of UsdStage::GetTimeCodesPerSecond().*/
    pub accelerations: crate::foundation::VtArray<crate::foundation::GfVec3f>,
    /**Provide an object-space orientation for individual points,
which, depending on subclass, may define a surface, curve, or free
points.  Note that 'normals' should not be authored on any Mesh that
is subdivided, since the subdivision algorithm will define its own
normals. 'normals' is not a generic primvar, but the number of elements
in this attribute will be determined by its 'interpolation'.  See
\SetNormalsInterpolation() . If 'normals' and 'primvars:normals'
are both specified, the latter has precedence.*/
    pub normals: crate::foundation::VtArray<crate::foundation::GfVec3f>,
    /**Curves-derived primitives can represent multiple distinct,
potentially disconnected curves.  The length of 'curveVertexCounts'
gives the number of such curves, and each element describes the
number of vertices in the corresponding curve*/
    pub curve_vertex_counts: crate::foundation::VtArray<i32>,
    /**Provides width specification for the curves, whose application
will depend on whether the curve is oriented (normals are defined for
it), in which case widths are "ribbon width", or unoriented, in which
case widths are cylinder width.  'widths' is not a generic Primvar,
but the number of elements in this attribute will be determined by
its 'interpolation'.  See \SetWidthsInterpolation() .  If 'widths'
and 'primvars:widths' are both specified, the latter has precedence.*/
    pub widths: crate::foundation::VtArray<f32>,
    /**Linear curves interpolate linearly between two vertices.
Cubic curves use a basis matrix with four vertices to interpolate a segment.*/
    pub type_: crate::foundation::TfToken,
    /**The basis specifies the vstep and matrix used for cubic
interpolation.  \Note: The 'hermite' and 'power' tokens have been
removed. We've provided UsdGeomHermiteCurves
as an alternative for the 'hermite' basis.*/
    pub basis: crate::foundation::TfToken,
    /**If wrap is set to periodic, the curve when rendered will
repeat the initial vertices (dependent on the vstep) to close the
curve. If wrap is set to 'pinned', phantom points may be created
to ensure that the curve interpolation starts at P[0] and ends at P[n-1].*/
    pub wrap: crate::foundation::TfToken,
}
impl Default for UsdGeomBasisCurves {
    fn default() -> Self {
        Self {
            visibility: crate::foundation::TfToken::new("inherited"),
            purpose: crate::foundation::TfToken::new("default"),
            xform_op_order: Vec::new(),
            extent: Vec::new(),
            primvars_display_color: Vec::new(),
            primvars_display_opacity: Vec::new(),
            double_sided: false,
            orientation: crate::foundation::TfToken::new("rightHanded"),
            points: Vec::new(),
            velocities: Vec::new(),
            accelerations: Vec::new(),
            normals: Vec::new(),
            curve_vertex_counts: Vec::new(),
            widths: Vec::new(),
            type_: crate::foundation::TfToken::new("cubic"),
            basis: crate::foundation::TfToken::new("bezier"),
            wrap: crate::foundation::TfToken::new("nonperiodic"),
        }
    }
}
impl crate::schema::generated::UsdSchemaInfo for UsdGeomBasisCurves {
    fn schema_name(&self) -> &'static str {
        "UsdGeomBasisCurves"
    }
    fn attribute_metadata(
        &self,
    ) -> &'static [crate::schema::generated::AttributeMetadata] {
        static META: &[crate::schema::generated::AttributeMetadata] = &[
            crate::schema::generated::AttributeMetadata {
                usd_name: "visibility",
                usd_type: "token",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "purpose",
                usd_type: "token",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "xformOpOrder",
                usd_type: "token[]",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "extent",
                usd_type: "float3[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "primvars:displayColor",
                usd_type: "color3f[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "primvars:displayOpacity",
                usd_type: "float[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "doubleSided",
                usd_type: "bool",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "orientation",
                usd_type: "token",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "points",
                usd_type: "point3f[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "velocities",
                usd_type: "vector3f[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "accelerations",
                usd_type: "vector3f[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "normals",
                usd_type: "normal3f[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "curveVertexCounts",
                usd_type: "int[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "widths",
                usd_type: "float[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "type",
                usd_type: "token",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "basis",
                usd_type: "token",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "wrap",
                usd_type: "token",
                is_uniform: true,
            },
        ];
        META
    }
}
/**This schema is analagous to NURBS Curves in packages like Maya
and Houdini, often used for interchange of rigging and modeling curves.
Unlike Maya, this curve spec supports batching of multiple curves into a
single prim, widths, and normals in the schema.  Additionally, we require
'numSegments + 2 * degree + 1' knots (2 more than maya does).  This is to
be more consistent with RenderMan's NURBS patch specification.

To express a periodic curve:
- knot[0] = knot[1] - (knots[-2] - knots[-3];
- knot[-1] = knot[-2] + (knot[2] - knots[1]);

To express a nonperiodic curve:
- knot[0] = knot[1];
- knot[-1] = knot[-2];

In spite of these slight differences in the spec, curves generated in Maya
should be preserved when roundtripping.

\*order and \*range, when representing a batched NurbsCurve should be
authored one value per curve.  \*knots should be the concatentation of
all batched curves.*/
#[derive(Clone, Debug)]
pub struct UsdGeomNurbsCurves {
    /**Visibility is meant to be the simplest form of "pruning"
visibility that is supported by most DCC apps.  Visibility is
animatable, allowing a sub-tree of geometry to be present for some
segment of a shot, and absent from others; unlike the action of
deactivating geometry prims, invisible geometry is still
available for inspection, for positioning, for defining volumes, etc.*/
    pub visibility: crate::foundation::TfToken,
    /**Purpose is a classification of geometry into categories that
can each be independently included or excluded from traversals of prims
on a stage, such as rendering or bounding-box computation traversals.

See \UsdGeom_ImageablePurpose for more detail about how
\*purpose is computed and used.*/
    pub purpose: crate::foundation::TfToken,
    /**Encodes the sequence of transformation operations in the
order in which they should be pushed onto a transform stack while
visiting a UsdStage's prims in a graph traversal that will effect
the desired positioning for this prim and its descendant prims.

You should rarely, if ever, need to manipulate this attribute directly.
It is managed by the AddXformOp(), SetResetXformStack(), and
SetXformOpOrder(), and consulted by GetOrderedXformOps() and
GetLocalTransformation().*/
    pub xform_op_order: crate::foundation::VtArray<crate::foundation::TfToken>,
    /**Extent is a three dimensional range measuring the geometric
extent of the authored gprim in its own local space (i.e. its own
transform not applied), \*without accounting for any shader-induced
displacement. If __any__ extent value has been authored for a given
Boundable, then it should be authored at every timeSample at which
geometry-affecting properties are authored, to ensure correct
evaluation via ComputeExtent(). If __no__ extent value has been
authored, then ComputeExtent() will call the Boundable's registered
ComputeExtentFunction(), which may be expensive, which is why we
strongly encourage proper authoring of extent.
\See: ComputeExtent()
\See: \UsdGeom_Boundable_Extent.

An authored extent on a prim which has children is expected to include
the extent of all children, as they will be pruned from BBox computation
during traversal.*/
    pub extent: crate::foundation::VtArray<crate::foundation::GfVec3f>,
    /**It is useful to have an "official" colorSet that can be used
as a display or modeling color, even in the absence of any specified
shader for a gprim.  DisplayColor serves this role; because it is a
UsdGeomPrimvar, it can also be used as a gprim override for any shader
that consumes a \*displayColor parameter.*/
    pub primvars_display_color: crate::foundation::VtArray<crate::foundation::GfVec3f>,
    /**Companion to \*displayColor that specifies opacity, broken
out as an independent attribute rather than an rgba color, both so that
each can be independently overridden, and because shaders rarely consume
rgba parameters.*/
    pub primvars_display_opacity: crate::foundation::VtArray<f32>,
    /**Although some renderers treat all parametric or polygonal
surfaces as if they were effectively laminae with outward-facing
normals on both sides, some renderers derive significant optimizations
by considering these surfaces to have only a single outward side,
typically determined by control-point winding order and/or
\*orientation.  By doing so they can perform "backface culling" to
avoid drawing the many polygons of most closed surfaces that face away
from the viewer.

However, it is often advantageous to model thin objects such as paper
and cloth as single, open surfaces that must be viewable from both
sides, always.  Setting a gprim's \*doubleSided attribute to
\\c true instructs all renderers to disable optimizations such as
backface culling for the gprim, and attempt (not all renderers are able
to do so, but the USD reference GL renderer always will) to provide
forward-facing normals on each side of the surface for lighting
calculations.*/
    pub double_sided: bool,
    /**Orientation specifies whether the gprim's surface normal
should be computed using the right hand rule, or the left hand rule.
Please see \UsdGeom_WindingOrder for a deeper explanation and
generalization of orientation to composed scenes with transformation
hierarchies.*/
    pub orientation: crate::foundation::TfToken,
    /**The primary geometry attribute for all PointBased
primitives, describes points in (local) space.*/
    pub points: crate::foundation::VtArray<crate::foundation::GfVec3f>,
    /**If provided, 'velocities' should be used by renderers to

compute positions between samples for the 'points' attribute, rather
than interpolating between neighboring 'points' samples.  This is the
only reasonable means of computing motion blur for topologically
varying PointBased primitives.  It follows that the length of each
'velocities' sample must match the length of the corresponding
'points' sample.  Velocity is measured in position units per second,
as per most simulation software. To convert to position units per
UsdTimeCode, divide by UsdStage::GetTimeCodesPerSecond().

See also \UsdGeom_VelocityInterpolation .*/
    pub velocities: crate::foundation::VtArray<crate::foundation::GfVec3f>,
    /**If provided, 'accelerations' should be used with
velocities to compute positions between samples for the 'points'
attribute rather than interpolating between neighboring 'points'
samples. Acceleration is measured in position units per second-squared.
To convert to position units per squared UsdTimeCode, divide by the
square of UsdStage::GetTimeCodesPerSecond().*/
    pub accelerations: crate::foundation::VtArray<crate::foundation::GfVec3f>,
    /**Provide an object-space orientation for individual points,
which, depending on subclass, may define a surface, curve, or free
points.  Note that 'normals' should not be authored on any Mesh that
is subdivided, since the subdivision algorithm will define its own
normals. 'normals' is not a generic primvar, but the number of elements
in this attribute will be determined by its 'interpolation'.  See
\SetNormalsInterpolation() . If 'normals' and 'primvars:normals'
are both specified, the latter has precedence.*/
    pub normals: crate::foundation::VtArray<crate::foundation::GfVec3f>,
    /**Curves-derived primitives can represent multiple distinct,
potentially disconnected curves.  The length of 'curveVertexCounts'
gives the number of such curves, and each element describes the
number of vertices in the corresponding curve*/
    pub curve_vertex_counts: crate::foundation::VtArray<i32>,
    /**Provides width specification for the curves, whose application
will depend on whether the curve is oriented (normals are defined for
it), in which case widths are "ribbon width", or unoriented, in which
case widths are cylinder width.  'widths' is not a generic Primvar,
but the number of elements in this attribute will be determined by
its 'interpolation'.  See \SetWidthsInterpolation() .  If 'widths'
and 'primvars:widths' are both specified, the latter has precedence.*/
    pub widths: crate::foundation::VtArray<f32>,
    /**Order of the curve.  Order must be positive and is
equal to the degree of the polynomial basis to be evaluated, plus 1.
Its value for the 'i'th curve must be less than or equal to
curveVertexCount[i]*/
    pub order: crate::foundation::VtArray<i32>,
    /**Knot vector providing curve parameterization.
The length of the slice of the array for the ith curve
must be ( curveVertexCount[i] + order[i] ), and its
entries must take on monotonically increasing values.*/
    pub knots: crate::foundation::VtArray<f64>,
    /**Provides the minimum and maximum parametric values (as defined
by knots) over which the curve is actually defined.  The minimum must
be less than the maximum, and greater than or equal to the value of the
knots['i'th curve slice][order[i]-1]. The maxium must be less
than or equal to the last element's value in knots['i'th curve slice].
Range maps to (vmin, vmax) in the RenderMan spec.*/
    pub ranges: crate::foundation::VtArray<crate::foundation::GfVec2d>,
    /**Optionally provides "w" components for each control point,
thus must be the same length as the points attribute.  If authored,
the curve will be rational.  If unauthored, the curve will be
polynomial, i.e. weight for all points is 1.0.
\Note: Some DCC's pre-weight the \*points, but in this schema,
\*points are not pre-weighted.*/
    pub point_weights: crate::foundation::VtArray<f64>,
}
impl Default for UsdGeomNurbsCurves {
    fn default() -> Self {
        Self {
            visibility: crate::foundation::TfToken::new("inherited"),
            purpose: crate::foundation::TfToken::new("default"),
            xform_op_order: Vec::new(),
            extent: Vec::new(),
            primvars_display_color: Vec::new(),
            primvars_display_opacity: Vec::new(),
            double_sided: false,
            orientation: crate::foundation::TfToken::new("rightHanded"),
            points: Vec::new(),
            velocities: Vec::new(),
            accelerations: Vec::new(),
            normals: Vec::new(),
            curve_vertex_counts: Vec::new(),
            widths: Vec::new(),
            order: Vec::new(),
            knots: Vec::new(),
            ranges: Vec::new(),
            point_weights: Vec::new(),
        }
    }
}
impl crate::schema::generated::UsdSchemaInfo for UsdGeomNurbsCurves {
    fn schema_name(&self) -> &'static str {
        "UsdGeomNurbsCurves"
    }
    fn attribute_metadata(
        &self,
    ) -> &'static [crate::schema::generated::AttributeMetadata] {
        static META: &[crate::schema::generated::AttributeMetadata] = &[
            crate::schema::generated::AttributeMetadata {
                usd_name: "visibility",
                usd_type: "token",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "purpose",
                usd_type: "token",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "xformOpOrder",
                usd_type: "token[]",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "extent",
                usd_type: "float3[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "primvars:displayColor",
                usd_type: "color3f[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "primvars:displayOpacity",
                usd_type: "float[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "doubleSided",
                usd_type: "bool",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "orientation",
                usd_type: "token",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "points",
                usd_type: "point3f[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "velocities",
                usd_type: "vector3f[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "accelerations",
                usd_type: "vector3f[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "normals",
                usd_type: "normal3f[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "curveVertexCounts",
                usd_type: "int[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "widths",
                usd_type: "float[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "order",
                usd_type: "int[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "knots",
                usd_type: "double[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "ranges",
                usd_type: "double2[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "pointWeights",
                usd_type: "double[]",
                is_uniform: false,
            },
        ];
        META
    }
}
impl UsdGeomNurbsCurves {
    /// Degree of curve `i` (order - 1).
    #[inline]
    pub fn degree(&self, i: usize) -> usize {
        (self.order[i] as usize).saturating_sub(1)
    }
    /// Knot slice for curve `i`.
    #[inline]
    pub fn knot_slice(&self, i: usize) -> &[f64] {
        let mut offset = 0usize;
        for j in 0..i {
            offset += self.curve_vertex_counts[j] as usize + self.order[j] as usize;
        }
        let len = self.curve_vertex_counts[i] as usize + self.order[i] as usize;
        &self.knots[offset..offset + len]
    }
    /// Control-point slice for curve `i`.
    #[inline]
    pub fn control_points(&self, i: usize) -> &[crate::foundation::GfVec3f] {
        let mut offset = 0usize;
        for j in 0..i {
            offset += self.curve_vertex_counts[j] as usize;
        }
        let len = self.curve_vertex_counts[i] as usize;
        &self.points[offset..offset + len]
    }
    /// Weight slice for curve `i`, or `None` if unweighted.
    #[inline]
    pub fn weight_slice(&self, i: usize) -> Option<&[f64]> {
        if self.point_weights.is_empty() {
            return None;
        }
        let mut offset = 0usize;
        for j in 0..i {
            offset += self.curve_vertex_counts[j] as usize;
        }
        let len = self.curve_vertex_counts[i] as usize;
        Some(&self.point_weights[offset..offset + len])
    }
}
/**Points are analogous to the <A HREF="https://renderman.pixar.com/resources/RenderMan_20/appnote.18.html">RiPoints spec</A>.

Points can be an efficient means of storing and rendering particle
effects comprised of thousands or millions of small particles.  Points
generally receive a single shading sample each, which should take
\*normals into account, if present.

While not technically UsdGeomPrimvars, the widths and normals also
have interpolation metadata.  It's common for authored widths and normals
to have constant or varying interpolation.*/
#[derive(Clone, Debug)]
pub struct UsdGeomPoints {
    /**Visibility is meant to be the simplest form of "pruning"
visibility that is supported by most DCC apps.  Visibility is
animatable, allowing a sub-tree of geometry to be present for some
segment of a shot, and absent from others; unlike the action of
deactivating geometry prims, invisible geometry is still
available for inspection, for positioning, for defining volumes, etc.*/
    pub visibility: crate::foundation::TfToken,
    /**Purpose is a classification of geometry into categories that
can each be independently included or excluded from traversals of prims
on a stage, such as rendering or bounding-box computation traversals.

See \UsdGeom_ImageablePurpose for more detail about how
\*purpose is computed and used.*/
    pub purpose: crate::foundation::TfToken,
    /**Encodes the sequence of transformation operations in the
order in which they should be pushed onto a transform stack while
visiting a UsdStage's prims in a graph traversal that will effect
the desired positioning for this prim and its descendant prims.

You should rarely, if ever, need to manipulate this attribute directly.
It is managed by the AddXformOp(), SetResetXformStack(), and
SetXformOpOrder(), and consulted by GetOrderedXformOps() and
GetLocalTransformation().*/
    pub xform_op_order: crate::foundation::VtArray<crate::foundation::TfToken>,
    /**Extent is a three dimensional range measuring the geometric
extent of the authored gprim in its own local space (i.e. its own
transform not applied), \*without accounting for any shader-induced
displacement. If __any__ extent value has been authored for a given
Boundable, then it should be authored at every timeSample at which
geometry-affecting properties are authored, to ensure correct
evaluation via ComputeExtent(). If __no__ extent value has been
authored, then ComputeExtent() will call the Boundable's registered
ComputeExtentFunction(), which may be expensive, which is why we
strongly encourage proper authoring of extent.
\See: ComputeExtent()
\See: \UsdGeom_Boundable_Extent.

An authored extent on a prim which has children is expected to include
the extent of all children, as they will be pruned from BBox computation
during traversal.*/
    pub extent: crate::foundation::VtArray<crate::foundation::GfVec3f>,
    /**It is useful to have an "official" colorSet that can be used
as a display or modeling color, even in the absence of any specified
shader for a gprim.  DisplayColor serves this role; because it is a
UsdGeomPrimvar, it can also be used as a gprim override for any shader
that consumes a \*displayColor parameter.*/
    pub primvars_display_color: crate::foundation::VtArray<crate::foundation::GfVec3f>,
    /**Companion to \*displayColor that specifies opacity, broken
out as an independent attribute rather than an rgba color, both so that
each can be independently overridden, and because shaders rarely consume
rgba parameters.*/
    pub primvars_display_opacity: crate::foundation::VtArray<f32>,
    /**Although some renderers treat all parametric or polygonal
surfaces as if they were effectively laminae with outward-facing
normals on both sides, some renderers derive significant optimizations
by considering these surfaces to have only a single outward side,
typically determined by control-point winding order and/or
\*orientation.  By doing so they can perform "backface culling" to
avoid drawing the many polygons of most closed surfaces that face away
from the viewer.

However, it is often advantageous to model thin objects such as paper
and cloth as single, open surfaces that must be viewable from both
sides, always.  Setting a gprim's \*doubleSided attribute to
\\c true instructs all renderers to disable optimizations such as
backface culling for the gprim, and attempt (not all renderers are able
to do so, but the USD reference GL renderer always will) to provide
forward-facing normals on each side of the surface for lighting
calculations.*/
    pub double_sided: bool,
    /**Orientation specifies whether the gprim's surface normal
should be computed using the right hand rule, or the left hand rule.
Please see \UsdGeom_WindingOrder for a deeper explanation and
generalization of orientation to composed scenes with transformation
hierarchies.*/
    pub orientation: crate::foundation::TfToken,
    /**The primary geometry attribute for all PointBased
primitives, describes points in (local) space.*/
    pub points: crate::foundation::VtArray<crate::foundation::GfVec3f>,
    /**If provided, 'velocities' should be used by renderers to

compute positions between samples for the 'points' attribute, rather
than interpolating between neighboring 'points' samples.  This is the
only reasonable means of computing motion blur for topologically
varying PointBased primitives.  It follows that the length of each
'velocities' sample must match the length of the corresponding
'points' sample.  Velocity is measured in position units per second,
as per most simulation software. To convert to position units per
UsdTimeCode, divide by UsdStage::GetTimeCodesPerSecond().

See also \UsdGeom_VelocityInterpolation .*/
    pub velocities: crate::foundation::VtArray<crate::foundation::GfVec3f>,
    /**If provided, 'accelerations' should be used with
velocities to compute positions between samples for the 'points'
attribute rather than interpolating between neighboring 'points'
samples. Acceleration is measured in position units per second-squared.
To convert to position units per squared UsdTimeCode, divide by the
square of UsdStage::GetTimeCodesPerSecond().*/
    pub accelerations: crate::foundation::VtArray<crate::foundation::GfVec3f>,
    /**Provide an object-space orientation for individual points,
which, depending on subclass, may define a surface, curve, or free
points.  Note that 'normals' should not be authored on any Mesh that
is subdivided, since the subdivision algorithm will define its own
normals. 'normals' is not a generic primvar, but the number of elements
in this attribute will be determined by its 'interpolation'.  See
\SetNormalsInterpolation() . If 'normals' and 'primvars:normals'
are both specified, the latter has precedence.*/
    pub normals: crate::foundation::VtArray<crate::foundation::GfVec3f>,
    /**Widths are defined as the \*diameter of the points, in
object space.  'widths' is not a generic Primvar, but
the number of elements in this attribute will be determined by
its 'interpolation'.  See \SetWidthsInterpolation() .  If
'widths' and 'primvars:widths' are both specified, the latter
has precedence.*/
    pub widths: crate::foundation::VtArray<f32>,
    /**Ids are optional; if authored, the ids array should be the same
length as the points array, specifying (at each timesample if
point identities are changing) the id of each point. The
type is signed intentionally, so that clients can encode some
binary state on Id'd points without adding a separate
primvar.*/
    pub ids: crate::foundation::VtArray<i64>,
}
impl Default for UsdGeomPoints {
    fn default() -> Self {
        Self {
            visibility: crate::foundation::TfToken::new("inherited"),
            purpose: crate::foundation::TfToken::new("default"),
            xform_op_order: Vec::new(),
            extent: Vec::new(),
            primvars_display_color: Vec::new(),
            primvars_display_opacity: Vec::new(),
            double_sided: false,
            orientation: crate::foundation::TfToken::new("rightHanded"),
            points: Vec::new(),
            velocities: Vec::new(),
            accelerations: Vec::new(),
            normals: Vec::new(),
            widths: Vec::new(),
            ids: Vec::new(),
        }
    }
}
impl crate::schema::generated::UsdSchemaInfo for UsdGeomPoints {
    fn schema_name(&self) -> &'static str {
        "UsdGeomPoints"
    }
    fn attribute_metadata(
        &self,
    ) -> &'static [crate::schema::generated::AttributeMetadata] {
        static META: &[crate::schema::generated::AttributeMetadata] = &[
            crate::schema::generated::AttributeMetadata {
                usd_name: "visibility",
                usd_type: "token",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "purpose",
                usd_type: "token",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "xformOpOrder",
                usd_type: "token[]",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "extent",
                usd_type: "float3[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "primvars:displayColor",
                usd_type: "color3f[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "primvars:displayOpacity",
                usd_type: "float[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "doubleSided",
                usd_type: "bool",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "orientation",
                usd_type: "token",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "points",
                usd_type: "point3f[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "velocities",
                usd_type: "vector3f[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "accelerations",
                usd_type: "vector3f[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "normals",
                usd_type: "normal3f[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "widths",
                usd_type: "float[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "ids",
                usd_type: "int64[]",
                is_uniform: false,
            },
        ];
        META
    }
}
/**Encodes vectorized instancing of multiple, potentially
animated, prototypes (object/instance masters), which can be arbitrary
prims/subtrees on a UsdStage.

PointInstancer is a "multi instancer", as it allows multiple prototypes
to be scattered among its "points".  We use a UsdRelationship
\*prototypes to identify and order all of the possible prototypes, by
targeting the root prim of each prototype.  The ordering imparted by
relationships associates a zero-based integer with each prototype, and
it is these integers we use to identify the prototype of each instance,
compactly, and allowing prototypes to be swapped out without needing to
reauthor all of the per-instance data.

The PointInstancer schema is designed to scale to billions of instances,
which motivates the choice to split the per-instance transformation into
position, (quaternion) orientation, and scales, rather than a
4x4 matrix per-instance.  In addition to requiring fewer bytes even if
all elements are authored (32 bytes vs 64 for a single-precision 4x4
matrix), we can also be selective about which attributes need to animate
over time, for substantial data reduction in many cases.

Note that PointInstancer is \*not a Gprim, since it is not a graphical
primitive by any stretch of the imagination. It \*is, however,
Boundable, since we will sometimes want to treat the entire PointInstancer
similarly to a procedural, from the perspective of inclusion or framing.

\## UsdGeomPointInstancer_varyingTopo Varying Instance Identity over Time

PointInstancers originating from simulations often have the characteristic
that points/instances are "born", move around for some time period, and then
die (or leave the area of interest). In such cases, billions of instances
may be birthed over time, while at any \*specific time, only a much
smaller number are actually alive.  To encode this situation efficiently,
the simulator may re-use indices in the instance arrays, when a particle
dies, its index will be taken over by a new particle that may be birthed in
a much different location.  This presents challenges both for
identity-tracking, and for motion-blur.

We facilitate identity tracking by providing an optional, animatable
\*ids attribute, that specifies the 64 bit integer ID of the particle
at each index, at each point in time.  If the simulator keeps monotonically
increasing a particle-count each time a new particle is birthed, it will
serve perfectly as particle \*ids.

We facilitate motion blur for varying-topology particle streams by
optionally allowing per-instance \*velocities and \*angularVelocities
to be authored.  If instance transforms are requested at a time between
samples and either of the velocity attributes is authored, then we will
not attempt to interpolate samples of \*positions or \*orientations.
If not authored, and the bracketing samples have the same length, then we
will interpolate.

\## UsdGeomPointInstancer_transform Computing an Instance Transform

Each instance's transformation is a combination of the SRT affine transform
described by its scale, orientation, and position, applied \*after
(i.e. less locally than) the local to parent transformation computed at
the root of the prototype it is instancing.

If your processing of prototype geometry naturally takes into account the
transform of the prototype root, then this term can be omitted from the
computation of each instance transform, and this can be controlled when
computing instance transformation matrices using the
UsdGeomPointInstancer::PrototypeXformInclusion enumeration.

To understand the computation of the instance transform, in order to put
an instance of a PointInstancer into the space of the PointInstancer's
parent prim we do the following:

1. Apply (most locally) the authored local to parent transformation for
<em>prototypes[protoIndices[i]]</em>
2. If *scales* is authored, next apply the scaling matrix from *scales[i]*
3. If *orientations* is authored: **if *angularVelocities* is authored**,
first multiply *orientations[i]* by the unit quaternion derived by scaling
*angularVelocities[i]* by the \UsdGeom_PITimeScaling "time differential"
from the left-bracketing timeSample for *orientation* to the requested
evaluation time *t*, storing the result in *R*, **else** assign *R*
directly from *orientations[i]*.  Apply the rotation matrix derived
from *R*.
4. Apply the translation derived from *positions[i]*. If *velocities* is
authored, apply the translation deriving from *velocities[i]* scaled by
the time differential from the left-bracketing timeSample for *positions*
to the requested evaluation time *t*.
5. Least locally, apply the transformation authored on the PointInstancer
prim itself (or the UsdGeomImageable::ComputeLocalToWorldTransform() of the
PointInstancer to put the instance directly into world space)

If neither *velocities* nor *angularVelocities* are authored, we fallback to
standard position and orientation computation logic (using linear
interpolation between timeSamples) as described by
\UsdGeom_VelocityInterpolation .

\UsdGeom_PITimeScaling
<b>Scaling Velocities for Interpolation</b>

When computing time-differentials by which to apply velocity or
angularVelocity to positions or orientations, we must scale by
( 1.0 / UsdStage::GetTimeCodesPerSecond() ), because velocities are recorded
in units/second, while we are interpolating in UsdTimeCode ordinates.

We provide both high and low-level API's for dealing with the
transformation as a matrix, both will compute the instance matrices using
multiple threads; the low-level API allows the client to cache unvarying
inputs so that they need not be read duplicately when computing over
time.

See also \UsdGeom_VelocityInterpolation .

\## UsdGeomPointInstancer_primvars Primvars on PointInstancer

\UsdGeomPrimvar "Primvars" authored on a PointInstancer prim should
always be applied to each instance with \*constant interpolation at
the root of the instance.  When you are authoring primvars on a
PointInstancer, think about it as if you were authoring them on a
point-cloud (e.g. a UsdGeomPoints gprim).  The same
<A HREF="https://renderman.pixar.com/resources/RenderMan_20/appnote.22.html#classSpecifiers">interpolation rules for points</A> apply here, substituting
"instance" for "point".

In other words, the (constant) value extracted for each instance
from the authored primvar value depends on the authored \*interpolation
and \*elementSize of the primvar, as follows:
\- <b>constant</b> or <b>uniform</b> : the entire authored value of the
primvar should be applied exactly to each instance.
\- <b>varying</b>, <b>vertex</b>, or <b>faceVarying</b>: the first
\*elementSize elements of the authored primvar array should be assigned to
instance zero, the second \*elementSize elements should be assigned to
instance one, and so forth.


\## UsdGeomPointInstancer_masking Masking Instances: "Deactivating" and Invising

Often a PointInstancer is created "upstream" in a graphics pipeline, and
the needs of "downstream" clients necessitate eliminating some of the
instances from further consideration.  Accomplishing this pruning by
re-authoring all of the per-instance attributes is not very attractive,
since it may mean destructively editing a large quantity of data.  We
therefore provide means of "masking" instances by ID, such that the
instance data is unmolested, but per-instance transform and primvar data
can be retrieved with the no-longer-desired instances eliminated from the
(smaller) arrays.  PointInstancer allows two independent means of masking
instances by ID, each with different features that meet the needs of
various clients in a pipeline.  Both pruning features' lists of ID's are
combined to produce the mask returned by ComputeMaskAtTime().

\Note: If a PointInstancer has no authored \*ids attribute, the masking
features will still be available, with the integers specifying element
position in the \*protoIndices array rather than ID.

\\subsection UsdGeomPointInstancer_inactiveIds InactiveIds: List-edited, Unvarying Masking

The first masking feature encodes a list of IDs in a list-editable metadatum
called \*inactiveIds, which, although it does not have any similar
impact to stage population as \UsdPrim::SetActive() "prim activation",
it shares with that feature that its application is uniform over all time.
Because it is list-editable, we can \*sparsely add and remove instances
from it in many layers.

This sparse application pattern makes \*inactiveIds a good choice when
further downstream clients may need to reverse masking decisions made
upstream, in a manner that is robust to many kinds of future changes to
the upstream data.

See ActivateId(), ActivateIds(), DeactivateId(), DeactivateIds(),
ActivateAllIds()

\\subsection UsdGeomPointInstancer_invisibleIds invisibleIds: Animatable Masking

The second masking feature encodes a list of IDs in a time-varying
Int64Array-valued UsdAttribute called \*invisibleIds , since it shares
with \UsdGeomImageable::GetVisibilityAttr() "Imageable visibility"
the ability to animate object visibility.

Unlike \*inactiveIds, overriding a set of opinions for \*invisibleIds
is not at all straightforward, because one will, in general need to
reauthor (in the overriding layer) **all** timeSamples for the attribute
just to change one Id's visibility state, so it cannot be authored
sparsely.  But it can be a very useful tool for situations like encoding
pre-computed camera-frustum culling of geometry when either or both of
the instances or the camera is animated.

See VisId(), VisIds(), InvisId(), InvisIds(), VisAllIds()

\## UsdGeomPointInstancer_protoProcessing Processing and Not Processing Prototypes

Any prim in the scenegraph can be targeted as a prototype by the
\*prototypes relationship.  We do not, however, provide a specific
mechanism for identifying prototypes as geometry that should not be drawn
(or processed) in their own, local spaces in the scenegraph.  We
encourage organizing all prototypes as children of the PointInstancer
prim that consumes them, and pruning "raw" processing and drawing
traversals when they encounter a PointInstancer prim; this is what the
UsdGeomBBoxCache and UsdImaging engines do.

There \*is a pattern one can deploy for organizing the prototypes
such that they will automatically be skipped by basic UsdPrim::GetChildren()
or UsdPrimRange traversals.  Usd prims each have a
\Usd_PrimSpecifiers "specifier" of "def", "over", or "class".  The
default traversals skip over prims that are "pure overs" or classes.  So
to protect prototypes from all generic traversals and processing, place
them under a prim that is just an "over".  For example,
\```
01 def PointInstancer "Crowd_Mid"
02 {
03     rel prototypes = [ </Crowd_Mid/Prototypes/MaleThin_Business>, </Crowd_Mid/Prototypes/MaleThin_Casual> ]
04
05     over "Prototypes"
06     {
07          def "MaleThin_Business" (
08              references = [@MaleGroupA/usd/MaleGroupA.usd@</MaleGroupA>]
09              variants = {
10                  string modelingVariant = "Thin"
11                  string costumeVariant = "BusinessAttire"
12              }
13          )
14          { ... }
15
16          def "MaleThin_Casual"
17          ...
18     }
19 }
\```*/
#[derive(Clone, Debug)]
pub struct UsdGeomPointInstancer {
    /**Visibility is meant to be the simplest form of "pruning"
visibility that is supported by most DCC apps.  Visibility is
animatable, allowing a sub-tree of geometry to be present for some
segment of a shot, and absent from others; unlike the action of
deactivating geometry prims, invisible geometry is still
available for inspection, for positioning, for defining volumes, etc.*/
    pub visibility: crate::foundation::TfToken,
    /**Purpose is a classification of geometry into categories that
can each be independently included or excluded from traversals of prims
on a stage, such as rendering or bounding-box computation traversals.

See \UsdGeom_ImageablePurpose for more detail about how
\*purpose is computed and used.*/
    pub purpose: crate::foundation::TfToken,
    /**Encodes the sequence of transformation operations in the
order in which they should be pushed onto a transform stack while
visiting a UsdStage's prims in a graph traversal that will effect
the desired positioning for this prim and its descendant prims.

You should rarely, if ever, need to manipulate this attribute directly.
It is managed by the AddXformOp(), SetResetXformStack(), and
SetXformOpOrder(), and consulted by GetOrderedXformOps() and
GetLocalTransformation().*/
    pub xform_op_order: crate::foundation::VtArray<crate::foundation::TfToken>,
    /**Extent is a three dimensional range measuring the geometric
extent of the authored gprim in its own local space (i.e. its own
transform not applied), \*without accounting for any shader-induced
displacement. If __any__ extent value has been authored for a given
Boundable, then it should be authored at every timeSample at which
geometry-affecting properties are authored, to ensure correct
evaluation via ComputeExtent(). If __no__ extent value has been
authored, then ComputeExtent() will call the Boundable's registered
ComputeExtentFunction(), which may be expensive, which is why we
strongly encourage proper authoring of extent.
\See: ComputeExtent()
\See: \UsdGeom_Boundable_Extent.

An authored extent on a prim which has children is expected to include
the extent of all children, as they will be pruned from BBox computation
during traversal.*/
    pub extent: crate::foundation::VtArray<crate::foundation::GfVec3f>,
    /**<b>Required property</b>. Per-instance index into
\*prototypes relationship that identifies what geometry should be
drawn for each instance.  <b>Topology attribute</b> - can be animated,
but at a potential performance impact for streaming.*/
    pub proto_indices: crate::foundation::VtArray<i32>,
    /**Ids are optional; if authored, the ids array should be the same
length as the \*protoIndices array, specifying (at each timeSample if
instance identities are changing) the id of each instance. The
type is signed intentionally, so that clients can encode some
binary state on Id'd instances without adding a separate primvar.
See also \UsdGeomPointInstancer_varyingTopo*/
    pub ids: crate::foundation::VtArray<i64>,
    /**<b>Required property</b>. Per-instance position.  See also
\UsdGeomPointInstancer_transform .*/
    pub positions: crate::foundation::VtArray<crate::foundation::GfVec3f>,
    /**If authored, per-instance orientation of each instance about its
prototype's origin, represented as a unit length quaternion, which
allows us to encode it with sufficient precision in a compact GfQuath.

It is client's responsibility to ensure that authored quaternions are
unit length; the convenience API below for authoring orientations from
rotation matrices will ensure that quaternions are unit length, though
it will not make any attempt to select the "better (for interpolation
with respect to neighboring samples)" of the two possible quaternions
that encode the rotation.

See also \UsdGeomPointInstancer_transform .*/
    pub orientations: crate::foundation::VtArray<[u16; 4]>,
    /**If authored, per-instance orientation of each instance about its
prototype's origin, represented as a unit length quaternion, encoded
as a GfQuatf to support higher precision computations.

It is client's responsibility to ensure that authored quaternions are
unit length; the convenience API below for authoring orientations from
rotation matrices will ensure that quaternions are unit length, though
it will not make any attempt to select the "better (for interpolation
with respect to neighboring samples)" of the two possible quaternions
that encode the rotation. Note that if the earliest time sample (or
default value if there are no time samples) of orientationsf is not empty
orientationsf will be preferred over orientations if both are authored.

See also \UsdGeomPointInstancer_transform .*/
    pub orientationsf: crate::foundation::VtArray<[f32; 4]>,
    /**If authored, per-instance scale to be applied to
each instance, before any rotation is applied.

See also \UsdGeomPointInstancer_transform .*/
    pub scales: crate::foundation::VtArray<crate::foundation::GfVec3f>,
    /**If provided, per-instance 'velocities' will be used to
compute positions between samples for the 'positions' attribute,
rather than interpolating between neighboring 'positions' samples.
Velocities should be considered mandatory if both \*protoIndices
and \*positions are animated.  Velocity is measured in position
units per second, as per most simulation software. To convert to
position units per UsdTimeCode, divide by
UsdStage::GetTimeCodesPerSecond().

See also \UsdGeomPointInstancer_transform,
\UsdGeom_VelocityInterpolation .*/
    pub velocities: crate::foundation::VtArray<crate::foundation::GfVec3f>,
    /**If authored, per-instance 'accelerations' will be used with
velocities to compute positions between samples for the 'positions'
attribute rather than interpolating between neighboring 'positions'
samples. Acceleration is measured in position units per second-squared.
To convert to position units per squared UsdTimeCode, divide by the
square of UsdStage::GetTimeCodesPerSecond().*/
    pub accelerations: crate::foundation::VtArray<crate::foundation::GfVec3f>,
    /**If authored, per-instance angular velocity vector to be used for
interoplating orientations.  Angular velocities should be considered
mandatory if both \*protoIndices and \*orientations are animated.
Angular velocity is measured in <b>degrees</b> per second. To convert
to degrees per UsdTimeCode, divide by
UsdStage::GetTimeCodesPerSecond().

See also \UsdGeomPointInstancer_transform .*/
    pub angular_velocities: crate::foundation::VtArray<crate::foundation::GfVec3f>,
    /**A list of id's to make invisible at the evaluation time.
See \UsdGeomPointInstancer_invisibleIds .*/
    pub invisible_ids: crate::foundation::VtArray<i64>,
}
impl Default for UsdGeomPointInstancer {
    fn default() -> Self {
        Self {
            visibility: crate::foundation::TfToken::new("inherited"),
            purpose: crate::foundation::TfToken::new("default"),
            xform_op_order: Vec::new(),
            extent: Vec::new(),
            proto_indices: Vec::new(),
            ids: Vec::new(),
            positions: Vec::new(),
            orientations: Vec::new(),
            orientationsf: Vec::new(),
            scales: Vec::new(),
            velocities: Vec::new(),
            accelerations: Vec::new(),
            angular_velocities: Vec::new(),
            invisible_ids: Vec::new(),
        }
    }
}
impl crate::schema::generated::UsdSchemaInfo for UsdGeomPointInstancer {
    fn schema_name(&self) -> &'static str {
        "UsdGeomPointInstancer"
    }
    fn attribute_metadata(
        &self,
    ) -> &'static [crate::schema::generated::AttributeMetadata] {
        static META: &[crate::schema::generated::AttributeMetadata] = &[
            crate::schema::generated::AttributeMetadata {
                usd_name: "visibility",
                usd_type: "token",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "purpose",
                usd_type: "token",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "xformOpOrder",
                usd_type: "token[]",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "extent",
                usd_type: "float3[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "protoIndices",
                usd_type: "int[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "ids",
                usd_type: "int64[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "positions",
                usd_type: "point3f[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "orientations",
                usd_type: "quath[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "orientationsf",
                usd_type: "quatf[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "scales",
                usd_type: "float3[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "velocities",
                usd_type: "vector3f[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "accelerations",
                usd_type: "vector3f[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "angularVelocities",
                usd_type: "vector3f[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "invisibleIds",
                usd_type: "int64[]",
                is_uniform: false,
            },
        ];
        META
    }
}
/**Transformable camera.

Describes optical properties of a camera via a common set of attributes
that provide control over the camera's frustum as well as its depth of
field. For stereo, the left and right camera are individual prims tagged
through the \UsdGeomCamera::GetStereoRoleAttr() "stereoRole attribute".

There is a corresponding class GfCamera, which can hold the state of a
camera (at a particular time). \UsdGeomCamera::GetCamera() and
\UsdGeomCamera::SetFromCamera() convert between a USD camera prim and
a GfCamera.

To obtain the camera's location in world space, call the following on a
UsdGeomCamera 'camera':
\```
GfMatrix4d camXform = camera.ComputeLocalToWorldTransform(time);
\```
\Note:
<b>Cameras in USD are always "Y up", regardless of the stage's orientation
(i.e. UsdGeomGetStageUpAxis()).</b> 'camXform' positions the camera in the
world, and the inverse transforms the world such that the camera is at the
origin, looking down the -Z axis, with +Y as the up axis, and +X pointing to
the right. This describes a __right handed coordinate system__.

\## UsdGeom_CameraUnits Units of Measure for Camera Properties

Despite the familiarity of millimeters for specifying some physical
camera properties, UsdGeomCamera opts for greater consistency with all
other UsdGeom schemas, which measure geometric properties in scene units,
as determined by UsdGeomGetStageMetersPerUnit().  We do make a
concession, however, in that lens and filmback properties are measured in
__tenths of a scene unit__ rather than "raw" scene units.  This means
that with the fallback value of .01 for _metersPerUnit_ - i.e. scene unit
of centimeters - then these "tenth of scene unit" properties are
effectively millimeters.

\Note: If one adds a Camera prim to a UsdStage whose scene unit is not
centimeters, the fallback values for filmback properties will be
incorrect (or at the least, unexpected) in an absolute sense; however,
proper imaging through a "default camera" with focusing disabled depends
only on ratios of the other properties, so the camera is still usable.
However, it follows that if even one property is authored in the correct
scene units, then they all must be.

\## UsdGeom_CameraExposure Camera Exposure Model

UsdGeomCamera models exposure by a camera in terms of exposure time, ISO,
f-stop, and exposure compensation, mirroring the controls on a real camera.
These parameters are provided by \UsdGeomCamera::GetExposureTimeAttr(),
\UsdGeomCamera::GetExposureIsoAttr(),
\UsdGeomCamera::GetExposureFStopAttr(),
and \UsdGeomCamera::GetExposureAttr(), respectively.
\UsdGeomCamera::GetExposureResponsivityAttr() provides an additional
scaling factor to model the overall responsivity of the system,
including response of the sensor and loss by the lens.

The calculated scaling factor can be obtained from
\UsdGeomCamera::ComputeLinearExposureScale(). It is computed as:
\```
linearExposureScale = exposureResponsivity *
(exposureTime * (exposureIso/100) * pow(2, exposure))
/ (exposureFStop * exposureFStop)
\```

This scaling factor is combined from two parts: The first, known as the
__imaging ratio__ (in _steradian-second_), converts from incident luminance
at the front of the lens system, in _nit_ (_cd/m^2_), to photometric
exposure at the sensor in _lux-second_. The second, `exposureResponsivity`
(in _inverse lux-second_), converts from photometric exposure at the sensor,
in _lux-second_, to a unitless output signal.

For a thorough treatment of this topic, see
https://github.com/wetadigital/physlight/blob/main/docs/physLight-v1.3-1bdb6ec3-20230805.pdf,
Section 2.2. Note that we are essentially implementing Equation 2.7, but are
choosing C such that it exactly cancels with the factor of pi in the
numerator, replacing it with a responsivity factor that defaults to 1.

Renderers should simply multiply the brightness of the image by the exposure
scale. The default values for the exposure-related attributes combine to
give a scale of 1.0.

\See: \UsdGeom_LinAlgBasics*/
#[derive(Clone, Debug)]
pub struct UsdGeomCamera {
    /**Visibility is meant to be the simplest form of "pruning"
visibility that is supported by most DCC apps.  Visibility is
animatable, allowing a sub-tree of geometry to be present for some
segment of a shot, and absent from others; unlike the action of
deactivating geometry prims, invisible geometry is still
available for inspection, for positioning, for defining volumes, etc.*/
    pub visibility: crate::foundation::TfToken,
    /**Purpose is a classification of geometry into categories that
can each be independently included or excluded from traversals of prims
on a stage, such as rendering or bounding-box computation traversals.

See \UsdGeom_ImageablePurpose for more detail about how
\*purpose is computed and used.*/
    pub purpose: crate::foundation::TfToken,
    /**Encodes the sequence of transformation operations in the
order in which they should be pushed onto a transform stack while
visiting a UsdStage's prims in a graph traversal that will effect
the desired positioning for this prim and its descendant prims.

You should rarely, if ever, need to manipulate this attribute directly.
It is managed by the AddXformOp(), SetResetXformStack(), and
SetXformOpOrder(), and consulted by GetOrderedXformOps() and
GetLocalTransformation().*/
    pub xform_op_order: crate::foundation::VtArray<crate::foundation::TfToken>,
    ///
    pub projection: crate::foundation::TfToken,
    /**Horizontal aperture in tenths of a scene unit; see
\UsdGeom_CameraUnits . Default is the equivalent of
the standard 35mm spherical projector aperture.*/
    pub horizontal_aperture: f32,
    /**Vertical aperture in tenths of a scene unit; see
\UsdGeom_CameraUnits . Default is the equivalent of
the standard 35mm spherical projector aperture.*/
    pub vertical_aperture: f32,
    /**Horizontal aperture offset in the same units as
horizontalAperture. Defaults to 0.*/
    pub horizontal_aperture_offset: f32,
    /**Vertical aperture offset in the same units as
verticalAperture. Defaults to 0.*/
    pub vertical_aperture_offset: f32,
    /**Perspective focal length in tenths of a scene unit; see
\UsdGeom_CameraUnits .*/
    pub focal_length: f32,
    /**Near and far clipping distances in scene units; see
\UsdGeom_CameraUnits .*/
    pub clipping_range: [f32; 2],
    ///Lens aperture. Defaults to 0.0, which turns off depth of field effects.
    pub f_stop: f32,
    /**Distance from the camera to the focus plane in scene units; see
\UsdGeom_CameraUnits .*/
    pub focus_distance: f32,
    /**If different from mono, the camera is intended to be the left
or right camera of a stereo setup.*/
    pub stereo_role: crate::foundation::TfToken,
    /**Frame relative shutter open time in UsdTimeCode units (negative
value indicates that the shutter opens before the current
frame time). Used for motion blur.*/
    pub shutter_open: f64,
    /**Frame relative shutter close time, analogous comments from
shutter:open apply. A value greater or equal to shutter:open
should be authored, otherwise there is no exposure and a
renderer should produce a black image. Used for motion blur.*/
    pub shutter_close: f64,
    /**Exposure compensation, as a log base-2 value.  The default
of 0.0 has no effect.  A value of 1.0 will double the
image-plane intensities in a rendered image; a value of
-1.0 will halve them.*/
    pub exposure: f32,
    /**The speed rating of the sensor or film when calculating exposure.
Higher numbers give a brighter image, lower numbers darker.*/
    pub exposure_iso: f32,
    /**Time in seconds that the sensor is exposed to light when calculating exposure.
Longer exposure times create a brighter image, shorter times darker.
Note that shutter:open and shutter:close model essentially the
same property of a physical camera, but are for specifying the
size of the motion blur streak which is for practical purposes
useful to keep separate.*/
    pub exposure_time: f32,
    /**f-stop of the aperture when calculating exposure. Smaller numbers
create a brighter image, larger numbers darker.
Note that the `fStop` attribute also models the diameter of the camera
aperture, but for specifying depth of field.  For practical
purposes it is useful to keep the exposure and the depth of field
controls separate.*/
    pub exposure_f_stop: f32,
    /**Scalar multiplier representing overall responsivity of the
sensor system to light when calculating exposure. Intended to be
used as a per camera/lens system measured scaling value.*/
    pub exposure_responsivity: f32,
}
impl Default for UsdGeomCamera {
    fn default() -> Self {
        Self {
            visibility: crate::foundation::TfToken::new("inherited"),
            purpose: crate::foundation::TfToken::new("default"),
            xform_op_order: Vec::new(),
            projection: crate::foundation::TfToken::new("perspective"),
            horizontal_aperture: 20.955f32,
            vertical_aperture: 15.2908f32,
            horizontal_aperture_offset: 0f32,
            vertical_aperture_offset: 0f32,
            focal_length: 50f32,
            clipping_range: [0.0f32; 2],
            f_stop: 0f32,
            focus_distance: 0f32,
            stereo_role: crate::foundation::TfToken::new("mono"),
            shutter_open: 0f64,
            shutter_close: 0f64,
            exposure: 0f32,
            exposure_iso: 100f32,
            exposure_time: 1f32,
            exposure_f_stop: 1f32,
            exposure_responsivity: 1f32,
        }
    }
}
impl crate::schema::generated::UsdSchemaInfo for UsdGeomCamera {
    fn schema_name(&self) -> &'static str {
        "UsdGeomCamera"
    }
    fn attribute_metadata(
        &self,
    ) -> &'static [crate::schema::generated::AttributeMetadata] {
        static META: &[crate::schema::generated::AttributeMetadata] = &[
            crate::schema::generated::AttributeMetadata {
                usd_name: "visibility",
                usd_type: "token",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "purpose",
                usd_type: "token",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "xformOpOrder",
                usd_type: "token[]",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "projection",
                usd_type: "token",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "horizontalAperture",
                usd_type: "float",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "verticalAperture",
                usd_type: "float",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "horizontalApertureOffset",
                usd_type: "float",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "verticalApertureOffset",
                usd_type: "float",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "focalLength",
                usd_type: "float",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "clippingRange",
                usd_type: "float2",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "fStop",
                usd_type: "float",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "focusDistance",
                usd_type: "float",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "stereoRole",
                usd_type: "token",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "shutter:open",
                usd_type: "double",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "shutter:close",
                usd_type: "double",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "exposure",
                usd_type: "float",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "exposure:iso",
                usd_type: "float",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "exposure:time",
                usd_type: "float",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "exposure:fStop",
                usd_type: "float",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "exposure:responsivity",
                usd_type: "float",
                is_uniform: false,
            },
        ];
        META
    }
}
/**This schema specifies a cubic hermite interpolated curve batch as
sometimes used for defining guides for animation. While hermite curves can
be useful because they interpolate through their control points, they are
not well supported by high-end renderers for imaging. Therefore, while we
include this schema for interchange, we strongly recommend the use of
UsdGeomBasisCurves as the representation of curves intended to be rendered
(ie. hair or grass). Hermite curves can be converted to a Bezier
representation (though not from Bezier back to Hermite in general).

\## UsdGeomHermiteCurves_Interpolation Point Interpolation

The initial cubic curve segment is defined by the first two points and
first two tangents. Additional segments are defined by additional
point / tangent pairs.  The number of segments for each non-batched hermite
curve would be len(curve.points) - 1.  The total number of segments
for the batched UsdGeomHermiteCurves representation is
len(points) - len(curveVertexCounts).

\## UsdGeomHermiteCurves_Primvars Primvar, Width, and Normal Interpolation

Primvar interpolation is not well specified for this type as it is not
intended as a rendering representation. We suggest that per point
primvars would be linearly interpolated across each segment and should
be tagged as 'varying'.

It is not immediately clear how to specify cubic or 'vertex' interpolation
for this type, as we lack a specification for primvar tangents. This
also means that width and normal interpolation should be restricted to
varying (linear), uniform (per curve element), or constant (per prim).*/
#[derive(Clone, Debug)]
pub struct UsdGeomHermiteCurves {
    /**Visibility is meant to be the simplest form of "pruning"
visibility that is supported by most DCC apps.  Visibility is
animatable, allowing a sub-tree of geometry to be present for some
segment of a shot, and absent from others; unlike the action of
deactivating geometry prims, invisible geometry is still
available for inspection, for positioning, for defining volumes, etc.*/
    pub visibility: crate::foundation::TfToken,
    /**Purpose is a classification of geometry into categories that
can each be independently included or excluded from traversals of prims
on a stage, such as rendering or bounding-box computation traversals.

See \UsdGeom_ImageablePurpose for more detail about how
\*purpose is computed and used.*/
    pub purpose: crate::foundation::TfToken,
    /**Encodes the sequence of transformation operations in the
order in which they should be pushed onto a transform stack while
visiting a UsdStage's prims in a graph traversal that will effect
the desired positioning for this prim and its descendant prims.

You should rarely, if ever, need to manipulate this attribute directly.
It is managed by the AddXformOp(), SetResetXformStack(), and
SetXformOpOrder(), and consulted by GetOrderedXformOps() and
GetLocalTransformation().*/
    pub xform_op_order: crate::foundation::VtArray<crate::foundation::TfToken>,
    /**Extent is a three dimensional range measuring the geometric
extent of the authored gprim in its own local space (i.e. its own
transform not applied), \*without accounting for any shader-induced
displacement. If __any__ extent value has been authored for a given
Boundable, then it should be authored at every timeSample at which
geometry-affecting properties are authored, to ensure correct
evaluation via ComputeExtent(). If __no__ extent value has been
authored, then ComputeExtent() will call the Boundable's registered
ComputeExtentFunction(), which may be expensive, which is why we
strongly encourage proper authoring of extent.
\See: ComputeExtent()
\See: \UsdGeom_Boundable_Extent.

An authored extent on a prim which has children is expected to include
the extent of all children, as they will be pruned from BBox computation
during traversal.*/
    pub extent: crate::foundation::VtArray<crate::foundation::GfVec3f>,
    /**It is useful to have an "official" colorSet that can be used
as a display or modeling color, even in the absence of any specified
shader for a gprim.  DisplayColor serves this role; because it is a
UsdGeomPrimvar, it can also be used as a gprim override for any shader
that consumes a \*displayColor parameter.*/
    pub primvars_display_color: crate::foundation::VtArray<crate::foundation::GfVec3f>,
    /**Companion to \*displayColor that specifies opacity, broken
out as an independent attribute rather than an rgba color, both so that
each can be independently overridden, and because shaders rarely consume
rgba parameters.*/
    pub primvars_display_opacity: crate::foundation::VtArray<f32>,
    /**Although some renderers treat all parametric or polygonal
surfaces as if they were effectively laminae with outward-facing
normals on both sides, some renderers derive significant optimizations
by considering these surfaces to have only a single outward side,
typically determined by control-point winding order and/or
\*orientation.  By doing so they can perform "backface culling" to
avoid drawing the many polygons of most closed surfaces that face away
from the viewer.

However, it is often advantageous to model thin objects such as paper
and cloth as single, open surfaces that must be viewable from both
sides, always.  Setting a gprim's \*doubleSided attribute to
\\c true instructs all renderers to disable optimizations such as
backface culling for the gprim, and attempt (not all renderers are able
to do so, but the USD reference GL renderer always will) to provide
forward-facing normals on each side of the surface for lighting
calculations.*/
    pub double_sided: bool,
    /**Orientation specifies whether the gprim's surface normal
should be computed using the right hand rule, or the left hand rule.
Please see \UsdGeom_WindingOrder for a deeper explanation and
generalization of orientation to composed scenes with transformation
hierarchies.*/
    pub orientation: crate::foundation::TfToken,
    /**The primary geometry attribute for all PointBased
primitives, describes points in (local) space.*/
    pub points: crate::foundation::VtArray<crate::foundation::GfVec3f>,
    /**If provided, 'velocities' should be used by renderers to

compute positions between samples for the 'points' attribute, rather
than interpolating between neighboring 'points' samples.  This is the
only reasonable means of computing motion blur for topologically
varying PointBased primitives.  It follows that the length of each
'velocities' sample must match the length of the corresponding
'points' sample.  Velocity is measured in position units per second,
as per most simulation software. To convert to position units per
UsdTimeCode, divide by UsdStage::GetTimeCodesPerSecond().

See also \UsdGeom_VelocityInterpolation .*/
    pub velocities: crate::foundation::VtArray<crate::foundation::GfVec3f>,
    /**If provided, 'accelerations' should be used with
velocities to compute positions between samples for the 'points'
attribute rather than interpolating between neighboring 'points'
samples. Acceleration is measured in position units per second-squared.
To convert to position units per squared UsdTimeCode, divide by the
square of UsdStage::GetTimeCodesPerSecond().*/
    pub accelerations: crate::foundation::VtArray<crate::foundation::GfVec3f>,
    /**Provide an object-space orientation for individual points,
which, depending on subclass, may define a surface, curve, or free
points.  Note that 'normals' should not be authored on any Mesh that
is subdivided, since the subdivision algorithm will define its own
normals. 'normals' is not a generic primvar, but the number of elements
in this attribute will be determined by its 'interpolation'.  See
\SetNormalsInterpolation() . If 'normals' and 'primvars:normals'
are both specified, the latter has precedence.*/
    pub normals: crate::foundation::VtArray<crate::foundation::GfVec3f>,
    /**Curves-derived primitives can represent multiple distinct,
potentially disconnected curves.  The length of 'curveVertexCounts'
gives the number of such curves, and each element describes the
number of vertices in the corresponding curve*/
    pub curve_vertex_counts: crate::foundation::VtArray<i32>,
    /**Provides width specification for the curves, whose application
will depend on whether the curve is oriented (normals are defined for
it), in which case widths are "ribbon width", or unoriented, in which
case widths are cylinder width.  'widths' is not a generic Primvar,
but the number of elements in this attribute will be determined by
its 'interpolation'.  See \SetWidthsInterpolation() .  If 'widths'
and 'primvars:widths' are both specified, the latter has precedence.*/
    pub widths: crate::foundation::VtArray<f32>,
    /**Defines the outgoing trajectory tangent for each point.
Tangents should be the same size as the points attribute.*/
    pub tangents: crate::foundation::VtArray<crate::foundation::GfVec3f>,
}
impl Default for UsdGeomHermiteCurves {
    fn default() -> Self {
        Self {
            visibility: crate::foundation::TfToken::new("inherited"),
            purpose: crate::foundation::TfToken::new("default"),
            xform_op_order: Vec::new(),
            extent: Vec::new(),
            primvars_display_color: Vec::new(),
            primvars_display_opacity: Vec::new(),
            double_sided: false,
            orientation: crate::foundation::TfToken::new("rightHanded"),
            points: Vec::new(),
            velocities: Vec::new(),
            accelerations: Vec::new(),
            normals: Vec::new(),
            curve_vertex_counts: Vec::new(),
            widths: Vec::new(),
            tangents: Vec::new(),
        }
    }
}
impl crate::schema::generated::UsdSchemaInfo for UsdGeomHermiteCurves {
    fn schema_name(&self) -> &'static str {
        "UsdGeomHermiteCurves"
    }
    fn attribute_metadata(
        &self,
    ) -> &'static [crate::schema::generated::AttributeMetadata] {
        static META: &[crate::schema::generated::AttributeMetadata] = &[
            crate::schema::generated::AttributeMetadata {
                usd_name: "visibility",
                usd_type: "token",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "purpose",
                usd_type: "token",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "xformOpOrder",
                usd_type: "token[]",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "extent",
                usd_type: "float3[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "primvars:displayColor",
                usd_type: "color3f[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "primvars:displayOpacity",
                usd_type: "float[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "doubleSided",
                usd_type: "bool",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "orientation",
                usd_type: "token",
                is_uniform: true,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "points",
                usd_type: "point3f[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "velocities",
                usd_type: "vector3f[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "accelerations",
                usd_type: "vector3f[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "normals",
                usd_type: "normal3f[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "curveVertexCounts",
                usd_type: "int[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "widths",
                usd_type: "float[]",
                is_uniform: false,
            },
            crate::schema::generated::AttributeMetadata {
                usd_name: "tangents",
                usd_type: "vector3f[]",
                is_uniform: false,
            },
        ];
        META
    }
}
/// Scope prim — no geometry, just a grouping node.
#[derive(Clone, Debug, Default)]
pub struct UsdGeomScope;
impl UsdSchemaInfo for UsdGeomScope {
    fn schema_name(&self) -> &'static str {
        "Scope"
    }
    fn attribute_metadata(&self) -> &'static [AttributeMetadata] {
        &[]
    }
}
/// Discriminated union of all concrete USD geometry schema types.
#[derive(Clone, Debug)]
pub enum SchemaData {
    Scope(UsdGeomScope),
    Xform(UsdGeomXform),
    Cube(UsdGeomCube),
    Sphere(UsdGeomSphere),
    Cylinder(UsdGeomCylinder),
    Capsule(UsdGeomCapsule),
    Cone(UsdGeomCone),
    Cylinder_1(UsdGeomCylinder_1),
    Capsule_1(UsdGeomCapsule_1),
    Plane(UsdGeomPlane),
    Mesh(UsdGeomMesh),
    TetMesh(UsdGeomTetMesh),
    GeomSubset(UsdGeomGeomSubset),
    NurbsPatch(UsdGeomNurbsPatch),
    BasisCurves(UsdGeomBasisCurves),
    NurbsCurves(UsdGeomNurbsCurves),
    Points(UsdGeomPoints),
    PointInstancer(UsdGeomPointInstancer),
    Camera(UsdGeomCamera),
    HermiteCurves(UsdGeomHermiteCurves),
}
impl Default for SchemaData {
    fn default() -> Self {
        SchemaData::Scope(UsdGeomScope)
    }
}
impl SchemaData {
    pub fn schema_name(&self) -> &'static str {
        match self {
            SchemaData::Scope(_) => "Scope",
            SchemaData::Xform(_) => "UsdGeomXform",
            SchemaData::Cube(_) => "UsdGeomCube",
            SchemaData::Sphere(_) => "UsdGeomSphere",
            SchemaData::Cylinder(_) => "UsdGeomCylinder",
            SchemaData::Capsule(_) => "UsdGeomCapsule",
            SchemaData::Cone(_) => "UsdGeomCone",
            SchemaData::Cylinder_1(_) => "UsdGeomCylinder_1",
            SchemaData::Capsule_1(_) => "UsdGeomCapsule_1",
            SchemaData::Plane(_) => "UsdGeomPlane",
            SchemaData::Mesh(_) => "UsdGeomMesh",
            SchemaData::TetMesh(_) => "UsdGeomTetMesh",
            SchemaData::GeomSubset(_) => "UsdGeomGeomSubset",
            SchemaData::NurbsPatch(_) => "UsdGeomNurbsPatch",
            SchemaData::BasisCurves(_) => "UsdGeomBasisCurves",
            SchemaData::NurbsCurves(_) => "UsdGeomNurbsCurves",
            SchemaData::Points(_) => "UsdGeomPoints",
            SchemaData::PointInstancer(_) => "UsdGeomPointInstancer",
            SchemaData::Camera(_) => "UsdGeomCamera",
            SchemaData::HermiteCurves(_) => "UsdGeomHermiteCurves",
        }
    }
}
impl From<UsdGeomScope> for SchemaData {
    fn from(v: UsdGeomScope) -> Self {
        SchemaData::Scope(v)
    }
}
impl From<UsdGeomXform> for SchemaData {
    fn from(v: UsdGeomXform) -> Self {
        SchemaData::Xform(v)
    }
}
impl From<UsdGeomCube> for SchemaData {
    fn from(v: UsdGeomCube) -> Self {
        SchemaData::Cube(v)
    }
}
impl From<UsdGeomSphere> for SchemaData {
    fn from(v: UsdGeomSphere) -> Self {
        SchemaData::Sphere(v)
    }
}
impl From<UsdGeomCylinder> for SchemaData {
    fn from(v: UsdGeomCylinder) -> Self {
        SchemaData::Cylinder(v)
    }
}
impl From<UsdGeomCapsule> for SchemaData {
    fn from(v: UsdGeomCapsule) -> Self {
        SchemaData::Capsule(v)
    }
}
impl From<UsdGeomCone> for SchemaData {
    fn from(v: UsdGeomCone) -> Self {
        SchemaData::Cone(v)
    }
}
impl From<UsdGeomCylinder_1> for SchemaData {
    fn from(v: UsdGeomCylinder_1) -> Self {
        SchemaData::Cylinder_1(v)
    }
}
impl From<UsdGeomCapsule_1> for SchemaData {
    fn from(v: UsdGeomCapsule_1) -> Self {
        SchemaData::Capsule_1(v)
    }
}
impl From<UsdGeomPlane> for SchemaData {
    fn from(v: UsdGeomPlane) -> Self {
        SchemaData::Plane(v)
    }
}
impl From<UsdGeomMesh> for SchemaData {
    fn from(v: UsdGeomMesh) -> Self {
        SchemaData::Mesh(v)
    }
}
impl From<UsdGeomTetMesh> for SchemaData {
    fn from(v: UsdGeomTetMesh) -> Self {
        SchemaData::TetMesh(v)
    }
}
impl From<UsdGeomGeomSubset> for SchemaData {
    fn from(v: UsdGeomGeomSubset) -> Self {
        SchemaData::GeomSubset(v)
    }
}
impl From<UsdGeomNurbsPatch> for SchemaData {
    fn from(v: UsdGeomNurbsPatch) -> Self {
        SchemaData::NurbsPatch(v)
    }
}
impl From<UsdGeomBasisCurves> for SchemaData {
    fn from(v: UsdGeomBasisCurves) -> Self {
        SchemaData::BasisCurves(v)
    }
}
impl From<UsdGeomNurbsCurves> for SchemaData {
    fn from(v: UsdGeomNurbsCurves) -> Self {
        SchemaData::NurbsCurves(v)
    }
}
impl From<UsdGeomPoints> for SchemaData {
    fn from(v: UsdGeomPoints) -> Self {
        SchemaData::Points(v)
    }
}
impl From<UsdGeomPointInstancer> for SchemaData {
    fn from(v: UsdGeomPointInstancer) -> Self {
        SchemaData::PointInstancer(v)
    }
}
impl From<UsdGeomCamera> for SchemaData {
    fn from(v: UsdGeomCamera) -> Self {
        SchemaData::Camera(v)
    }
}
impl From<UsdGeomHermiteCurves> for SchemaData {
    fn from(v: UsdGeomHermiteCurves) -> Self {
        SchemaData::HermiteCurves(v)
    }
}
impl UsdSchema for UsdGeomScope {
    fn from_schema_data(d: &SchemaData) -> Option<&Self> {
        match d {
            SchemaData::Scope(ref v) => Some(v),
            _ => None,
        }
    }
    fn from_schema_data_mut(d: &mut SchemaData) -> Option<&mut Self> {
        match d {
            SchemaData::Scope(ref mut v) => Some(v),
            _ => None,
        }
    }
}
impl UsdSchema for UsdGeomXform {
    fn from_schema_data(d: &SchemaData) -> Option<&Self> {
        match d {
            SchemaData::Xform(ref v) => Some(v),
            _ => None,
        }
    }
    fn from_schema_data_mut(d: &mut SchemaData) -> Option<&mut Self> {
        match d {
            SchemaData::Xform(ref mut v) => Some(v),
            _ => None,
        }
    }
}
impl UsdSchema for UsdGeomCube {
    fn from_schema_data(d: &SchemaData) -> Option<&Self> {
        match d {
            SchemaData::Cube(ref v) => Some(v),
            _ => None,
        }
    }
    fn from_schema_data_mut(d: &mut SchemaData) -> Option<&mut Self> {
        match d {
            SchemaData::Cube(ref mut v) => Some(v),
            _ => None,
        }
    }
}
impl UsdSchema for UsdGeomSphere {
    fn from_schema_data(d: &SchemaData) -> Option<&Self> {
        match d {
            SchemaData::Sphere(ref v) => Some(v),
            _ => None,
        }
    }
    fn from_schema_data_mut(d: &mut SchemaData) -> Option<&mut Self> {
        match d {
            SchemaData::Sphere(ref mut v) => Some(v),
            _ => None,
        }
    }
}
impl UsdSchema for UsdGeomCylinder {
    fn from_schema_data(d: &SchemaData) -> Option<&Self> {
        match d {
            SchemaData::Cylinder(ref v) => Some(v),
            _ => None,
        }
    }
    fn from_schema_data_mut(d: &mut SchemaData) -> Option<&mut Self> {
        match d {
            SchemaData::Cylinder(ref mut v) => Some(v),
            _ => None,
        }
    }
}
impl UsdSchema for UsdGeomCapsule {
    fn from_schema_data(d: &SchemaData) -> Option<&Self> {
        match d {
            SchemaData::Capsule(ref v) => Some(v),
            _ => None,
        }
    }
    fn from_schema_data_mut(d: &mut SchemaData) -> Option<&mut Self> {
        match d {
            SchemaData::Capsule(ref mut v) => Some(v),
            _ => None,
        }
    }
}
impl UsdSchema for UsdGeomCone {
    fn from_schema_data(d: &SchemaData) -> Option<&Self> {
        match d {
            SchemaData::Cone(ref v) => Some(v),
            _ => None,
        }
    }
    fn from_schema_data_mut(d: &mut SchemaData) -> Option<&mut Self> {
        match d {
            SchemaData::Cone(ref mut v) => Some(v),
            _ => None,
        }
    }
}
impl UsdSchema for UsdGeomCylinder_1 {
    fn from_schema_data(d: &SchemaData) -> Option<&Self> {
        match d {
            SchemaData::Cylinder_1(ref v) => Some(v),
            _ => None,
        }
    }
    fn from_schema_data_mut(d: &mut SchemaData) -> Option<&mut Self> {
        match d {
            SchemaData::Cylinder_1(ref mut v) => Some(v),
            _ => None,
        }
    }
}
impl UsdSchema for UsdGeomCapsule_1 {
    fn from_schema_data(d: &SchemaData) -> Option<&Self> {
        match d {
            SchemaData::Capsule_1(ref v) => Some(v),
            _ => None,
        }
    }
    fn from_schema_data_mut(d: &mut SchemaData) -> Option<&mut Self> {
        match d {
            SchemaData::Capsule_1(ref mut v) => Some(v),
            _ => None,
        }
    }
}
impl UsdSchema for UsdGeomPlane {
    fn from_schema_data(d: &SchemaData) -> Option<&Self> {
        match d {
            SchemaData::Plane(ref v) => Some(v),
            _ => None,
        }
    }
    fn from_schema_data_mut(d: &mut SchemaData) -> Option<&mut Self> {
        match d {
            SchemaData::Plane(ref mut v) => Some(v),
            _ => None,
        }
    }
}
impl UsdSchema for UsdGeomMesh {
    fn from_schema_data(d: &SchemaData) -> Option<&Self> {
        match d {
            SchemaData::Mesh(ref v) => Some(v),
            _ => None,
        }
    }
    fn from_schema_data_mut(d: &mut SchemaData) -> Option<&mut Self> {
        match d {
            SchemaData::Mesh(ref mut v) => Some(v),
            _ => None,
        }
    }
}
impl UsdSchema for UsdGeomTetMesh {
    fn from_schema_data(d: &SchemaData) -> Option<&Self> {
        match d {
            SchemaData::TetMesh(ref v) => Some(v),
            _ => None,
        }
    }
    fn from_schema_data_mut(d: &mut SchemaData) -> Option<&mut Self> {
        match d {
            SchemaData::TetMesh(ref mut v) => Some(v),
            _ => None,
        }
    }
}
impl UsdSchema for UsdGeomGeomSubset {
    fn from_schema_data(d: &SchemaData) -> Option<&Self> {
        match d {
            SchemaData::GeomSubset(ref v) => Some(v),
            _ => None,
        }
    }
    fn from_schema_data_mut(d: &mut SchemaData) -> Option<&mut Self> {
        match d {
            SchemaData::GeomSubset(ref mut v) => Some(v),
            _ => None,
        }
    }
}
impl UsdSchema for UsdGeomNurbsPatch {
    fn from_schema_data(d: &SchemaData) -> Option<&Self> {
        match d {
            SchemaData::NurbsPatch(ref v) => Some(v),
            _ => None,
        }
    }
    fn from_schema_data_mut(d: &mut SchemaData) -> Option<&mut Self> {
        match d {
            SchemaData::NurbsPatch(ref mut v) => Some(v),
            _ => None,
        }
    }
}
impl UsdSchema for UsdGeomBasisCurves {
    fn from_schema_data(d: &SchemaData) -> Option<&Self> {
        match d {
            SchemaData::BasisCurves(ref v) => Some(v),
            _ => None,
        }
    }
    fn from_schema_data_mut(d: &mut SchemaData) -> Option<&mut Self> {
        match d {
            SchemaData::BasisCurves(ref mut v) => Some(v),
            _ => None,
        }
    }
}
impl UsdSchema for UsdGeomNurbsCurves {
    fn from_schema_data(d: &SchemaData) -> Option<&Self> {
        match d {
            SchemaData::NurbsCurves(ref v) => Some(v),
            _ => None,
        }
    }
    fn from_schema_data_mut(d: &mut SchemaData) -> Option<&mut Self> {
        match d {
            SchemaData::NurbsCurves(ref mut v) => Some(v),
            _ => None,
        }
    }
}
impl UsdSchema for UsdGeomPoints {
    fn from_schema_data(d: &SchemaData) -> Option<&Self> {
        match d {
            SchemaData::Points(ref v) => Some(v),
            _ => None,
        }
    }
    fn from_schema_data_mut(d: &mut SchemaData) -> Option<&mut Self> {
        match d {
            SchemaData::Points(ref mut v) => Some(v),
            _ => None,
        }
    }
}
impl UsdSchema for UsdGeomPointInstancer {
    fn from_schema_data(d: &SchemaData) -> Option<&Self> {
        match d {
            SchemaData::PointInstancer(ref v) => Some(v),
            _ => None,
        }
    }
    fn from_schema_data_mut(d: &mut SchemaData) -> Option<&mut Self> {
        match d {
            SchemaData::PointInstancer(ref mut v) => Some(v),
            _ => None,
        }
    }
}
impl UsdSchema for UsdGeomCamera {
    fn from_schema_data(d: &SchemaData) -> Option<&Self> {
        match d {
            SchemaData::Camera(ref v) => Some(v),
            _ => None,
        }
    }
    fn from_schema_data_mut(d: &mut SchemaData) -> Option<&mut Self> {
        match d {
            SchemaData::Camera(ref mut v) => Some(v),
            _ => None,
        }
    }
}
impl UsdSchema for UsdGeomHermiteCurves {
    fn from_schema_data(d: &SchemaData) -> Option<&Self> {
        match d {
            SchemaData::HermiteCurves(ref v) => Some(v),
            _ => None,
        }
    }
    fn from_schema_data_mut(d: &mut SchemaData) -> Option<&mut Self> {
        match d {
            SchemaData::HermiteCurves(ref mut v) => Some(v),
            _ => None,
        }
    }
}
/// Token constants from allowed values in the USD schema.
pub mod tokens {
    pub const INHERITED: &str = "inherited";
    pub const INVISIBLE: &str = "invisible";
    pub const DEFAULT: &str = "default";
    pub const RENDER: &str = "render";
    pub const PROXY: &str = "proxy";
    pub const GUIDE: &str = "guide";
    pub const VISIBLE: &str = "visible";
    pub const RIGHT_HANDED: &str = "rightHanded";
    pub const LEFT_HANDED: &str = "leftHanded";
    pub const X: &str = "X";
    pub const Y: &str = "Y";
    pub const Z: &str = "Z";
    pub const CATMULL_CLARK: &str = "catmullClark";
    pub const LOOP: &str = "loop";
    pub const BILINEAR: &str = "bilinear";
    pub const NONE: &str = "none";
    pub const EDGE_ONLY: &str = "edgeOnly";
    pub const EDGE_AND_CORNER: &str = "edgeAndCorner";
    pub const CORNERS_ONLY: &str = "cornersOnly";
    pub const CORNERS_PLUS1: &str = "cornersPlus1";
    pub const CORNERS_PLUS2: &str = "cornersPlus2";
    pub const BOUNDARIES: &str = "boundaries";
    pub const ALL: &str = "all";
    pub const SMOOTH: &str = "smooth";
    pub const FACE: &str = "face";
    pub const POINT: &str = "point";
    pub const EDGE: &str = "edge";
    pub const SEGMENT: &str = "segment";
    pub const TETRAHEDRON: &str = "tetrahedron";
    pub const OPEN: &str = "open";
    pub const CLOSED: &str = "closed";
    pub const PERIODIC: &str = "periodic";
    pub const LINEAR: &str = "linear";
    pub const CUBIC: &str = "cubic";
    pub const BEZIER: &str = "bezier";
    pub const BSPLINE: &str = "bspline";
    pub const CATMULL_ROM: &str = "catmullRom";
    pub const NONPERIODIC: &str = "nonperiodic";
    pub const PINNED: &str = "pinned";
    pub const PERSPECTIVE: &str = "perspective";
    pub const ORTHOGRAPHIC: &str = "orthographic";
    pub const MONO: &str = "mono";
    pub const LEFT: &str = "left";
    pub const RIGHT: &str = "right";
    pub const ORIGIN: &str = "origin";
    pub const BOUNDS: &str = "bounds";
    pub const CARDS: &str = "cards";
    pub const CROSS: &str = "cross";
    pub const BOX: &str = "box";
    pub const FROM_TEXTURE: &str = "fromTexture";
}
