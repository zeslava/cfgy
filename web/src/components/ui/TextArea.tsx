import React from "react";
import clsx from "clsx";

export type TextAreaState = "default" | "focus" | "disabled" | "error" | "success";

interface TextAreaProps
  extends React.TextareaHTMLAttributes<HTMLTextAreaElement> {
  state?: TextAreaState;
  label?: string;
  description?: string;
  invalid?: boolean; // Для обратной совместимости
}

export const TextArea: React.FC<TextAreaProps> = ({
  state,
  invalid,
  label,
  description,
  className,
  disabled,
  ...rest
}) => {
  const [isFocused, setIsFocused] = React.useState(false);
  const effectiveState = invalid ? "error" : (state || "default");
  const isDisabled = disabled || effectiveState === "disabled";

  const getBorderColor = () => {
    if (effectiveState === "error") return "border-states-error";
    if (effectiveState === "success") return "border-states-success";
    if (isFocused && !isDisabled) return "border-basic-accent";
    if (isDisabled) return "border-lines-gray";
    return "border-lines-gray";
  };

  const getBgColor = () => {
    if (isDisabled) return "bg-states-disable";
    return "bg-background-light";
  };

  const getTextColor = () => {
    if (isDisabled) return "text-basic-gray";
    return "text-text-dark";
  };

  const getDescriptionColor = () => {
    if (effectiveState === "error") return "text-states-error";
    if (effectiveState === "success") return "text-states-success";
    return "text-basic-gray";
  };

  const handleFocus = (e: React.FocusEvent<HTMLTextAreaElement>) => {
    setIsFocused(true);
    rest.onFocus?.(e);
  };

  const handleBlur = (e: React.FocusEvent<HTMLTextAreaElement>) => {
    setIsFocused(false);
    rest.onBlur?.(e);
  };

  return (
    <div className={clsx("flex flex-col gap-1", className)}>
      {label && (
        <label className="font-inter font-normal text-regular-24 text-text-dark">
          {label}
        </label>
      )}
      <textarea
        {...rest}
        className={clsx(
          "w-full px-4 py-2 rounded border font-inter font-normal text-regular-24 outline-none placeholder:text-basic-gray transition resize-y",
          getBorderColor(),
          getBgColor(),
          getTextColor()
        )}
        disabled={isDisabled}
        onFocus={handleFocus}
        onBlur={handleBlur}
      />
      {description && (
        <p className={clsx("font-inter font-normal text-regular-16", getDescriptionColor())}>
          {description}
        </p>
      )}
    </div>
  );
};
