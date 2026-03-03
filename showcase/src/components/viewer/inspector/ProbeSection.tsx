import type { ProbeUiState } from "@/lib/viewer-types";

function fmtVec(v: { x: number; y: number; z: number }): string {
  return `(${v.x.toFixed(4)}, ${v.y.toFixed(4)}, ${v.z.toFixed(4)})`;
}

interface ProbeSectionProps {
  probeUiState: ProbeUiState;
  onUpdateProbe: (tNorm: number, commit: boolean) => void;
  followCamera?: boolean;
  onToggleFollowCamera?: () => void;
}

export function ProbeSection({ probeUiState, onUpdateProbe, followCamera = false, onToggleFollowCamera }: ProbeSectionProps) {
  return (
    <section className="inspector-section" aria-label="Probe controls">
      <h2>Probe</h2>
      <label className="inspector-field">
        <span>t</span>
        <input
          type="range"
          min={0}
          max={1}
          step={0.0005}
          value={probeUiState.tNorm}
          onChange={(event) => onUpdateProbe(Number(event.currentTarget.value), false)}
          onPointerUp={(event) => onUpdateProbe(Number(event.currentTarget.value), true)}
          onTouchEnd={(event) => onUpdateProbe(Number(event.currentTarget.value), true)}
        />
      </label>
      <div className="inspector-readout">
        <span>t value</span>
        <output>{probeUiState.tNorm.toFixed(5)}</output>
      </div>
      <div className="inspector-readout">
        <span>x</span>
        <output>{probeUiState.x.toFixed(5)}</output>
      </div>
      <div className="inspector-readout">
        <span>y</span>
        <output>{probeUiState.y.toFixed(5)}</output>
      </div>
      <div className="inspector-readout">
        <span>z</span>
        <output>{probeUiState.z.toFixed(5)}</output>
      </div>
      <div className="inspector-readout">
        <span>s(t)</span>
        <output>{probeUiState.probeLength.toFixed(5)}</output>
      </div>
      <div className="inspector-readout">
        <span>s(total)</span>
        <output>{probeUiState.totalLength.toFixed(5)}</output>
      </div>

      {probeUiState.tangent ? (
        <div className="inspector-readout">
          <span>Tangent</span>
          <output style={{ fontSize: 10 }}>{fmtVec(probeUiState.tangent)}</output>
        </div>
      ) : null}
      {probeUiState.normal ? (
        <div className="inspector-readout">
          <span>Normal</span>
          <output style={{ fontSize: 10 }}>{fmtVec(probeUiState.normal)}</output>
        </div>
      ) : null}

      {onToggleFollowCamera ? (
        <label className="inspector-field inspector-toggle">
          <span>Follow camera</span>
          <input type="checkbox" checked={followCamera} onChange={onToggleFollowCamera} />
        </label>
      ) : null}
    </section>
  );
}

export function ProbeUnavailableSection() {
  return (
    <section className="inspector-section" aria-label="Probe controls unavailable">
      <h2>Probe</h2>
      <p className="inspector-note">
        Probe readout is hidden for intersection-focused examples.
      </p>
    </section>
  );
}
