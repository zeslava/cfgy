import React, { createContext, useContext, useState, useCallback } from "react";
import { ToastContainer } from "../components/ui/Toast";
import type { ToastVariant } from "../components/ui/Toast";

interface Toast {
  id: string;
  message: string;
  variant?: ToastVariant;
  duration?: number;
}

interface ToastContextValue {
  showToast: (
    message: string,
    variant?: ToastVariant,
    duration?: number
  ) => void;
  showSuccess: (message: string) => void;
  showError: (message: string) => void;
  showInfo: (message: string) => void;
  showWarning: (message: string) => void;
}

const ToastContext = createContext<ToastContextValue | undefined>(undefined);

export const ToastProvider: React.FC<{ children: React.ReactNode }> = ({
  children,
}) => {
  const [toasts, setToasts] = useState<Toast[]>([]);

  const removeToast = useCallback((id: string) => {
    setToasts((prev) => prev.filter((t) => t.id !== id));
  }, []);

  const showToast = useCallback(
    (message: string, variant: ToastVariant = "info", duration = 5000) => {
      const id = `toast-${Date.now()}-${Math.random()}`;
      setToasts((prev) => [...prev, { id, message, variant, duration }]);
    },
    []
  );

  const showSuccess = useCallback(
    (message: string) => showToast(message, "success"),
    [showToast]
  );
  const showError = useCallback(
    (message: string) => showToast(message, "error"),
    [showToast]
  );
  const showInfo = useCallback(
    (message: string) => showToast(message, "info"),
    [showToast]
  );
  const showWarning = useCallback(
    (message: string) => showToast(message, "warning"),
    [showToast]
  );

  return (
    <ToastContext.Provider
      value={{ showToast, showSuccess, showError, showInfo, showWarning }}
    >
      {children}
      <ToastContainer toasts={toasts} onClose={removeToast} />
    </ToastContext.Provider>
  );
};

export const useToast = (): ToastContextValue => {
  const context = useContext(ToastContext);
  if (!context) {
    throw new Error("useToast must be used within ToastProvider");
  }
  return context;
};
