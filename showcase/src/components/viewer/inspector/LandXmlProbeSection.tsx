import type { LandXmlAlignmentInfo, LandXmlProbeUiState } from "@/lib/viewer-types";

interface LandXmlProbeSectionProps {
  alignments: LandXmlAlignmentInfo[];
  probeState: LandXmlProbeUiState;
  selectedAlignIdx: number;
  selectedProfIdx: number;
  onAlignmentChange: (idx: number) => void;
  onProfileChange: (idx: number) => void;
  onStationChange: (stationNorm: number, commit: boolean) => void;
}

function fmtVec(v: { x: number; y: number; z: number }): string {
  return `(${v.x.toFixed(4)}, ${v.y.toFixed(4)}, ${v.z.toFixed(4)})`;
}

export function LandXmlProbeSection({
  alignments,
  probeState,
  selectedAlignIdx,
  selectedProfIdx,
  onAlignmentChange,
  onProfileChange,
  onStationChange,
}: LandXmlProbeSectionProps) {
  const info = alignments[selectedAlignIdx];
  const hasMultipleAlignments = alignments.length > 1;
  const hasMultipleProfiles = (info?.profileCount ?? 0) > 1;
  const hasProfiles = (info?.profileCount ?? 0) > 0;

  return (
    <section className="inspector-section" aria-label="LandXML Alignment Probe">
      <h2>Alignment Probe</h2>

      {hasMultipleAlignments ? (
        <label className="inspector-field">
          <span>Alignment</span>
          <select
            style={{ width: "100%", appearance: "auto", textAlign: "left", cursor: "pointer" }}
            value={selectedAlignIdx}
            onChange={(e) => onAlignmentChange(Number(e.target.value))}
          >
            {alignments.map((a) => (
              <option key={a.index} value={a.index}>
                {a.name}
              </option>
            ))}
          </select>
        </label>
      ) : null}

      {hasMultipleProfiles && info ? (
        <label className="inspector-field">
          <span>Profile</span>
          <select
            style={{ width: "100%", appearance: "auto", textAlign: "left", cursor: "pointer" }}
            value={selectedProfIdx}
            onChange={(e) => onProfileChange(Number(e.target.value))}
          >
            {info.profileNames.map((name, i) => (
              <option key={i} value={i}>
                {name}
              </option>
            ))}
          </select>
        </label>
      ) : null}

      {!hasProfiles ? (
        <p className="inspector-note">No profiles for selected alignment.</p>
      ) : (
        <>
          <label className="inspector-field">
            <span>Station</span>
            <input
              type="range"
              min={0}
              max={1}
              step={0.001}
              value={probeState.stationNorm}
              onChange={(e) => onStationChange(Number(e.currentTarget.value), false)}
              onPointerUp={(e) => onStationChange(Number(e.currentTarget.value), true)}
              onTouchEnd={(e) => onStationChange(Number(e.currentTarget.value), true)}
            />
          </label>
          <div className="inspector-readout">
            <span>Station</span>
            <output>{probeState.station.toFixed(2)}</output>
          </div>
          <div className="inspector-readout">
            <span>Grade</span>
            <output>{(probeState.grade * 100).toFixed(2)}%</output>
          </div>

          <h3 style={{ fontSize: 11, margin: "8px 0 2px", opacity: 0.7 }}>Alignment Point</h3>
          <div className="inspector-readout">
            <span>x</span>
            <output>{probeState.alignmentPoint.x.toFixed(3)}</output>
          </div>
          <div className="inspector-readout">
            <span>y</span>
            <output>{probeState.alignmentPoint.y.toFixed(3)}</output>
          </div>
          <div className="inspector-readout">
            <span>z</span>
            <output>{probeState.alignmentPoint.z.toFixed(3)}</output>
          </div>

          <h3 style={{ fontSize: 11, margin: "8px 0 2px", opacity: 0.7 }}>Profile Point</h3>
          <div className="inspector-readout">
            <span>x</span>
            <output>{probeState.profilePoint.x.toFixed(3)}</output>
          </div>
          <div className="inspector-readout">
            <span>y</span>
            <output>{probeState.profilePoint.y.toFixed(3)}</output>
          </div>
          <div className="inspector-readout">
            <span>z</span>
            <output>{probeState.profilePoint.z.toFixed(3)}</output>
          </div>

          <div className="inspector-readout">
            <span>Tangent</span>
            <output style={{ fontSize: 10 }}>{fmtVec(probeState.tangent)}</output>
          </div>

        </>
      )}
    </section>
  );
}
