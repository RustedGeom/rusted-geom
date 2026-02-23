"use client";

import { useEffect, useRef } from "react";

type KeyHandler = (event: KeyboardEvent) => void;

export interface KeyboardShortcut {
  key: string;
  meta?: boolean;
  ctrl?: boolean;
  shift?: boolean;
  /** If true, fires even when focus is in an input/textarea */
  allowInInput?: boolean;
  handler: KeyHandler;
}

function isInputFocused(): boolean {
  const el = document.activeElement;
  if (!el) return false;
  const tag = el.tagName.toLowerCase();
  return tag === "input" || tag === "textarea" || (el as HTMLElement).isContentEditable;
}

export function useKeyboardShortcuts(shortcuts: KeyboardShortcut[]): void {
  const shortcutsRef = useRef(shortcuts);
  shortcutsRef.current = shortcuts;

  useEffect(() => {
    function onKeyDown(event: KeyboardEvent): void {
      for (const shortcut of shortcutsRef.current) {
        if (!shortcut.allowInInput && isInputFocused()) continue;

        const keyMatch = event.key.toLowerCase() === shortcut.key.toLowerCase();
        const metaMatch = shortcut.meta ? event.metaKey || event.ctrlKey : true;
        const ctrlMatch = shortcut.ctrl ? event.ctrlKey : true;
        const shiftMatch = shortcut.shift ? event.shiftKey : true;
        const noUnwantedMeta =
          !shortcut.meta && !shortcut.ctrl ? !event.metaKey && !event.ctrlKey : true;

        if (keyMatch && metaMatch && ctrlMatch && shiftMatch && noUnwantedMeta) {
          event.preventDefault();
          shortcut.handler(event);
          return;
        }
      }
    }

    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, []); // Stable: uses ref to always read latest shortcuts
}
