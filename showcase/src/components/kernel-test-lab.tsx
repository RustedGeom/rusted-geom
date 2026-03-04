"use client";

import Link from "next/link";
import { useCallback, useMemo, useState } from "react";

import {
  CONTRACT_CASES,
  formatContractLogsAsText,
  runKernelContractSuite,
  type ContractCaseResult,
  type ContractCaseStatus,
  type ContractLogEntry,
  type ContractSuiteResult,
} from "@/lib/kernel-contract-suite";
import styles from "./kernel-test-lab.module.css";

interface CaseUiState {
  id: string;
  title: string;
  summary: string;
  status: ContractCaseStatus;
  durationMs: number | null;
  errorMessage: string | null;
}

function formatMs(value: number): string {
  return `${value.toFixed(1)}ms`;
}

function fileSafeStamp(): string {
  const d = new Date();
  const year = d.getFullYear();
  const month = String(d.getMonth() + 1).padStart(2, "0");
  const day = String(d.getDate()).padStart(2, "0");
  const hh = String(d.getHours()).padStart(2, "0");
  const mm = String(d.getMinutes()).padStart(2, "0");
  const ss = String(d.getSeconds()).padStart(2, "0");
  return `${year}-${month}-${day}_${hh}-${mm}-${ss}`;
}

function downloadText(filename: string, text: string): void {
  const blob = new Blob([text], { type: "text/plain;charset=utf-8" });
  const url = URL.createObjectURL(blob);
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = filename;
  anchor.click();
  URL.revokeObjectURL(url);
}

function toInitialCases(): CaseUiState[] {
  return CONTRACT_CASES.map((testCase) => ({
    id: testCase.id,
    title: testCase.title,
    summary: testCase.summary,
    status: "idle",
    durationMs: null,
    errorMessage: null,
  }));
}

function statusLabel(status: ContractCaseStatus): string {
  switch (status) {
    case "idle":
      return "Idle";
    case "running":
      return "Running";
    case "pass":
      return "Pass";
    case "fail":
      return "Fail";
    default:
      return status;
  }
}

