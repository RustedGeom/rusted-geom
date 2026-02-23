import type { ReactNode } from "react";

interface ToolIconProps {
  children: ReactNode;
  size?: number;
}

export function ToolIcon({ children, size = 14 }: ToolIconProps) {
  return (
    <svg
      className="tool-icon"
      viewBox="0 0 16 16"
      width={size}
      height={size}
      aria-hidden="true"
      focusable="false"
    >
      {children}
    </svg>
  );
}
