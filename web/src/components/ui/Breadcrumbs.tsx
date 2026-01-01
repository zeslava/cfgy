import React from "react";
import { Link } from "react-router-dom";
import clsx from "clsx";

interface BreadcrumbItem {
  label: string;
  to?: string;
  onClick?: () => void;
}

interface BreadcrumbsProps {
  items: BreadcrumbItem[];
  className?: string;
}

export const Breadcrumbs: React.FC<BreadcrumbsProps> = ({
  items,
  className,
}) => {
  return (
    <div className={clsx("flex items-center flex-nowrap", className)}>
      {items.map((item, index) => {
        const isLast = index === items.length - 1;
        const isClickable = !!(item.to || item.onClick);

        // Активные (кликабельные) элементы - светло-синий цвет
        // Неактивные (текущая страница) - светло-серый цвет
        const itemColorClass = isClickable
          ? "text-lines-accent"
          : "text-basic-gray";

        // Разделитель использует тот же цвет, что и текущий элемент (предшествующий разделителю)
        const separatorColorClass = itemColorClass;

        return (
          <React.Fragment key={index}>
            {item.to ? (
              <Link
                to={item.to}
                className={clsx(
                  "font-inter font-normal text-regular-16 whitespace-nowrap",
                  itemColorClass
                )}
                style={{ textDecoration: "none" }}
              >
                {item.label}
              </Link>
            ) : item.onClick ? (
              <button
                onClick={item.onClick}
                className={clsx(
                  "font-inter font-normal text-regular-16 whitespace-nowrap",
                  itemColorClass
                )}
                style={{ textDecoration: "none" }}
              >
                {item.label}
              </button>
            ) : (
              <span
                className={clsx(
                  "font-inter font-normal text-regular-16 whitespace-nowrap",
                  itemColorClass
                )}
              >
                {item.label}
              </span>
            )}
            {!isLast && (
              <span
                className={clsx(
                  "font-inter font-normal text-regular-16 whitespace-nowrap",
                  separatorColorClass
                )}
                style={{
                  marginLeft: "var(--spacing-2, 8px)",
                  marginRight: "var(--spacing-2, 8px)",
                }}
              >
                /
              </span>
            )}
          </React.Fragment>
        );
      })}
    </div>
  );
};
