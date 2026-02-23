import type { ReactNode } from "react";

interface ToolButtonProps {
  children: ReactNode;
  isActive?: boolean;
  disabled?: boolean;
  onClick?: () => void;
  ariaLabel?: string;
  title?: string;
  className?: string;
}

export function ToolButton({
  children,
  isActive = false,
  disabled = false,
  onClick,
  ariaLabel,
  title,
  className = "",
}: ToolButtonProps) {
  return (
    <button
      type="button"
      className={`tool-btn ${isActive ? "is-active" : ""} ${className}`.trim()}
      disabled={disabled}
      onClick={onClick}
      aria-label={ariaLabel}
      title={title}
    >
      {children}
    </button>
  );
}
