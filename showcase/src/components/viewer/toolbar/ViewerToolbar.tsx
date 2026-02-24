"use client";

import Link from "next/link";
import { useCallback, useEffect, useRef, useState } from "react";
import { ToolIcon } from "./ToolIcon";

export interface ViewerToolbarProps {
  canImportIges: boolean;
  canExportIges: boolean;
  onLoadSession: () => void;
  onSaveSession: () => void;
  orbitEnabled: boolean;
  showGrid: boolean;
  showAxes: boolean;
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
  onLoadSession,
  onSaveSession,
  orbitEnabled,
  showGrid,
  showAxes,
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

  // Close on outside click
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

  // Close on Escape
  useEffect(() => {
    if (!overflowOpen) return;
    function onKey(e: KeyboardEvent) { if (e.key === "Escape") closeOverflow(); }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [overflowOpen, closeOverflow]);

  function overflowAction(fn: () => void) {
    return () => { fn(); closeOverflow(); };
  }

  return (
    <header className="toolbar" role="toolbar" aria-label="Viewer actions">
      {/* Brand */}
      <div className="toolbar-brand" aria-label="rusted-geom">
        <span className="toolbar-brand-mark" aria-hidden="true">◈</span>
        <span className="toolbar-brand-name">rusted-geom</span>
      </div>

      <div className="toolbar-divider" aria-hidden="true" />

      {/* Examples — primary action, always prominent */}
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

      <div className="toolbar-divider" aria-hidden="true" />

      {/* Viewport toggles */}
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
            <circle cx="8" cy="8" r="1.8" />
            <ellipse cx="8" cy="8" rx="5.2" ry="2.8" />
            <path d="M8 2.8a5.2 5.2 0 0 1 0 10.4" />
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
            <path d="M3 3h10v10H3z" />
            <path d="M6.3 3v10M9.7 3v10M3 6.3h10M3 9.7h10" />
          </ToolIcon>
        </button>
      </div>

      <div className="toolbar-divider" aria-hidden="true" />

      {/* Panel toggles */}
      <div className="toolbar-group" role="group" aria-label="Panels">
        <button
          type="button"
          className={`tool-btn ${isInspectorOpen ? "is-active" : ""}`}
          onClick={onToggleInspector}
          aria-label="Controls panel"
          aria-pressed={isInspectorOpen}
          title="Controls (I)"
        >
          <ToolIcon>
            <path d="M3 4.2h10M3 8h10M3 11.8h10" />
            <circle cx="5.4" cy="4.2" r="0.9" />
            <circle cx="10.6" cy="8" r="0.9" />
            <circle cx="7.2" cy="11.8" r="0.9" />
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
            <rect x="2.6" y="3.1" width="10.8" height="9.8" rx="1.4" />
            <path d="M4.7 10.2h6.6M4.7 7.8h3.8" />
          </ToolIcon>
        </button>
      </div>

      <div className="toolbar-divider" aria-hidden="true" />

      {/* Theme + overflow */}
      <button
        type="button"
        className={`tool-btn ${isDarkMode ? "is-active" : ""}`}
        onClick={onToggleDarkMode}
        aria-label={isDarkMode ? "Light mode" : "Dark mode"}
        title={isDarkMode ? "Light mode" : "Dark mode"}
      >
        <ToolIcon>
          {isDarkMode ? (
            <>
              <circle cx="8" cy="8" r="3.2" />
              <path d="M8 1.6v1.6M8 12.8v1.6M1.6 8h1.6M12.8 8h1.6" />
              <path d="m3.7 3.7 1.1 1.1m6.4 6.4 1.1 1.1m0-8.6-1.1 1.1M4.8 11.2l-1.1 1.1" />
            </>
          ) : (
            <path d="M13.5 10.5A5.5 5.5 0 0 1 5.5 2.5a5.5 5.5 0 1 0 8 8z" />
          )}
        </ToolIcon>
      </button>

      {/* Overflow trigger */}
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
          <span className="overflow-dots">
            <span /><span /><span />
          </span>
        </button>

        {overflowOpen && (
          <div className="toolbar-overflow-menu" role="menu" aria-label="More actions">
            <div className="overflow-section">
              <span className="overflow-label">Camera</span>
              <div className="overflow-row">
                <button type="button" className="overflow-btn" onClick={overflowAction(onZoomExtents)} role="menuitem">
                  <ToolIcon size={13}>
                    <rect x="3.2" y="3.2" width="9.6" height="9.6" />
                    <path d="M1.6 1.6 4 4m8 8 2.4 2.4M14.4 1.6 12 4M1.6 14.4l2.4-2.4" />
                  </ToolIcon>
                  Zoom fit
                </button>
                <button type="button" className="overflow-btn" onClick={overflowAction(onResetCamera)} role="menuitem">
                  <ToolIcon size={13}>
                    <path d="M8 2.6a5.4 5.4 0 1 0 5.1 7.2" />
                    <path d="M10.6 2.8h2.8v2.8m-2.6 0 2.6-2.8" />
                  </ToolIcon>
                  Reset view
                </button>
                <button
                  type="button"
                  className={`overflow-btn ${showAxes ? "is-active" : ""}`}
                  onClick={overflowAction(onToggleAxes)}
                  role="menuitem"
                  aria-pressed={showAxes}
                >
                  <ToolIcon size={13}>
                    <path d="M2.8 12.8 8 8 13.2 3.2M8 8v5.2" />
                    <circle cx="8" cy="8" r="1.1" />
                  </ToolIcon>
                  Axes
                </button>
                <button type="button" className="overflow-btn" onClick={overflowAction(onSaveScreenshot)} role="menuitem">
                  <ToolIcon size={13}>
                    <rect x="2.8" y="4.1" width="10.4" height="8.2" rx="1.2" />
                    <path d="m5.2 9.8 1.8-1.8 1.8 1.8 1.3-1.3 1.9 1.9" />
                    <circle cx="6.1" cy="6.6" r="0.8" />
                  </ToolIcon>
                  Save PNG
                </button>
              </div>
            </div>

            <div className="overflow-section">
              <span className="overflow-label">Session</span>
              <div className="overflow-row">
                <button type="button" className="overflow-btn" onClick={overflowAction(onLoadSession)} role="menuitem">
                  <ToolIcon size={13}>
                    <path d="M2.5 5.5h4.2l1.2 1.2h5.8v5.8H2.5z" />
                    <path d="M8 9.1v3.4m-1.4-1.4 1.4 1.4 1.4-1.4" />
                  </ToolIcon>
                  Load
                </button>
                <button type="button" className="overflow-btn" onClick={overflowAction(onSaveSession)} role="menuitem">
                  <ToolIcon size={13}>
                    <path d="M3 2.7h8.4l1.6 1.6V13.3H3z" />
                    <path d="M5 2.7v3.2h5.5V2.7M5.2 10.6h5.5" />
                  </ToolIcon>
                  Save
                </button>
                <button
                  type="button"
                  className="overflow-btn"
                  disabled={!canImportIges}
                  title="IGES import pending"
                  role="menuitem"
                >
                  <ToolIcon size={13}>
                    <path d="M2.8 3.2h10.4v9.6H2.8zM4.9 6.2h6.2M4.9 8h4.2M4.9 9.8h6.2" />
                  </ToolIcon>
                  IGES in
                </button>
                <button
                  type="button"
                  className="overflow-btn"
                  disabled={!canExportIges}
                  title="IGES export pending"
                  role="menuitem"
                >
                  <ToolIcon size={13}>
                    <path d="M2.8 3.2h10.4v9.6H2.8zM8 6v4.2m-1.6-1.3L8 10.2l1.6-1.6" />
                  </ToolIcon>
                  IGES out
                </button>
              </div>
            </div>

            <div className="overflow-section">
              <span className="overflow-label">Console</span>
              <div className="overflow-row">
                <button type="button" className="overflow-btn" onClick={overflowAction(onExportLogs)} role="menuitem">
                  <ToolIcon size={13}>
                    <path d="M8 2.5v7.2m-2.4-2.4L8 9.7l2.4-2.4" />
                    <path d="M3 11.1h10v2.2H3z" />
                  </ToolIcon>
                  Export logs
                </button>
                <button type="button" className="overflow-btn overflow-btn-danger" onClick={overflowAction(onClearLogs)} role="menuitem">
                  <ToolIcon size={13}>
                    <path d="M3.9 4.2h8.2M5 4.2v8.1h6v-8.1M6.3 6.1v4.2M9.7 6.1v4.2M6.1 2.8h3.8" />
                  </ToolIcon>
                  Clear logs
                </button>
              </div>
            </div>

            <div className="overflow-section">
              <span className="overflow-label">Diagnostics</span>
              <div className="overflow-row overflow-row-single">
                <Link
                  href="/tests"
                  className="overflow-link"
                  role="menuitem"
                  onClick={closeOverflow}
                  aria-label="Open Test Lab"
                >
                  <ToolIcon size={13}>
                    <path d="M3.2 3.2h9.6v9.6H3.2z" />
                    <path d="M5.2 10.8 7 8.4l1.4 1.5 2.4-3.2" />
                  </ToolIcon>
                  Open Test Lab
                </Link>
              </div>
            </div>
          </div>
        )}
      </div>
    </header>
  );
}
