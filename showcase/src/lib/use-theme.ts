"use client";

import { useCallback, useEffect, useState } from "react";

const STORAGE_KEY = "rgm-theme";

function getInitialTheme(): boolean {
  if (typeof window === "undefined") return false;
  const stored = localStorage.getItem(STORAGE_KEY);
  if (stored === "dark") return true;
  if (stored === "light") return false;
  return window.matchMedia("(prefers-color-scheme: dark)").matches;
}

function applyTheme(isDark: boolean): void {
  document.documentElement.setAttribute("data-theme", isDark ? "dark" : "light");
}

export function useTheme(): { isDarkMode: boolean; toggleDarkMode: () => void } {
  const [isDarkMode, setIsDarkMode] = useState(false);

  useEffect(() => {
    const initial = getInitialTheme();
    setIsDarkMode(initial);
    applyTheme(initial);
  }, []);

  const toggleDarkMode = useCallback(() => {
    setIsDarkMode((prev) => {
      const next = !prev;
      applyTheme(next);
      localStorage.setItem(STORAGE_KEY, next ? "dark" : "light");
      return next;
    });
  }, []);

  return { isDarkMode, toggleDarkMode };
}
