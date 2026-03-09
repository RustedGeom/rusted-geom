import type { RgmPoint3 } from "@rustedgeom/kernel";

export interface ClosestPointPair {
  queryPt: RgmPoint3;
  footPt:  RgmPoint3;
  t?:      number;
  u?:      number;
  v?:      number;
  distance: number;
}

interface ClosestPointSectionProps {
  pairs: ClosestPointPair[];
  selectedIdx: number;
  onSelect: (idx: number) => void;
  kind: "curve" | "surface";
}

export function ClosestPointSection({ pairs, selectedIdx, onSelect, kind }: ClosestPointSectionProps) {
  if (pairs.length === 0) {
    return (
      <section className="inspector-section" aria-label="Closest point">
        <h2>Closest Point</h2>
        <p className="inspector-note">No projection pairs.</p>
      </section>
    );
  }

  return (
    <section className="inspector-section" aria-label="Closest point">
      <h2>Closest Point</h2>
      {pairs.map((pair, i) => {
        const isSelected = i === selectedIdx;
        return (
          <div
            key={i}
            onClick={() => onSelect(i)}
            style={{
              marginBottom: 8,
              padding: "6px 8px",
              borderRadius: 4,
              cursor: "pointer",
              border: isSelected ? "1px solid var(--accent, #4a90d9)" : "1px solid transparent",
              background: isSelected ? "rgba(74,144,217,0.08)" : "transparent",
            }}
          >
            <div style={{ fontSize: 11, opacity: 0.5, marginBottom: 4 }}>#{i + 1}</div>

            <div style={{ fontSize: 11, fontWeight: 600, color: "#ff8c42", marginBottom: 2 }}>Query</div>
            <div className="inspector-readout">
              <span>x</span><output>{pair.queryPt.x.toFixed(5)}</output>
            </div>
            <div className="inspector-readout">
              <span>y</span><output>{pair.queryPt.y.toFixed(5)}</output>
            </div>
            <div className="inspector-readout">
              <span>z</span><output>{pair.queryPt.z.toFixed(5)}</output>
            </div>

            <div style={{ fontSize: 11, fontWeight: 600, color: "#42d9c8", marginBottom: 2, marginTop: 4 }}>
              {kind === "curve" ? "On curve" : "On surface"}
            </div>
            <div className="inspector-readout">
              <span>x</span><output>{pair.footPt.x.toFixed(5)}</output>
            </div>
            <div className="inspector-readout">
              <span>y</span><output>{pair.footPt.y.toFixed(5)}</output>
            </div>
            <div className="inspector-readout">
              <span>z</span><output>{pair.footPt.z.toFixed(5)}</output>
            </div>

            {kind === "curve" && pair.t !== undefined ? (
              <div className="inspector-readout" style={{ marginTop: 4 }}>
                <span>t</span><output>{pair.t.toFixed(5)}</output>
              </div>
            ) : null}
            {kind === "surface" && pair.u !== undefined && pair.v !== undefined ? (
              <>
                <div className="inspector-readout" style={{ marginTop: 4 }}>
                  <span>u</span><output>{pair.u.toFixed(5)}</output>
                </div>
                <div className="inspector-readout">
                  <span>v</span><output>{pair.v.toFixed(5)}</output>
                </div>
              </>
            ) : null}

            <div className="inspector-readout">
              <span>distance</span><output>{pair.distance.toFixed(5)}</output>
            </div>
          </div>
        );
      })}
    </section>
  );
}
