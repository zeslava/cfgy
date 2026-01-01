import React from "react";
import clsx from "clsx";
import { EyeIcon } from "./Icons";
import "@/styles/components/input.css";

export type InputState = "default" | "focus" | "disabled" | "error" | "success";

interface InputProps extends React.InputHTMLAttributes<HTMLInputElement> {
  state?: InputState;
  mode?: "any" | "digits";
  label?: string;
  description?: string;
  leftIcon?: React.ReactNode;
  rightIcon?: React.ReactNode;
  showPasswordToggle?: boolean;
  onPasswordToggle?: (show: boolean) => void;
}

export const Input: React.FC<InputProps> = ({
  type = "text",
  state = "default",
  mode = "any",
  label,
  description,
  leftIcon,
  rightIcon,
  showPasswordToggle = false,
  onPasswordToggle,
  className,
  disabled,
  ...rest
}) => {
  const [showPassword, setShowPassword] = React.useState(false);
  const [isFocused, setIsFocused] = React.useState(false);
  const isPassword = type === "password";

  const handlePasswordToggle = () => {
    const newValue = !showPassword;
    setShowPassword(newValue);
    onPasswordToggle?.(newValue);
  };

  const inputType = isPassword && showPassword ? "text" : type;

  const isDisabled = disabled || state === "disabled";

  const { onChange, onFocus, onBlur, ...inputProps } = rest;

  // Определяем класс контейнера в зависимости от состояния
  const getContainerClass = () => {
    if (state === "error") return "input-container-error";
    if (state === "success") return "input-container-success";
    if (isDisabled) return "input-container-disabled";
    if (!isDisabled && (state === "focus" || isFocused))
      return "input-container-focus";
    return "input-container-default";
  };

  const handleFocus = (e: React.FocusEvent<HTMLInputElement>) => {
    setIsFocused(true);
    onFocus?.(e);
  };

  const handleBlur = (e: React.FocusEvent<HTMLInputElement>) => {
    setIsFocused(false);
    onBlur?.(e);
  };

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (mode === "digits") {
      const normalized = e.target.value.replace(/\D/g, "");
      if (normalized !== e.target.value) {
        e.target.value = normalized;
      }
    }
    onChange?.(e);
  };

  const getTextColor = () => {
    if (isDisabled) return "input-field-disabled";
    return "input-field-default";
  };

  const getDescriptionColor = () => {
    if (state === "error") return "input-description-error";
    if (state === "success") return "input-description-success";
    return "input-description-default";
  };

  const getIconColor = () => {
    if (state === "error") return "input-icon-error";
    if (state === "success") return "input-icon-success";
    return "input-icon-default";
  };

  return (
    <div className={clsx("input-wrapper", className)}>
      {label && <label className="input-label">{label}</label>}
      <div className={clsx("input-container", getContainerClass())}>
        {leftIcon && (
          <span className={clsx("input-icon", getIconColor())}>{leftIcon}</span>
        )}
        <input
          {...inputProps}
          type={inputType}
          inputMode={mode === "digits" ? "numeric" : undefined}
          pattern={mode === "digits" ? "\\d*" : undefined}
          className={clsx("input-field", getTextColor())}
          disabled={isDisabled}
          onChange={handleChange}
          onFocus={handleFocus}
          onBlur={handleBlur}
        />
        {showPasswordToggle && isPassword && (
          <button
            type="button"
            onClick={handlePasswordToggle}
            className="input-toggle-button"
            disabled={isDisabled}
          >
            <EyeIcon
              className={clsx("input-icon", getIconColor())}
              open={showPassword}
            />
          </button>
        )}
        {rightIcon && !showPasswordToggle && (
          <span className={clsx("input-icon", getIconColor())}>
            {rightIcon}
          </span>
        )}
      </div>
      {description && (
        <p className={clsx("input-description", getDescriptionColor())}>
          {description}
        </p>
      )}
    </div>
  );
};
