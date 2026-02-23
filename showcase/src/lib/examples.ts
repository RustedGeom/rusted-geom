import type { ExampleKey } from "./viewer-types";

export const EXAMPLE_OPTIONS: Record<string, ExampleKey> = {
  "NURBS (fit points)": "nurbs",
  "Line (3D skew)": "line",
  "Polyline (spatial)": "polyline",
  "Polycurve (mixed)": "polycurve",
  "Arc (tilted)": "arc",
  "Circle (tilted)": "circle",
  "Intersection (curve-curve)": "intersectCurveCurve",
  "Intersection (curve-plane)": "intersectCurvePlane",
  "Mesh (large torus)": "meshLarge",
  "Mesh (transform chain)": "meshTransform",
  "Mesh (mesh-mesh intersection)": "meshIntersectMeshMesh",
  "Mesh (mesh-plane section)": "meshIntersectMeshPlane",
  "Mesh (CSG difference: box - torus)": "meshBoolean",
  "Surface (large untrimmed)": "surfaceLarge",
  "Surface (transform chain)": "surfaceTransform",
  "Surface (UV evaluate D0/D1/D2)": "surfaceUvEval",
  "Surface (surface-surface intersection)": "surfaceIntersectSurface",
  "Surface (surface-plane intersection)": "surfaceIntersectPlane",
  "Surface (surface-curve intersection)": "surfaceIntersectCurve",
  "Trim (edit workflow)": "trimEditWorkflow",
  "Trim (validation failures)": "trimValidationFailures",
};

export const EXAMPLE_SUMMARIES: Record<ExampleKey, string> = {
  nurbs: "Interpolates a smooth NURBS curve from fit points.",
  line: "Shows a single 3D line segment sampled by the kernel.",
  polyline: "Builds a piecewise linear spatial polyline.",
  polycurve: "Combines line and arc segments into one chained polycurve.",
  arc: "Creates a planar arc in a tilted frame.",
  circle: "Creates a full circle in a tilted frame.",
  intersectCurveCurve: "Finds intersection points between two curves.",
  intersectCurvePlane: "Finds where a 3D curve crosses an oblique plane.",
  meshLarge: "Displays a dense torus mesh to inspect mesh rendering scale.",
  meshTransform: "Applies translate/rotate/scale transforms and rebuilds in kernel.",
  meshIntersectMeshMesh: "Computes raw segment pairs from mesh-mesh intersection.",
  meshIntersectMeshPlane: "Cuts a mesh with a plane and shows section segments.",
  meshBoolean:
    "Select A or B, move it with the gizmo, and recompute the CSG difference (A - B) on every drag commit.",
  surfaceLarge: "Builds a high-density untrimmed NURBS surface and tessellates it in-kernel.",
  surfaceTransform: "Applies translation, rotation, and scaling to a surface in-kernel.",
  surfaceUvEval:
    "Evaluates a non-trivial rational NURBS surface at normalized UV points and reports D0/D1 plus D2 when available.",
  surfaceIntersectSurface: "Computes untrimmed surface-surface intersection branches in-kernel.",
  surfaceIntersectPlane: "Computes untrimmed surface-plane section branches in-kernel.",
  surfaceIntersectCurve: "Computes surface-curve intersections with UV and curve-parameter traces.",
  trimEditWorkflow: "Demonstrates trim loop edit operations and retessellation in-kernel.",
  trimValidationFailures:
    "Creates an intentionally invalid trim topology and reports validation/heal behavior.",
};

export interface ExampleCategoryItem {
  key: ExampleKey;
  label: string;
}

export interface ExampleCategory {
  label: string;
  key: string;
  items: ExampleCategoryItem[];
}

export const EXAMPLE_CATEGORIES: ExampleCategory[] = [
  {
    label: "Curves",
    key: "curves",
    items: [
      { key: "nurbs", label: "NURBS (fit points)" },
      { key: "line", label: "Line (3D skew)" },
      { key: "polyline", label: "Polyline (spatial)" },
      { key: "polycurve", label: "Polycurve (mixed)" },
      { key: "arc", label: "Arc (tilted)" },
      { key: "circle", label: "Circle (tilted)" },
    ],
  },
  {
    label: "Intersections",
    key: "intersections",
    items: [
      { key: "intersectCurveCurve", label: "Curve × Curve" },
      { key: "intersectCurvePlane", label: "Curve × Plane" },
    ],
  },
  {
    label: "Meshes",
    key: "meshes",
    items: [
      { key: "meshLarge", label: "Large Torus" },
      { key: "meshTransform", label: "Transform Chain" },
      { key: "meshIntersectMeshMesh", label: "Mesh × Mesh" },
      { key: "meshIntersectMeshPlane", label: "Mesh × Plane" },
      { key: "meshBoolean", label: "CSG Difference (A − B)" },
    ],
  },
  {
    label: "Surfaces",
    key: "surfaces",
    items: [
      { key: "surfaceLarge", label: "Large Untrimmed" },
      { key: "surfaceTransform", label: "Transform Chain" },
      { key: "surfaceUvEval", label: "UV Evaluate D0/D1/D2" },
      { key: "surfaceIntersectSurface", label: "Surface × Surface" },
      { key: "surfaceIntersectPlane", label: "Surface × Plane" },
      { key: "surfaceIntersectCurve", label: "Surface × Curve" },
    ],
  },
  {
    label: "Trim",
    key: "trim",
    items: [
      { key: "trimEditWorkflow", label: "Edit Workflow" },
      { key: "trimValidationFailures", label: "Validation Failures" },
    ],
  },
];

export function parseExampleSelection(value: unknown): ExampleKey | null {
  const raw = String(value);
  const validKeys: ExampleKey[] = [
    "nurbs", "line", "polyline", "polycurve", "arc", "circle",
    "intersectCurveCurve", "intersectCurvePlane",
    "meshLarge", "meshTransform", "meshIntersectMeshMesh", "meshIntersectMeshPlane", "meshBoolean",
    "surfaceLarge", "surfaceTransform", "surfaceUvEval",
    "surfaceIntersectSurface", "surfaceIntersectPlane", "surfaceIntersectCurve",
    "trimEditWorkflow", "trimValidationFailures",
  ];
  if (validKeys.includes(raw as ExampleKey)) {
    return raw as ExampleKey;
  }
  const mapped = EXAMPLE_OPTIONS[raw];
  return mapped ?? null;
}

export function getCategoryForExample(key: ExampleKey): string {
  for (const cat of EXAMPLE_CATEGORIES) {
    if (cat.items.some((item) => item.key === key)) {
      return cat.label;
    }
  }
  return "";
}

export function getLabelForExample(key: ExampleKey): string {
  for (const cat of EXAMPLE_CATEGORIES) {
    const item = cat.items.find((i) => i.key === key);
    if (item) return item.label;
  }
  return key;
}
