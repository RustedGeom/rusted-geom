"use client";

import Link from "next/link";
import { useCallback, useEffect, useRef, useState } from "react";
import { ToolIcon } from "./ToolIcon";
import type { CameraMode, ViewPresetName } from "@/lib/viewer-types";

export interface ViewerToolbarProps {
  canImportIges: boolean;
  canExportIges: boolean;
  canExportSat: boolean;
  onLoadSession: () => void;
  onSaveSession: () => void;
  onExportIges: () => void;
  onExportSat: () => void;
  onExportStl: () => void;
  onExportGltf: () => void;
  exportMode: "cad" | "mesh";
  orbitEnabled: boolean;
  showGrid: boolean;
  showAxes: boolean;
  cameraMode: CameraMode;
  onToggleCameraMode: () => void;
  onApplyViewPreset: (preset: ViewPresetName) => void;
  onZoomExtents: () => void;
  onResetCamera: () => void;
  onToggleOrbit: () => void;
  onToggleGrid: () => void;
  onToggleAxes: () => void;
  onSaveScreenshot: () => void;
  isInspectorOpen: boolean;
  isConsoleOpen: boolean;
  onToggleInspector: () => void;
  onToggleConsole: () => void;
  onClearLogs: () => void;
  onExportLogs: () => void;
  isDarkMode: boolean;
  onToggleDarkMode: () => void;
  onOpenExampleBrowser: () => void;
}

