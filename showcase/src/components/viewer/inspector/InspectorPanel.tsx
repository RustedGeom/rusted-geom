import type { ExampleKey, GizmoMode, LandXmlAlignmentInfo, LandXmlProbeUiState, ProbeUiState, SurfaceProbeUiState, ViewerPerformance } from "@/lib/viewer-types";
import { LANDXML_FILE_LIST } from "@/lib/viewer-types";
import { ExampleSection } from "./ExampleSection";
import { GizmoSection } from "./GizmoSection";
import { LandXmlProbeSection } from "./LandXmlProbeSection";
import { PerformanceSection } from "./PerformanceSection";
import { ProbeSection, ProbeUnavailableSection } from "./ProbeSection";
import { SurfaceProbeSection } from "./SurfaceProbeSection";

export interface LandXmlInspectorStats {
  surfCount: number;
  alignCount: number;
  vertCount: number;
  featureLineCount: number;
  breaklineCount: number;
  unit: string;
  warnCount: number;
  parseMs: number;
}

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
  activeLandXmlFile?: string;
  onLandXmlFileChange?: (filename: string) => void;
  landXmlStats?: LandXmlInspectorStats | null;
  landXmlDatumOffset?: number;
  onLandXmlDatumOffsetChange?: (offset: number) => void;
  landXmlVertExag?: number;
  onLandXmlVertExagChange?: (exag: number) => void;
  landXmlZRange?: { min: number; max: number };
  landXmlAlignments?: LandXmlAlignmentInfo[];
  landXmlProbeState?: LandXmlProbeUiState;
  landXmlProbeAlignIdx?: number;
  landXmlProbeProfileIdx?: number;
  onLandXmlAlignmentChange?: (idx: number) => void;
  onLandXmlProfileChange?: (idx: number) => void;
  onLandXmlStationChange?: (stationNorm: number, commit: boolean) => void;
  followCamera?: boolean;
  onToggleFollowCamera?: () => void;
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
  activeLandXmlFile,
  onLandXmlFileChange,
  landXmlStats,
  landXmlDatumOffset = 0,
  onLandXmlDatumOffsetChange,
  landXmlVertExag = 1,
  onLandXmlVertExagChange,
  landXmlZRange,
  landXmlAlignments,
  landXmlProbeState,
  landXmlProbeAlignIdx = 0,
  landXmlProbeProfileIdx = 0,
  onLandXmlAlignmentChange,
  onLandXmlProfileChange,
  onLandXmlStationChange,
  followCamera = false,
  onToggleFollowCamera,
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

        {activeExample === "landxmlViewer" && onLandXmlFileChange ? (
          <section className="inspector-section" aria-label="LandXML File">
            <h2>LandXML File</h2>
            <select
              className="example-trigger"
              style={{ width: "100%", appearance: "auto", textAlign: "left", cursor: "pointer" }}
              value={activeLandXmlFile ?? ""}
              onChange={(e) => onLandXmlFileChange(e.target.value)}
            >
              {LANDXML_FILE_LIST.map((f) => (
                <option key={f} value={f}>{f}</option>
              ))}
            </select>
            {landXmlStats ? (
              <>
                <div className="inspector-readout">
                  <span>Surfaces</span>
                  <output>{landXmlStats.surfCount}</output>
                </div>
                <div className="inspector-readout">
                  <span>Alignments</span>
                  <output>{landXmlStats.alignCount}</output>
                </div>
                <div className="inspector-readout">
                  <span>Vertices</span>
                  <output>{landXmlStats.vertCount.toLocaleString()}</output>
                </div>
                {(landXmlStats.featureLineCount > 0 || landXmlStats.breaklineCount > 0) ? (
                  <>
                    <div className="inspector-readout">
                      <span>FeatureLines</span>
                      <output>{landXmlStats.featureLineCount}</output>
                    </div>
                    <div className="inspector-readout">
                      <span>Breaklines</span>
                      <output>{landXmlStats.breaklineCount}</output>
                    </div>
                  </>
                ) : null}
                <div className="inspector-readout">
                  <span>Units</span>
                  <output>{landXmlStats.unit}</output>
                </div>
                <div className="inspector-readout">
                  <span>Warnings</span>
                  <output>{landXmlStats.warnCount}</output>
                </div>
                <div className="inspector-readout">
                  <span>Parse time</span>
                  <output>{landXmlStats.parseMs.toFixed(0)}ms</output>
                </div>
              </>
            ) : null}

            {onLandXmlDatumOffsetChange && landXmlZRange ? (
              <div style={{ marginTop: 8 }}>
                <label style={{ display: "block", fontSize: 12, marginBottom: 2 }}>
                  Datum offset <span style={{ opacity: 0.6 }}>{landXmlDatumOffset.toFixed(1)}m</span>
                </label>
                <input
                  type="range"
                  min={0}
                  max={landXmlZRange.max}
                  step={0.5}
                  value={landXmlDatumOffset}
                  onChange={(e) => onLandXmlDatumOffsetChange(Number(e.target.value))}
                  style={{ width: "100%" }}
                />
              </div>
            ) : null}

            {onLandXmlVertExagChange ? (
              <div style={{ marginTop: 8 }}>
                <label style={{ display: "block", fontSize: 12, marginBottom: 2 }}>
                  Vertical exaggeration
                </label>
                <div style={{ display: "flex", gap: 4 }}>
                  {[1, 5, 10, 20].map((v) => (
                    <button
                      key={v}
                      className="example-trigger"
                      style={{
                        flex: 1,
                        padding: "3px 0",
                        fontSize: 12,
                        fontWeight: landXmlVertExag === v ? 700 : 400,
                        opacity: landXmlVertExag === v ? 1 : 0.6,
                        border: landXmlVertExag === v ? "1px solid var(--accent)" : "1px solid transparent",
                      }}
                      onClick={() => onLandXmlVertExagChange(v)}
                    >
                      {v}×
                    </button>
                  ))}
                </div>
              </div>
            ) : null}
          </section>
        ) : null}

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

        {activeExample === "landxmlViewer" && landXmlAlignments && landXmlProbeState && onLandXmlStationChange ? (
          <LandXmlProbeSection
            alignments={landXmlAlignments}
            probeState={landXmlProbeState}
            selectedAlignIdx={landXmlProbeAlignIdx}
            selectedProfIdx={landXmlProbeProfileIdx}
            onAlignmentChange={onLandXmlAlignmentChange ?? (() => {})}
            onProfileChange={onLandXmlProfileChange ?? (() => {})}
            onStationChange={onLandXmlStationChange}
            followCamera={followCamera}
            onToggleFollowCamera={onToggleFollowCamera}
          />
        ) : showSurfaceProbeControls ? (
          <SurfaceProbeSection
            surfaceProbeUiState={surfaceProbeUiState}
            onUpdateSurfaceProbe={onUpdateSurfaceProbe}
          />
        ) : showProbeControls ? (
          <ProbeSection probeUiState={probeUiState} onUpdateProbe={onUpdateProbe} followCamera={followCamera} onToggleFollowCamera={onToggleFollowCamera} />
        ) : (
          <ProbeUnavailableSection />
        )}
      </div>
    </aside>
  );
}
