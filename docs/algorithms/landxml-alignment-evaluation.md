# LandXML Alignment Evaluation

Implementation reference for horizontal and vertical alignment evaluation in
`crates/kernel/src/landxml/`.

## Overview

LandXML files describe road and rail alignments as a composition of horizontal and
vertical geometric elements. rusted-geom parses these into typed segment sequences
and evaluates them at arbitrary stations to produce 2D plan positions, 3D profiles,
tangent vectors, and vertical grades.

## Horizontal Alignment

Source: `crates/kernel/src/landxml/horizontal.rs`

A horizontal alignment is a sequence of segments, each defined by a start station
and length. Three segment types are supported:

### Line Segments

Straight segments defined by start point, direction, and length. Evaluation is
simple linear interpolation along the direction vector.

### Circular Arcs

Defined by start point, start direction, radius, and length. The center of
curvature is computed from the start point and perpendicular direction. Evaluation
uses angular parametrization around the arc center:

```
theta = distance / radius
point = center + radius * [cos(start_angle + theta), sin(start_angle + theta)]
```

### Spirals

Source: `crates/kernel/src/landxml/spiral.rs`

Clothoid (Euler) spirals where curvature varies linearly with distance. The
implementation uses Fresnel integral approximation. Supported spiral types:

- Clothoid (default)
- Cubic parabola approximation

Evaluation uses numerical integration of the curvature function along the arc
length, with adaptive step sizing for accuracy on tight spirals.

## Vertical Alignment

Source: `crates/kernel/src/landxml/vertical.rs`

Vertical profiles are defined as a series of Vertical Points of Intersection (VPIs)
connected by vertical curve elements:

### Tangent Segments

Straight-grade segments between VPIs. The elevation at any station is:

```
elevation = vpi_elevation + grade * (station - vpi_station)
```

### Parabolic Vertical Curves

Symmetric or asymmetric parabolic curves at VPIs. The elevation is computed using
the standard parabolic equation with entry/exit grades and curve length.

### Circular Vertical Curves

Less common; uses circular arc geometry in the station-elevation plane.

## 3D Evaluation

Source: `crates/kernel/src/landxml/alignment3d.rs`

`evaluate_alignment_3d(alignment, profile, station)` combines horizontal and vertical:

1. Evaluate the horizontal alignment at the given station to get `(x, y)` and
   the horizontal tangent direction.
2. Evaluate the vertical profile at the same station to get `z` (elevation) and
   the vertical grade.
3. Construct the 3D tangent vector from the horizontal direction and grade.
4. Return `AlignmentSample3d { point, tangent, grade }`.

## Station Equations

Source: `crates/kernel/src/landxml/station.rs`

Station equations handle chainage discontinuities (e.g., station equations at
intersections). The kernel maps display stations to internal stations transparently.

## Coordinate Systems

LandXML files may use different point orderings (NEZ, ENZ, EZN). The parser accepts
a `LandXmlPointOrder` option and reorders coordinates accordingly during parsing, so
downstream evaluation always operates in a consistent (E, N, Z) coordinate frame.

## Units

When `NormalizeToMeters` is selected (default), all coordinates and lengths are
converted to meters during parsing. The source unit string is preserved in
`landxml_linear_unit()` for display purposes.
