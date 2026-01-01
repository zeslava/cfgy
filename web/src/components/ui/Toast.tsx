import React, { useEffect } from "react";
import clsx from "clsx";
import { CrossIcon } from "./Icons";
import { CheckIcon } from "./Icons";
import "@/styles/components/toast.css";

export type ToastVariant = "success" | "error" | "info" | "warning";

interface ToastProps {
  id: string;
  message: string;
  variant?: ToastVariant;
  duration?: number;
  onClose: (id: string) => void;
}

const variantStyles: Record<ToastVariant, string> = {
  success: "toast-success",
  error: "toast-error",
  info: "toast-info",
  warning: "toast-warning",
};

const variantIconColors: Record<ToastVariant, string> = {
  success: "toast-icon-success",
  error: "toast-icon-error",
  info: "toast-icon-info",
  warning: "toast-icon-warning",
};

export const Toast: React.FC<ToastProps> = ({
  id,
  message,
  variant = "info",
  duration = 5000,
  onClose,
}) => {
  useEffect(() => {
    if (duration > 0) {
      const timer = setTimeout(() => onClose(id), duration);
      return () => clearTimeout(timer);
    }
  }, [id, duration, onClose]);

  return (
    <div
      className={clsx("toast", variantStyles[variant])}
      role="alert"
    >
      {variant === "success" && (
        <CheckIcon className={clsx("toast-icon", variantIconColors[variant])} />
      )}
      {variant === "error" && (
        <CrossIcon className={clsx("toast-icon", variantIconColors[variant])} />
      )}
      {variant === "info" && (
        <span className={clsx("toast-icon", variantIconColors[variant])} style={{ fontSize: '20px' }}>ℹ</span>
      )}
      {variant === "warning" && (
        <span className={clsx("toast-icon", variantIconColors[variant])} style={{ fontSize: '20px' }}>⚠</span>
      )}
      <p className="toast-message">{message}</p>
      <button
        onClick={() => onClose(id)}
        className="toast-close-button"
        aria-label="Close"
      >
        <CrossIcon className="toast-close-icon" />
      </button>
    </div>
  );
};

interface ToastContainerProps {
  toasts: Array<{
    id: string;
    message: string;
    variant?: ToastVariant;
    duration?: number;
  }>;
  onClose: (id: string) => void;
}

export const ToastContainer: React.FC<ToastContainerProps> = ({
  toasts,
  onClose,
}) => {
  return (
    <div className="toast-container">
      {toasts.map((toast) => (
        <Toast key={toast.id} {...toast} onClose={onClose} />
      ))}
    </div>
  );
};
