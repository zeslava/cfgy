import React from "react";
import clsx from "clsx";
import "@/styles/components/toggle.css";

interface ToggleProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
  disabled?: boolean;
  className?: string;
  "aria-label"?: string;
}

export const Toggle: React.FC<ToggleProps> = ({
  checked,
  onChange,
  disabled = false,
  className,
  "aria-label": ariaLabel,
}) => {
  const handleClick = () => {
    if (!disabled) {
      onChange(!checked);
    }
  };

  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      aria-label={ariaLabel}
      disabled={disabled}
      onClick={handleClick}
      className={clsx(
        "toggle",
        checked && "toggle--checked",
        disabled && "toggle--disabled",
        className
      )}
    >
      <span className="toggle-track">
        <span className="toggle-thumb" />
      </span>
    </button>
  );
};
