import React from "react";
import clsx from "clsx";
import "@/styles/components/button.css";

export type ButtonVariant =
  | "primary"
  | "secondary"
  | "danger"
  | "outline"
  | "ghost"
  | "basic-secondary"
  | "basic-accent"
  | "basic-gray";
export type ButtonSize =
  | "big-text"
  | "medium-text"
  | "small-text"
  | "small-icon"
  | "sm"
  | "md"
  | "lg";
export type ButtonState =
  | "default"
  | "hover"
  | "active"
  | "pressed"
  | "disabled";

interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  size?: ButtonSize;
  state?: ButtonState;
  icon?: React.ReactNode;
  fullWidth?: boolean;
}

const getVariantClasses = (variant: ButtonVariant, state: ButtonState) => {
  // Обратная совместимость со старыми вариантами
  if (variant === "danger") {
    switch (state) {
      case "default":
        return "button-danger-default";
      case "hover":
        return "button-danger-hover";
      case "active":
        return "button-danger-active";
      case "pressed":
        return "button-danger-pressed";
      case "disabled":
        return "button-danger-disabled";
      default:
        return "button-danger-default";
    }
  }
  if (variant === "outline") {
    switch (state) {
      case "default":
        return "button-outline-default";
      case "hover":
        return "button-outline-hover";
      case "active":
        return "button-outline-active";
      case "pressed":
        return "button-outline-pressed";
      case "disabled":
        return "button-outline-disabled";
      default:
        return "button-outline-default";
    }
  }
  if (variant === "ghost") {
    switch (state) {
      case "default":
        return "button-ghost-default";
      case "hover":
        return "button-ghost-hover";
      case "active":
        return "button-ghost-active";
      case "pressed":
        return "button-ghost-pressed";
      case "disabled":
        return "button-ghost-disabled";
      default:
        return "button-ghost-default";
    }
  }
  // Primary кнопки (акцентные) - согласно дизайну Figma с семантическими токенами
  if (variant === "primary" || variant === "basic-accent") {
    switch (state) {
      case "default":
        return "button-primary-default";
      case "hover":
        return "button-primary-hover";
      case "active":
        return "button-primary-active";
      case "pressed":
        return "button-primary-pressed";
      case "disabled":
        return "button-primary-disabled";
      default:
        return "button-primary-default";
    }
  } else if (variant === "basic-secondary" || variant === "secondary") {
    // Secondary кнопки - согласно дизайну Figma с семантическими токенами
    switch (state) {
      case "default":
        return "button-secondary-default";
      case "hover":
        return "button-secondary-hover";
      case "active":
        return "button-secondary-active";
      case "pressed":
        return "button-secondary-pressed";
      case "disabled":
        return "button-secondary-disabled";
      default:
        return "button-secondary-default";
    }
  } else if (variant === "basic-gray") {
    switch (state) {
      case "default":
        return "button-basic-gray-default";
      case "hover":
        return "button-basic-gray-hover";
      case "active":
        return "button-basic-gray-active";
      case "pressed":
        return "button-basic-gray-pressed";
      case "disabled":
        return "button-basic-gray-disabled";
      default:
        return "button-basic-gray-default";
    }
  } else {
    // secondary (по умолчанию) - согласно дизайну Figma
    switch (state) {
      case "default":
        return "button-secondary-default";
      case "hover":
        return "button-secondary-hover";
      case "active":
        return "button-secondary-active";
      case "pressed":
        return "button-secondary-pressed";
      case "disabled":
        return "button-secondary-disabled";
      default:
        return "button-secondary-default";
    }
  }
};

const getSizeClasses = (size: ButtonSize) => {
  switch (size) {
    case "big-text":
      return "button-big-text";
    case "medium-text":
      return "button-medium-text";
    case "small-text":
      return "button-small-text";
    case "small-icon":
      return "button-small-icon";
    case "sm":
      return "button-sm";
    case "md":
      return "button-md";
    case "lg":
      return "button-lg";
    default:
      return "button-big-text";
  }
};

export const Button: React.FC<ButtonProps> = ({
  variant = "primary",
  size = "big-text",
  state = "default",
  icon,
  fullWidth,
  className,
  children,
  disabled,
  ...rest
}) => {
  const effectiveState = disabled ? "disabled" : state;

  return (
    <button
      className={clsx(
        "button",
        getVariantClasses(variant, effectiveState),
        getSizeClasses(size),
        fullWidth && "button-full-width",
        className
      )}
      disabled={disabled || effectiveState === "disabled"}
      {...rest}
    >
      {size === "small-icon" ? (
        icon || children
      ) : (
        <>
          {icon && <span className="shrink-0">{icon}</span>}
          {children}
        </>
      )}
    </button>
  );
};
