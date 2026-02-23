import type { ProbeUiState } from "@/lib/viewer-types";

interface ProbeSectionProps {
  probeUiState: ProbeUiState;
  onUpdateProbe: (tNorm: number, commit: boolean) => void;
}

export function ProbeSection({ probeUiState, onUpdateProbe }: ProbeSectionProps) {
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
