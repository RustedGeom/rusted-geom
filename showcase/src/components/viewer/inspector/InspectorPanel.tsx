import type { ExampleKey, GizmoMode, ProbeUiState, SurfaceProbeUiState, ViewerPerformance } from "@/lib/viewer-types";
import { ExampleSection } from "./ExampleSection";
import { GizmoSection } from "./GizmoSection";
import { PerformanceSection } from "./PerformanceSection";
import { ProbeSection, ProbeUnavailableSection } from "./ProbeSection";
import { SurfaceProbeSection } from "./SurfaceProbeSection";

export interface InspectorPanelProps {
  isOpen: boolean;
  activeExample: ExampleKey;
  activeCurveName: string;
  activeDegreeLabel: string;
  perfStats: ViewerPerformance;
  showGizmoControls: boolean;
  showTransformTargetControls: boolean;
  showMeshPlaneTargetControls: boolean;
  showSurfaceProbeControls: boolean;
  showProbeControls: boolean;
  gizmoMode: GizmoMode;
  onSetGizmoMode: (mode: GizmoMode) => void;
  transformTargetsUi: Array<{ key: string; label: string }>;
  transformTargetKey: string;
  onTransformTargetChange: (key: string) => void;
  meshPlaneTarget: "mesh" | "plane";
  onMeshPlaneTargetChange: (target: "mesh" | "plane") => void;
  probeUiState: ProbeUiState;
  onUpdateProbe: (tNorm: number, commit: boolean) => void;
  surfaceProbeUiState: SurfaceProbeUiState;
  onUpdateSurfaceProbe: (u: number, v: number, commit: boolean) => void;
  onOpenExampleBrowser: () => void;
}

export function InspectorPanel({
  isOpen,
  activeExample,
  activeCurveName,
  activeDegreeLabel,
  perfStats,
  showGizmoControls,
  showTransformTargetControls,
  showMeshPlaneTargetControls,
  showSurfaceProbeControls,
  showProbeControls,
  gizmoMode,
  onSetGizmoMode,
  transformTargetsUi,
  transformTargetKey,
  onTransformTargetChange,
  meshPlaneTarget,
  onMeshPlaneTargetChange,
  probeUiState,
  onUpdateProbe,
  surfaceProbeUiState,
  onUpdateSurfaceProbe,
  onOpenExampleBrowser,
}: InspectorPanelProps) {
  return (
    <aside
      className={`inspector-panel ${isOpen ? "is-open" : "is-collapsed"}`}
      aria-label="Viewer controls"
      aria-hidden={!isOpen}
    >
      <div className="inspector-header">
        <strong>Controls</strong>
      </div>
      <div className="inspector-body">
        <ExampleSection
          activeExample={activeExample}
          activeCurveName={activeCurveName}
          activeDegreeLabel={activeDegreeLabel}
          onOpenExampleBrowser={onOpenExampleBrowser}
        />

        <PerformanceSection perfStats={perfStats} />

        {showGizmoControls ? (
          <GizmoSection
            gizmoMode={gizmoMode}
            onSetGizmoMode={onSetGizmoMode}
            showTransformTargetControls={showTransformTargetControls}
            transformTargetsUi={transformTargetsUi}
            transformTargetKey={transformTargetKey}
            onTransformTargetChange={onTransformTargetChange}
            showMeshPlaneTargetControls={showMeshPlaneTargetControls}
            meshPlaneTarget={meshPlaneTarget}
            onMeshPlaneTargetChange={onMeshPlaneTargetChange}
          />
        ) : null}

        {showSurfaceProbeControls ? (
          <SurfaceProbeSection
            surfaceProbeUiState={surfaceProbeUiState}
            onUpdateSurfaceProbe={onUpdateSurfaceProbe}
          />
        ) : showProbeControls ? (
          <ProbeSection probeUiState={probeUiState} onUpdateProbe={onUpdateProbe} />
        ) : (
          <ProbeUnavailableSection />
        )}
      </div>
    </aside>
  );
}