export function KernelTestLab() {
  const [cases, setCases] = useState<CaseUiState[]>(() => toInitialCases());
  const [logs, setLogs] = useState<ContractLogEntry[]>([]);
  const [running, setRunning] = useState(false);
  const [summary, setSummary] = useState<ContractSuiteResult | null>(null);
  const [filterText, setFilterText] = useState("");

  const totals = useMemo(() => {
    const pass = cases.filter((testCase) => testCase.status === "pass").length;
    const fail = cases.filter((testCase) => testCase.status === "fail").length;
    const inFlight = cases.filter((testCase) => testCase.status === "running").length;
    return { pass, fail, inFlight };
  }, [cases]);

  const updateCase = useCallback((id: string, updater: (testCase: CaseUiState) => CaseUiState) => {
    setCases((previous) =>
      previous.map((testCase) => (testCase.id === id ? updater(testCase) : testCase)),
    );
  }, []);

  const clearAll = useCallback(() => {
    if (running) return;
    setLogs([]);
    setCases(toInitialCases());
    setSummary(null);
  }, [running]);

  const exportLogs = useCallback(() => {
    const filename = `rusted-geom-contract-suite-${fileSafeStamp()}.log`;
    downloadText(filename, formatContractLogsAsText(logs));
  }, [logs]);

  const runWithFilter = useCallback(async (filterIds?: string[]) => {
    if (running) return;

    setRunning(true);
    setSummary(null);
    setLogs([]);
    if (!filterIds) {
      setCases(toInitialCases());
    }

    try {
      const result = await runKernelContractSuite({
        onCaseStart: (id) => {
          updateCase(id, (testCase) => ({
            ...testCase,
            status: "running",
            durationMs: null,
            errorMessage: null,
          }));
        },
        onCaseEnd: (resultCase: ContractCaseResult) => {
          updateCase(resultCase.id, (testCase) => ({
            ...testCase,
            status: resultCase.status,
            durationMs: resultCase.durationMs,
            errorMessage: resultCase.errorMessage ?? null,
          }));
        },
        onLog: (entry) => {
          setLogs((previous) => {
            const next = [...previous, entry];
            if (next.length > 1200) {
              return next.slice(next.length - 1200);
            }
            return next;
          });
        },
      }, filterIds);
      setSummary(result);
    } finally {
      setRunning(false);
    }
  }, [running, updateCase]);

  const runSuite = useCallback(() => runWithFilter(), [runWithFilter]);

  const runSingle = useCallback((caseId: string) => {
    updateCase(caseId, (tc) => ({ ...tc, status: "idle", durationMs: null, errorMessage: null }));
    void runWithFilter([caseId]);
  }, [runWithFilter, updateCase]);

  const filteredCases = useMemo(() => {
    if (!filterText.trim()) return cases;
    const q = filterText.toLowerCase();
    return cases.filter(
      (c) => c.id.toLowerCase().includes(q) || c.title.toLowerCase().includes(q) || c.summary.toLowerCase().includes(q),
    );
  }, [cases, filterText]);

  return (
    <div className={styles.page}>
      <a href="#test-console" className={styles.skipLink}>
        Skip to Test Console
      </a>

      <div className={styles.shell}>
        <header className={styles.commandBar}>
          <div className={styles.commandBrand}>
            <span className={styles.brandMark} aria-hidden="true">
              ◈
            </span>
            <span className={styles.brandName}>rusted-geom</span>
            <span className={styles.brandBadge}>Test Lab</span>
          </div>

          <div className={styles.heroActions}>
            <button
              type="button"
              className={styles.primaryButton}
              onClick={() => {
                void runSuite();
              }}
              disabled={running}
              aria-label="Run Tests"
            >
              {running ? "Running…" : "Run Tests"}
            </button>
            <button
              type="button"
              className={styles.secondaryButton}
              onClick={clearAll}
              disabled={running}
              aria-label="Clear Logs"
            >
              Clear
            </button>
            <button
              type="button"
              className={styles.secondaryButton}
              onClick={exportLogs}
              aria-label="Export Logs"
            >
              Export Logs
            </button>
            <Link href="/" className={styles.viewerLink} aria-label="Back to Viewer">
              Back to Viewer
            </Link>
          </div>
        </header>

        <section className={styles.hero} aria-label="Runtime contract suite overview">
          <p className={styles.kicker}>Kernel Diagnostics</p>
          <h1 className={styles.title}>Runtime Contract Lab</h1>
          <p className={styles.subtitle}>
            Mirrors the runtime contract assertions and streams pass/fail diagnostics in a viewer-style console.
          </p>
          <p className={styles.heroSummary}>
            {summary
              ? `Last run: ${summary.passed} pass / ${summary.failed} fail in ${formatMs(summary.totalDurationMs)}`
              : "Ready to run all runtime contract cases."}
          </p>
        </section>

        <main className={styles.layout}>
          <section className={styles.panel} aria-label="Contract case status" aria-busy={running}>
            <div className={styles.panelHeader}>
              <h2 className={styles.panelTitle}>Case Status</h2>
              <span className={styles.panelStat}>{cases.length} total</span>
            </div>
            <div className={styles.summaryStrip}>
              <div className={styles.summaryTile}>
                <span>Pass</span>
                <strong>{totals.pass}</strong>
              </div>
              <div className={styles.summaryTile}>
                <span>Fail</span>
                <strong>{totals.fail}</strong>
              </div>
              <div className={styles.summaryTile}>
                <span>Running</span>
                <strong>{totals.inFlight}</strong>
              </div>
              <div className={styles.summaryTile}>
                <span>Logs</span>
                <strong>{logs.length}</strong>
              </div>
            </div>

            <div style={{ padding: "0 16px 8px" }}>
              <input
                type="search"
                placeholder="Filter cases…"
                value={filterText}
                onChange={(e) => setFilterText(e.currentTarget.value)}
                style={{ width: "100%", padding: "6px 8px", fontSize: 13, borderRadius: 4, border: "1px solid var(--border, #333)" }}
                aria-label="Filter test cases"
              />
            </div>

            <ul className={styles.caseList}>
              {filteredCases.map((testCase) => (
                <li key={testCase.id} className={styles.caseItem}>
                  <div className={styles.caseRow}>
                    <h3 className={styles.caseTitle}>{testCase.title}</h3>
                    <div style={{ display: "flex", gap: 4, alignItems: "center" }}>
                      <button
                        type="button"
                        className={styles.secondaryButton}
                        disabled={running}
                        onClick={() => runSingle(testCase.id)}
                        style={{ fontSize: 10, padding: "2px 6px" }}
                        aria-label={`Run ${testCase.title}`}
                      >
                        Run
                      </button>
                      <span className={`${styles.caseBadge} ${styles[`status_${testCase.status}`]}`}>
                        {statusLabel(testCase.status)}
                      </span>
                    </div>
                  </div>
                  <p className={styles.caseSummary}>{testCase.summary}</p>
                  <div className={styles.caseMeta}>
                    <code>{testCase.id}</code>
                    {testCase.durationMs !== null && <span>{formatMs(testCase.durationMs)}</span>}
                  </div>
                  {testCase.errorMessage && <p className={styles.caseError}>{testCase.errorMessage}</p>}
                </li>
              ))}
            </ul>

            {summary && (
              <p className={styles.runSummary}>
                Completed {summary.cases.length} Cases in {formatMs(summary.totalDurationMs)}.
              </p>
            )}
          </section>

          <section
            id="test-console"
            className={styles.consolePanel}
            aria-label="Runtime contract console"
            aria-busy={running}
          >
            <div className={styles.panelHeader}>
              <h2 className={styles.panelTitle}>Console</h2>
              <span className={styles.panelStat}>{running ? "streaming" : "idle"}</span>
            </div>
            <div className={styles.console} role="log" aria-live="polite" aria-relevant="additions text">
              {logs.length === 0 ? (
                <p className={styles.emptyConsole}>No logs yet. Run the suite to stream diagnostics.</p>
              ) : (
                logs.map((entry) => (
                  <p key={entry.id} className={styles.logLine} data-contract-level={entry.level}>
                    <span className={styles.logTime}>[{entry.time}]</span>
                    <span className={styles.logCase}>{entry.caseId}</span>
                    <span className={styles.logLevel}>{entry.level.toUpperCase()}</span>
                    <span className={styles.logMessage}>{entry.message}</span>
                  </p>
                ))
              )}
            </div>
          </section>
        </main>
      </div>
    </div>
  );
}
