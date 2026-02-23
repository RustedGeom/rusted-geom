import { useCallback, useState } from "react";
import type { LogEntry } from "@/lib/viewer-types";

interface ConsoleLogEntryProps {
  entry: LogEntry;
}

export function ConsoleLogEntry({ entry }: ConsoleLogEntryProps) {
  const [copied, setCopied] = useState(false);

  const handleClick = useCallback(() => {
    navigator.clipboard.writeText(entry.message).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 1200);
    }).catch(() => {});
  }, [entry.message]);

  return (
    <div
      className={`kernel-log kernel-log-${entry.level} ${copied ? "is-copied" : ""}`}
      onClick={handleClick}
      title="Click to copy"
      role="button"
      tabIndex={-1}
      onKeyDown={(e) => { if (e.key === "Enter") handleClick(); }}
    >
      <span className="kernel-log-time">{entry.time}</span>
      <span className="kernel-log-level">{entry.level.toUpperCase()}</span>
      <span className="kernel-log-message">{entry.message}</span>
      {copied && <span className="kernel-log-copied-tip">Copied</span>}
    </div>
  );
}
