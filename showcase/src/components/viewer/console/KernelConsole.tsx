import { useEffect, useRef } from "react";
import type { LogEntry, LogLevel } from "@/lib/viewer-types";
import { ConsoleLogEntry } from "./ConsoleLogEntry";

export interface KernelConsoleProps {
  isOpen: boolean;
  logs: LogEntry[];
  onExportLogs: () => void;
  onClearLogs: () => void;
  activeFilter: LogLevel | "all";
  onFilterChange: (filter: LogLevel | "all") => void;
}

const FILTER_OPTIONS: Array<{ value: LogLevel | "all"; label: string }> = [
  { value: "all", label: "All" },
  { value: "info", label: "Info" },
  { value: "debug", label: "Debug" },
  { value: "error", label: "Error" },
];

export function KernelConsole({
  isOpen,
  logs,
  onExportLogs,
  onClearLogs,
  activeFilter,
  onFilterChange,
}: KernelConsoleProps) {
  const logBodyRef = useRef<HTMLDivElement | null>(null);

  const filteredLogs =
    activeFilter === "all" ? logs : logs.filter((entry) => entry.level === activeFilter);

  const counts = {
    all: logs.length,
    info: logs.filter((e) => e.level === "info").length,
    debug: logs.filter((e) => e.level === "debug").length,
    error: logs.filter((e) => e.level === "error").length,
  };

  useEffect(() => {
    const body = logBodyRef.current;
    if (!body) return;
    body.scrollTop = body.scrollHeight;
  }, [filteredLogs.length]);

  return (
    <aside
      className={`kernel-console ${isOpen ? "is-open" : "is-collapsed"}`}
      aria-label="Kernel console"
      aria-hidden={!isOpen}
    >
      <div className="kernel-console-header">
        <strong>Kernel Console</strong>
        <div className="console-filter-row">
          {FILTER_OPTIONS.map((opt) => (
            <button
              key={opt.value}
              type="button"
              className={`console-filter-btn ${activeFilter === opt.value ? "is-active" : ""} ${opt.value === "error" ? "is-danger" : ""}`}
              onClick={() => onFilterChange(opt.value)}
            >
              {opt.label}
              {counts[opt.value] > 0 && (
                <span className="console-filter-count">{counts[opt.value]}</span>
              )}
            </button>
          ))}
        </div>
        <div className="kernel-console-actions">
          <button type="button" className="tool-btn" onClick={onExportLogs}>
            Export
          </button>
          <button type="button" className="tool-btn" onClick={onClearLogs}>
            Clear
          </button>
        </div>
      </div>
      <div ref={logBodyRef} className="kernel-console-body">
        {filteredLogs.length > 0 ? (
          filteredLogs.map((entry) => <ConsoleLogEntry key={entry.id} entry={entry} />)
        ) : (
          <div className="kernel-console-empty">
            {activeFilter === "all" ? "No logs yet." : `No ${activeFilter} logs.`}
          </div>
        )}
      </div>
    </aside>
  );
}
