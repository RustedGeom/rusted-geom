import type { RgmPoint3, RgmVec3 } from "@rustedgeom/kernel";

export function magnitude(vector: RgmVec3): number {
  return Math.sqrt(vector.x * vector.x + vector.y * vector.y + vector.z * vector.z);
}

export function dist(a: RgmPoint3, b: RgmPoint3): number {
  const dx = a.x - b.x;
  const dy = a.y - b.y;
  const dz = a.z - b.z;
  return Math.sqrt(dx * dx + dy * dy + dz * dz);
}

export function addScaled(point: RgmPoint3, vector: RgmVec3, scale: number): RgmPoint3 {
  return {
    x: point.x + vector.x * scale,
    y: point.y + vector.y * scale,
    z: point.z + vector.z * scale,
  };
}

export function scaleVec(vector: RgmVec3, scale: number): RgmVec3 {
  return {
    x: vector.x * scale,
    y: vector.y * scale,
    z: vector.z * scale,
  };
}

export function crossVec(a: RgmVec3, b: RgmVec3): RgmVec3 {
  return {
    x: a.y * b.z - a.z * b.y,
    y: a.z * b.x - a.x * b.z,
    z: a.x * b.y - a.y * b.x,
  };
}

export function normalizedVec(vector: RgmVec3): RgmVec3 | null {
  const len = magnitude(vector);
  if (!Number.isFinite(len) || len <= 1e-12) {
    return null;
  }
  return {
    x: vector.x / len,
    y: vector.y / len,
    z: vector.z / len,
  };
}

export function formatPoint(point: RgmPoint3): string {
  return `(${point.x.toFixed(4)}, ${point.y.toFixed(4)}, ${point.z.toFixed(4)})`;
}

export function formatVec(vector: RgmVec3): string {
  return `(${vector.x.toFixed(4)}, ${vector.y.toFixed(4)}, ${vector.z.toFixed(4)})`;
}
