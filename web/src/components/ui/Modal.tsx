import React, { useEffect } from "react";
import { createPortal } from "react-dom";
import { Button } from "./Button";
import { CrossIcon } from "./Icons";
import "@/styles/components/modal.css";

interface ModalProps {
  open: boolean;
  title?: string;
  onClose: () => void;
  children: React.ReactNode;
  footer?: React.ReactNode;
  width?: string;
}

export const Modal: React.FC<ModalProps> = ({
  open,
  title,
  onClose,
  children,
  footer,
  width = "max-w-lg",
}) => {
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") onClose();
    };
    if (open) window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [open, onClose]);

  if (!open) return null;

  const modalContentClass = width === "max-w-lg" ? "modal-content" : `modal-content ${width}`;

  return createPortal(
    <div className="modal-overlay">
      <div
        className="modal-backdrop"
        onClick={onClose}
        aria-hidden="true"
      />
      <div className={modalContentClass}>
        {title && (
          <div className="modal-header">
            <div className="modal-title">{title}</div>
            <Button
              variant="ghost"
              size="small-icon"
              aria-label="Close"
              onClick={onClose}
              className="modal-close-button"
            >
              <CrossIcon className="size-6" />
            </Button>
          </div>
        )}
        <div className="modal-body">{children}</div>
        {footer && (
          <div className="modal-footer">{footer}</div>
        )}
      </div>
    </div>,
    document.body
  );
};
