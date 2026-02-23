import type { SurfaceProbeUiState } from "@/lib/viewer-types";

function formatPoint(point: { x: number; y: number; z: number }): string {
  return `(${point.x.toFixed(4)}, ${point.y.toFixed(4)}, ${point.z.toFixed(4)})`;
}

function formatVec(v: { x: number; y: number; z: number }): string {
  return `(${v.x.toFixed(4)}, ${v.y.toFixed(4)}, ${v.z.toFixed(4)})`;
}

function magnitude(v: { x: number; y: number; z: number }): number {
  return Math.sqrt(v.x * v.x + v.y * v.y + v.z * v.z);
}

interface SurfaceProbeSectionProps {
  surfaceProbeUiState: SurfaceProbeUiState;
  onUpdateSurfaceProbe: (u: number, v: number, commit: boolean) => void;
}

export function SurfaceProbeSection({
  surfaceProbeUiState,
  onUpdateSurfaceProbe,
}: SurfaceProbeSectionProps) {
  return (
    <section className="inspector-section" aria-label="Surface probe controls">
      <h2>Surface Probe</h2>
      <label className="inspector-field">
        <span>u</span>
        <input
          type="range"
          min={0}
          max={1}
          step={0.001}
          value={surfaceProbeUiState.u}
          onChange={(event) =>
            onUpdateSurfaceProbe(Number(event.currentTarget.value), surfaceProbeUiState.v, false)
          }
          onPointerUp={(event) =>
            onUpdateSurfaceProbe(Number(event.currentTarget.value), surfaceProbeUiState.v, true)
          }
          onTouchEnd={(event) =>
            onUpdateSurfaceProbe(Number(event.currentTarget.value), surfaceProbeUiState.v, true)
          }
        />
      </label>
      <label className="inspector-field">
        <span>v</span>
        <input
          type="range"
          min={0}
          max={1}
          step={0.001}
          value={surfaceProbeUiState.v}
          onChange={(event) =>
            onUpdateSurfaceProbe(surfaceProbeUiState.u, Number(event.currentTarget.value), false)
          }
          onPointerUp={(event) =>
            onUpdateSurfaceProbe(surfaceProbeUiState.u, Number(event.currentTarget.value), true)
          }
          onTouchEnd={(event) =>
            onUpdateSurfaceProbe(surfaceProbeUiState.u, Number(event.currentTarget.value), true)
          }
        />
      </label>
      <div className="inspector-readout">
        <span>uv</span>
        <output>{`(${surfaceProbeUiState.u.toFixed(4)}, ${surfaceProbeUiState.v.toFixed(4)})`}</output>
      </div>
      <div className="inspector-readout">
        <span>D0 point</span>
        <output>{formatPoint(surfaceProbeUiState.point)}</output>
      </div>
      <div className="inspector-readout">
        <span>D1 du</span>
        <output>{`${formatVec(surfaceProbeUiState.du)} |du|=${magnitude(surfaceProbeUiState.du).toFixed(4)}`}</output>
      </div>
      <div className="inspector-readout">
        <span>D1 dv</span>
        <output>{`${formatVec(surfaceProbeUiState.dv)} |dv|=${magnitude(surfaceProbeUiState.dv).toFixed(4)}`}</output>
      </div>
      <div className="inspector-readout">
        <span>normal</span>
        <output>{formatVec(surfaceProbeUiState.normal)}</output>
      </div>
      {surfaceProbeUiState.hasD2 ? (
        <>
          <div className="inspector-readout">
            <span>D2 duu</span>
            <output>{`${formatVec(surfaceProbeUiState.duu)} |duu|=${magnitude(surfaceProbeUiState.duu).toFixed(4)}`}</output>
          </div>
          <div className="inspector-readout">
            <span>D2 duv</span>
            <output>{`${formatVec(surfaceProbeUiState.duv)} |duv|=${magnitude(surfaceProbeUiState.duv).toFixed(4)}`}</output>
          </div>
          <div className="inspector-readout">
            <span>D2 dvv</span>
            <output>{`${formatVec(surfaceProbeUiState.dvv)} |dvv|=${magnitude(surfaceProbeUiState.dvv).toFixed(4)}`}</output>
          </div>
        </>
      ) : (
        <p className="inspector-note">
          D2 is unavailable in the currently loaded runtime build.
        </p>
      )}
    </section>
  );
}