export function ViewerToolbar({
  canImportIges,
  canExportIges,
  canExportSat,
  onLoadSession,
  onSaveSession,
  onExportIges,
  onExportSat,
  onExportStl,
  onExportGltf,
  exportMode,
  orbitEnabled,
  showGrid,
  showAxes,
  cameraMode,
  onToggleCameraMode,
  onApplyViewPreset,
  onZoomExtents,
  onResetCamera,
  onToggleOrbit,
  onToggleGrid,
  onToggleAxes,
  onSaveScreenshot,
  isInspectorOpen,
  isConsoleOpen,
  onToggleInspector,
  onToggleConsole,
  onClearLogs,
  onExportLogs,
  isDarkMode,
  onToggleDarkMode,
  onOpenExampleBrowser,
}: ViewerToolbarProps) {
  const [overflowOpen, setOverflowOpen] = useState(false);
  const overflowRef = useRef<HTMLDivElement>(null);
  const overflowBtnRef = useRef<HTMLButtonElement>(null);

  const closeOverflow = useCallback(() => setOverflowOpen(false), []);

  useEffect(() => {
    if (!overflowOpen) return;
    function onPointerDown(e: PointerEvent) {
      if (
        overflowRef.current?.contains(e.target as Node) ||
        overflowBtnRef.current?.contains(e.target as Node)
      ) return;
      closeOverflow();
    }
    window.addEventListener("pointerdown", onPointerDown);
    return () => window.removeEventListener("pointerdown", onPointerDown);
  }, [overflowOpen, closeOverflow]);

  useEffect(() => {
    if (!overflowOpen) return;
    function onKey(e: KeyboardEvent) { if (e.key === "Escape") closeOverflow(); }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [overflowOpen, closeOverflow]);

  function overflowAction(fn: () => void) {
    return () => { fn(); closeOverflow(); };
  }

  const isOrtho = cameraMode === "orthographic";

  return (
    <div className="toolbar-tray" role="toolbar" aria-label="Viewer actions">

      {/* ── Left float: Examples ── */}
      <div className="toolbar-float toolbar-float-left">
        <button
          type="button"
          className="tool-btn tool-btn-examples"
          onClick={onOpenExampleBrowser}
          aria-label="Browse examples (⌘K)"
          title="Browse examples (⌘K)"
        >
          <ToolIcon>
            <circle cx="7.2" cy="7.2" r="4.2" />
            <path d="m10.4 10.4 3 3" />
          </ToolIcon>
          <span className="tool-label">Examples</span>
        </button>
      </div>

      {/* ── Center float: viewport + camera controls ── */}
      <div className="toolbar-float toolbar-float-center">
        <div className="toolbar-group" role="group" aria-label="Viewport">
          <button
            type="button"
            className={`tool-btn ${orbitEnabled ? "is-active" : ""}`}
            onClick={onToggleOrbit}
            aria-label="Orbit"
            aria-pressed={orbitEnabled}
            title="Orbit"
          >
            <ToolIcon>
              <circle cx="8" cy="8" r="5.2" />
              <ellipse cx="8" cy="8" rx="5.2" ry="2.4" />
              <path d="M8 2.8v10.4" />
            </ToolIcon>
          </button>
          <button
            type="button"
            className={`tool-btn ${showGrid ? "is-active" : ""}`}
            onClick={onToggleGrid}
            aria-label="Grid"
            aria-pressed={showGrid}
            title="Grid (G)"
          >
            <ToolIcon>
              <path d="M3 13 8 8l5 5M3 13h10" />
              <path d="M5.5 10.5 8 8l2.5 2.5" />
              <path d="M8 8V3" />
              <path d="M5.5 5.5 8 3l2.5 2.5" />
            </ToolIcon>
          </button>
          <button
            type="button"
            className={`tool-btn ${isOrtho ? "is-active" : ""}`}
            onClick={onToggleCameraMode}
            aria-label={isOrtho ? "Perspective projection" : "Orthographic projection"}
            aria-pressed={isOrtho}
            title={isOrtho ? "Switch to Perspective" : "Switch to Orthographic"}
          >
            <ToolIcon>
              {isOrtho ? (
                <><rect x="3.5" y="3.5" width="9" height="9" /><path d="M5.5 3.5v9M10.5 3.5v9M3.5 5.5h9M3.5 10.5h9" /></>
              ) : (
                <><path d="M8 3.5l-5 9h10z" /><path d="M5 8.5h6" /></>
              )}
            </ToolIcon>
          </button>
        </div>

        <div className="toolbar-divider toolbar-divider-desktop" aria-hidden="true" />

        {/* Camera group — hidden on tablet/mobile */}
        <div className="toolbar-camera-group" role="group" aria-label="Camera">
          <button type="button" className="tool-btn" onClick={onZoomExtents} aria-label="Zoom to fit" title="Zoom to fit (F)">
            <ToolIcon>
              <rect x="4" y="4" width="8" height="8" />
              <path d="M2.4 2.4 4 4M12 4l1.6-1.6M12 12l1.6 1.6M4 12l-1.6 1.6" />
            </ToolIcon>
          </button>
          <button type="button" className="tool-btn" onClick={onResetCamera} aria-label="Reset view" title="Reset view (Home)">
            <ToolIcon>
              <path d="M3.5 8A4.5 4.5 0 1 1 7 12.3" />
              <path d="M3.5 5v3h3" />
            </ToolIcon>
          </button>
          <button type="button" className="tool-btn" onClick={() => onApplyViewPreset("top")} aria-label="Top view" title="Top view (T)">
            <ToolIcon>
              <rect x="4" y="4" width="8" height="8" />
              <path d="M2 2l2 2M14 2l-2 2M14 14l-2-2M2 14l2-2" />
            </ToolIcon>
          </button>
        </div>

      </div>

      {/* ── Right float: panels + theme + more ── */}
      <div className="toolbar-float toolbar-float-right">
        <div className="toolbar-group toolbar-panel-group" role="group" aria-label="Panels">
          <button
            type="button"
            className={`tool-btn ${isInspectorOpen ? "is-active" : ""}`}
            onClick={onToggleInspector}
            aria-label="Controls panel"
            aria-pressed={isInspectorOpen}
            title="Controls (I)"
          >
            <ToolIcon>
              <path d="M3 4.5h10M3 8h10M3 11.5h10" />
              <circle cx="5.5" cy="4.5" r="1.2" />
              <circle cx="10.5" cy="8" r="1.2" />
              <circle cx="7" cy="11.5" r="1.2" />
            </ToolIcon>
          </button>
          <button
            type="button"
            className={`tool-btn ${isConsoleOpen ? "is-active" : ""}`}
            onClick={onToggleConsole}
            aria-label="Console"
            aria-pressed={isConsoleOpen}
            title="Console (C)"
          >
            <ToolIcon>
              <rect x="2.5" y="3" width="11" height="10" rx="1.5" />
              <path d="M5 7.5l2 2-2 2" />
              <path d="M9 11.5h2" />
            </ToolIcon>
          </button>
        </div>

        <div className="toolbar-divider toolbar-divider-before-panels" aria-hidden="true" />

        <button
          type="button"
          className={`tool-btn ${isDarkMode ? "is-active" : ""}`}
          onClick={onToggleDarkMode}
          aria-label={isDarkMode ? "Light mode" : "Dark mode"}
          title={isDarkMode ? "Switch to light mode" : "Switch to dark mode"}
        >
          <ToolIcon>
            {isDarkMode ? (
              <><circle cx="8" cy="8" r="2.8" /><path d="M8 2v1.5M8 12.5V14M2 8h1.5M12.5 8H14M3.8 3.8l1 1M11.2 11.2l1 1M11.2 3.8l-1 1M4.8 11.2l-1 1" /></>
            ) : (
              <path d="M12.5 10A5 5 0 0 1 6 3.5a5 5 0 1 0 6.5 6.5z" />
            )}
          </ToolIcon>
        </button>

        {/* Overflow — must NOT have transform on any ancestor for fixed positioning to work */}
        <div className="toolbar-overflow-wrap" ref={overflowRef}>
          <button
            ref={overflowBtnRef}
            type="button"
            className={`tool-btn tool-btn-overflow ${overflowOpen ? "is-active" : ""}`}
            onClick={() => setOverflowOpen((v) => !v)}
            aria-label="More actions"
            aria-expanded={overflowOpen}
            title="More"
          >
            <ToolIcon>
              <circle cx="8" cy="4" r="1.2" />
              <circle cx="8" cy="8" r="1.2" />
              <circle cx="8" cy="12" r="1.2" />
            </ToolIcon>
          </button>

          {overflowOpen && (
            <>
              <div className="toolbar-overflow-backdrop" onClick={closeOverflow} />
              <div className="toolbar-overflow-menu" role="menu" aria-label="More actions">

                <div className="overflow-section overflow-section-camera">
                  <span className="overflow-label">Camera</span>
                  <div className="overflow-row">
                    <button type="button" className="overflow-btn" onClick={overflowAction(onZoomExtents)} role="menuitem">
                      <ToolIcon size={15}><rect x="4" y="4" width="8" height="8" /><path d="M2.4 2.4 4 4M12 4l1.6-1.6M12 12l1.6 1.6M4 12l-1.6 1.6" /></ToolIcon>
                      Zoom fit
                    </button>
                    <button type="button" className="overflow-btn" onClick={overflowAction(onResetCamera)} role="menuitem">
                      <ToolIcon size={15}><path d="M3.5 8A4.5 4.5 0 1 1 7 12.3" /><path d="M3.5 5v3h3" /></ToolIcon>
                      Reset
                    </button>
                    <button type="button" className={`overflow-btn ${isOrtho ? "is-active" : ""}`} onClick={overflowAction(onToggleCameraMode)} role="menuitem">
                      <ToolIcon size={15}>
                        {isOrtho
                          ? <><rect x="3.5" y="3.5" width="9" height="9" /><path d="M5.5 3.5v9M10.5 3.5v9M3.5 5.5h9M3.5 10.5h9" /></>
                          : <><path d="M8 3.5l-5 9h10z" /><path d="M5 8.5h6" /></>}
                      </ToolIcon>
                      {isOrtho ? "Persp" : "Ortho"}
                    </button>
                    <button type="button" className="overflow-btn" onClick={overflowAction(() => onApplyViewPreset("top"))} role="menuitem">
                      <ToolIcon size={15}><rect x="4" y="4" width="8" height="8" /><path d="M2 2l2 2M14 2l-2 2M14 14l-2-2M2 14l2-2" /></ToolIcon>
                      Top
                    </button>
                  </div>
                </div>

                <div className="overflow-section">
                  <span className="overflow-label">View</span>
                  <div className="overflow-row">
                    <button type="button" className={`overflow-btn ${showAxes ? "is-active" : ""}`} onClick={overflowAction(onToggleAxes)} role="menuitem" aria-pressed={showAxes}>
                      <ToolIcon size={15}><path d="M8 8 3 13M8 8l5 5M8 8V2.5" /><circle cx="8" cy="8" r="1" /></ToolIcon>
                      Axes
                    </button>
                    <button type="button" className="overflow-btn" onClick={overflowAction(onSaveScreenshot)} role="menuitem">
                      <ToolIcon size={15}><rect x="2" y="4.5" width="12" height="9" rx="1.5" /><circle cx="8" cy="9" r="2.4" /><path d="M5.5 4.5l1-2h3l1 2" /></ToolIcon>
                      Save PNG
                    </button>
                  </div>
                </div>

                <div className="overflow-section">
                  <span className="overflow-label">Session</span>
                  <div className="overflow-row">
                    <button type="button" className="overflow-btn" onClick={overflowAction(onLoadSession)} role="menuitem">
                      <ToolIcon size={15}><path d="M2 11.5V5.5h3.5l1.2-1.5H14v7.5z" /><path d="M8 7.5v3M6.5 9l1.5 1.5L9.5 9" /></ToolIcon>
                      Load
                    </button>
                    <button type="button" className="overflow-btn" onClick={overflowAction(onSaveSession)} role="menuitem">
                      <ToolIcon size={15}><rect x="3" y="2.5" width="10" height="11" rx="1" /><rect x="5" y="2.5" width="6" height="4" /><rect x="4" y="9" width="8" height="4" /></ToolIcon>
                      Save
                    </button>
                    {exportMode === "cad" ? (
                      <>
                        <button type="button" className="overflow-btn" disabled={!canExportIges} onClick={canExportIges ? overflowAction(onExportIges) : undefined} role="menuitem">
                          <ToolIcon size={15}><path d="M3 13C5 9 7 4 13 3" /><path d="M3 13h3M11 3h2v2" /></ToolIcon>
                          IGES
                        </button>
                        <button type="button" className="overflow-btn" disabled={!canExportSat} onClick={canExportSat ? overflowAction(onExportSat) : undefined} role="menuitem">
                          <ToolIcon size={15}><path d="M8 3l4 2.5v5L8 13 4 10.5v-5z" /><path d="M8 3v10M4 5.5l4 3 4-3" /></ToolIcon>
                          SAT
                        </button>
                      </>
                    ) : (
                      <>
                        <button type="button" className="overflow-btn" onClick={overflowAction(onExportStl)} role="menuitem">
                          <ToolIcon size={15}><path d="M8 3l4.5 7.5h-9zM3.5 10.5l4.5-3.5 4.5 3.5" /></ToolIcon>
                          STL
                        </button>
                        <button type="button" className="overflow-btn" onClick={overflowAction(onExportGltf)} role="menuitem">
                          <ToolIcon size={15}><path d="M8 3l4 2.3v4.4L8 12 4 9.7V5.3z" /><path d="M4 5.3l4 2.3 4-2.3M8 7.6V12" /></ToolIcon>
                          glTF
                        </button>
                      </>
                    )}
                  </div>
                </div>

                <div className="overflow-section">
                  <span className="overflow-label">Console</span>
                  <div className="overflow-row">
                    <button type="button" className="overflow-btn" onClick={overflowAction(onExportLogs)} role="menuitem">
                      <ToolIcon size={15}><rect x="3" y="2.5" width="8" height="10" rx="1" /><path d="M5 5.5h4M5 7.5h4M5 9.5h2" /><path d="M12 9v4.5M10.5 12l1.5 1.5 1.5-1.5" /></ToolIcon>
                      Export logs
                    </button>
                    <button type="button" className="overflow-btn overflow-btn-danger" onClick={overflowAction(onClearLogs)} role="menuitem">
                      <ToolIcon size={15}><path d="M4 5h8M5.5 5V4h5V5M5.5 5l.5 8h4l.5-8" /><path d="M6.8 7.5l2.4 3M9.2 7.5l-2.4 3" /></ToolIcon>
                      Clear logs
                    </button>
                  </div>
                </div>

                <div className="overflow-section">
                  <span className="overflow-label">Diagnostics</span>
                  <div className="overflow-row overflow-row-single">
                    <Link href="/tests" className="overflow-link" role="menuitem" onClick={closeOverflow} aria-label="Open Test Lab">
                      <ToolIcon size={15}><path d="M6 3v5l-3 5h10l-3-5V3" /><path d="M5.5 3h5" /><circle cx="9" cy="11" r="0.8" /></ToolIcon>
                      Open Test Lab
                    </Link>
                  </div>
                </div>

              </div>
            </>
          )}
        </div>
      </div>

    </div>
  );
}
