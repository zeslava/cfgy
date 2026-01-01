import React, { useState } from "react";
import { EyeIcon } from "../../ui/Icons";

interface ParameterValueMaskedProps {
  value: string;
  parameterKey?: string; // Имя параметра для автоматического определения секретных
  monospace?: boolean;
  onCopy?: (value: string) => void;
  masked?: boolean; // Если true, значение маскируется по умолчанию
}

// Автоматически определяем, является ли параметр секретным по его имени
const isSecretParameter = (parameterKey?: string): boolean => {
  if (!parameterKey) return false;
  const lowerKey = parameterKey.toLowerCase();
  return (
    lowerKey.includes("password") ||
    lowerKey.includes("secret") ||
    lowerKey.includes("token") ||
    lowerKey.includes("key") ||
    lowerKey.includes("api_key") ||
    lowerKey.includes("private")
  );
};

export const ParameterValueMasked: React.FC<ParameterValueMaskedProps> = ({
  value,
  parameterKey,
  monospace = true,
  onCopy: _onCopy, // Пока не используется, но может понадобиться в будущем
  masked,
}) => {
  // Автоматически определяем, нужно ли маскировать
  const shouldMask =
    masked !== undefined ? masked : isSecretParameter(parameterKey);
  const [visible, setVisible] = useState(!shouldMask);

  // Если не маскированное значение, просто показываем текст
  if (!shouldMask) {
    return <span className={monospace ? "font-mono" : ""}>{value}</span>;
  }

  return (
    <span className="inline-flex items-center gap-[10px]">
      <span className={monospace ? "font-mono" : ""}>
        {visible ? value : "****"}
      </span>
      <button
        type="button"
        onClick={() => setVisible((v) => !v)}
        className="shrink-0 w-6 h-6 flex items-center justify-center"
        aria-label={visible ? "Скрыть" : "Показать"}
      >
        <EyeIcon className="w-6 h-6 text-text-dark" open={visible} />
      </button>
    </span>
  );
};
