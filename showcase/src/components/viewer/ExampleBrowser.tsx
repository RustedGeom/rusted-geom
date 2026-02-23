"use client";

import { createPortal } from "react-dom";
import { useCallback, useEffect, useRef, useState } from "react";
import { EXAMPLE_CATEGORIES, EXAMPLE_SUMMARIES } from "@/lib/examples";
import type { ExampleKey } from "@/lib/viewer-types";

export interface ExampleBrowserProps {
  isOpen: boolean;
  activeExample: ExampleKey;
  onSelect: (key: ExampleKey) => void;
  onClose: () => void;
}

export function ExampleBrowser({ isOpen, activeExample, onSelect, onClose }: ExampleBrowserProps) {
  const [query, setQuery] = useState("");
  const inputRef = useRef<HTMLInputElement | null>(null);
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    setMounted(true);
  }, []);

  useEffect(() => {
    if (isOpen) {
      setQuery("");
      requestAnimationFrame(() => inputRef.current?.focus());
    }
  }, [isOpen]);

  useEffect(() => {
    if (!isOpen) return;
    function onKey(event: KeyboardEvent) {
      if (event.key === "Escape") {
        event.preventDefault();
        onClose();
      }
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [isOpen, onClose]);

  const handleSelect = useCallback(
    (key: ExampleKey) => {
      onSelect(key);
      onClose();
    },
    [onSelect, onClose],
  );

  if (!mounted || !isOpen) return null;

  const normalizedQuery = query.trim().toLowerCase();

  const content = (
    <div className="example-browser-backdrop" onClick={onClose} role="presentation">
      <div
        className="example-browser"
        onClick={(e) => e.stopPropagation()}
        role="dialog"
        aria-label="Browse examples"
        aria-modal="true"
      >
        <div className="example-browser-search">
          <svg
            className="example-browser-search-icon"
            viewBox="0 0 16 16"
            width="14"
            height="14"
            aria-hidden="true"
          >
            <circle cx="7.2" cy="7.2" r="4.2" />
            <path d="m10.4 10.4 3 3" />
          </svg>
          <input
            ref={inputRef}
            type="text"
            placeholder="Search examples…"
            value={query}
            onChange={(e) => setQuery(e.currentTarget.value)}
            className="example-browser-input"
            aria-label="Search examples"
          />
          {query && (
            <button
              type="button"
              className="example-browser-clear"
              onClick={() => setQuery("")}
              aria-label="Clear search"
            >
              ×
            </button>
          )}
        </div>

        <div className="example-browser-list">
          {normalizedQuery ? (
            renderFiltered(normalizedQuery, activeExample, handleSelect)
          ) : (
            renderCategorized(activeExample, handleSelect)
          )}
        </div>

        <div className="example-browser-hint">
          <span><kbd>↵</kbd> select</span>
          <span><kbd>esc</kbd> close</span>
          <span><kbd>⌘K</kbd> toggle</span>
        </div>
      </div>
    </div>
  );

  return createPortal(content, document.body);
}

function renderFiltered(
  query: string,
  activeExample: ExampleKey,
  onSelect: (key: ExampleKey) => void,
) {
  const results: Array<{ key: ExampleKey; label: string; category: string }> = [];
  for (const cat of EXAMPLE_CATEGORIES) {
    for (const item of cat.items) {
      const summary = EXAMPLE_SUMMARIES[item.key].toLowerCase();
      if (item.label.toLowerCase().includes(query) || summary.includes(query) || cat.label.toLowerCase().includes(query)) {
        results.push({ key: item.key, label: item.label, category: cat.label });
      }
    }
  }

  if (results.length === 0) {
    return (
      <div className="example-browser-empty">No examples match &ldquo;{query}&rdquo;</div>
    );
  }

  return results.map((result) => (
    <ExampleItem
      key={result.key}
      exampleKey={result.key}
      label={result.label}
      summary={EXAMPLE_SUMMARIES[result.key]}
      category={result.category}
      isActive={result.key === activeExample}
      onSelect={onSelect}
    />
  ));
}

function renderCategorized(
  activeExample: ExampleKey,
  onSelect: (key: ExampleKey) => void,
) {
  return EXAMPLE_CATEGORIES.map((cat) => (
    <div key={cat.key} className="example-browser-category">
      <div className="example-browser-category-header">{cat.label}</div>
      {cat.items.map((item) => (
        <ExampleItem
          key={item.key}
          exampleKey={item.key}
          label={item.label}
          summary={EXAMPLE_SUMMARIES[item.key]}
          isActive={item.key === activeExample}
          onSelect={onSelect}
        />
      ))}
    </div>
  ));
}

interface ExampleItemProps {
  exampleKey: ExampleKey;
  label: string;
  summary: string;
  category?: string;
  isActive: boolean;
  onSelect: (key: ExampleKey) => void;
}

function ExampleItem({ exampleKey, label, summary, category, isActive, onSelect }: ExampleItemProps) {
  return (
    <button
      type="button"
      className={`example-browser-item ${isActive ? "is-active" : ""}`}
      onClick={() => onSelect(exampleKey)}
    >
      <span className="example-browser-item-title">{label}</span>
      <span className="example-browser-item-summary">{summary}</span>
      {category && (
        <span className="example-browser-item-badge">{category}</span>
      )}
    </button>
  );
}
