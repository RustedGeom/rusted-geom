import type { GizmoMode } from "@/lib/viewer-types";

interface GizmoSectionProps {
  gizmoMode: GizmoMode;
  onSetGizmoMode: (mode: GizmoMode) => void;
  showTransformTargetControls: boolean;
  transformTargetsUi: Array<{ key: string; label: string }>;
  transformTargetKey: string;
  onTransformTargetChange: (key: string) => void;
  showMeshPlaneTargetControls: boolean;
  meshPlaneTarget: "mesh" | "plane";
  onMeshPlaneTargetChange: (target: "mesh" | "plane") => void;
}

export function GizmoSection({
  gizmoMode,
  onSetGizmoMode,
  showTransformTargetControls,
  transformTargetsUi,
  transformTargetKey,
  onTransformTargetChange,
  showMeshPlaneTargetControls,
  meshPlaneTarget,
  onMeshPlaneTargetChange,
}: GizmoSectionProps) {
  return (
    <section className="inspector-section" aria-label="Mesh transform gizmo controls">
      <h2>Gizmo</h2>
      {showTransformTargetControls ? (
        <label className="inspector-field">
          <span>Target</span>
          <select
            value={transformTargetKey}
            onChange={(event) => onTransformTargetChange(event.currentTarget.value)}
          >
            {transformTargetsUi.map((target) => (
              <option key={target.key} value={target.key}>
                {target.label}
              </option>
            ))}
          </select>
        </label>
      ) : null}
      {showMeshPlaneTargetControls ? (
        <label className="inspector-field">
          <span>Element</span>
          <select
            value={meshPlaneTarget}
            onChange={(event) =>
              onMeshPlaneTargetChange(event.currentTarget.value as "mesh" | "plane")
            }
          >
            <option value="mesh">Section mesh</option>
            <option value="plane">Section plane</option>
          </select>
        </label>
      ) : null}
      <div className="gizmo-mode-row">
        <button
          type="button"
          className={`tool-btn ${gizmoMode === "translate" ? "is-active" : ""}`}
          onClick={() => onSetGizmoMode("translate")}
        >
          Translate
        </button>
        <button
          type="button"
          className={`tool-btn ${gizmoMode === "rotate" ? "is-active" : ""}`}
          onClick={() => onSetGizmoMode("rotate")}
        >
          Rotate
        </button>
        <button
          type="button"
          className={`tool-btn ${gizmoMode === "scale" ? "is-active" : ""}`}
          disabled={showMeshPlaneTargetControls && meshPlaneTarget === "plane"}
          onClick={() => onSetGizmoMode("scale")}
        >
          Scale
        </button>
      </div>
      <p className="inspector-note">
        Drag in viewport to transform. Kernel update is committed when drag ends.
      </p>
    </section>
  );
}
