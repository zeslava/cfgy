import React from "react";
import { Modal } from "./Modal";
import { Button } from "./Button";

interface ConfirmDialogProps {
  open: boolean;
  title?: string;
  message: string;
  confirmLabel?: string;
  cancelLabel?: string;
  variant?: "danger" | "primary";
  onConfirm: () => void;
  onCancel: () => void;
}

export const ConfirmDialog: React.FC<ConfirmDialogProps> = ({
  open,
  title = "Подтверждение",
  message,
  confirmLabel = "Подтвердить",
  cancelLabel = "Отмена",
  variant = "primary",
  onConfirm,
  onCancel,
}) => {
  return (
    <Modal
      open={open}
      title={title}
      onClose={onCancel}
      width="max-w-md"
      footer={
        <div className="flex gap-2 justify-end">
          <Button variant="outline" size="sm" onClick={onCancel}>
            {cancelLabel}
          </Button>
          <Button variant={variant} size="sm" onClick={onConfirm}>
            {confirmLabel}
          </Button>
        </div>
      }
    >
      <p className="text-sm text-gray-700">{message}</p>
    </Modal>
  );
};
