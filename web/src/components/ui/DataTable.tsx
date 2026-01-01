// Компонент таблицы данных с заголовком и строками
import React from "react";
import clsx from "clsx";
import "@/styles/components/data-table.css";

// === DataTable ===
interface DataTableProps {
  children: React.ReactNode;
  className?: string;
}

export const DataTable: React.FC<DataTableProps> = ({
  children,
  className,
}) => {
  return <div className={clsx("data-table", className)}>{children}</div>;
};

// === DataTableHeader ===
interface DataTableHeaderProps {
  children: React.ReactNode;
  className?: string;
}

export const DataTableHeader: React.FC<DataTableHeaderProps> = ({
  children,
  className,
}) => {
  return <div className={clsx("data-table-header", className)}>{children}</div>;
};

// === DataTableHeaderCell ===
interface DataTableHeaderCellProps {
  children: React.ReactNode;
  className?: string;
  align?: "left" | "center" | "right";
  width?: string | number;
}

export const DataTableHeaderCell: React.FC<DataTableHeaderCellProps> = ({
  children,
  className,
  align = "left",
  width,
}) => {
  const style: React.CSSProperties = {};
  if (width) {
    style.width = typeof width === "number" ? `${width}px` : width;
  }

  return (
    <div
      className={clsx(
        "data-table-header-cell",
        `data-table-header-cell--${align}`,
        className
      )}
      style={style}
    >
      {children}
    </div>
  );
};

// === DataTableBody ===
interface DataTableBodyProps {
  children: React.ReactNode;
  className?: string;
}

export const DataTableBody: React.FC<DataTableBodyProps> = ({
  children,
  className,
}) => {
  return <div className={clsx("data-table-body", className)}>{children}</div>;
};

// === DataTableRow ===
interface DataTableRowProps {
  children: React.ReactNode;
  className?: string;
  isEditing?: boolean;
  onClick?: () => void;
}

export const DataTableRow: React.FC<DataTableRowProps> = ({
  children,
  className,
  isEditing = false,
  onClick,
}) => {
  return (
    <div
      className={clsx(
        "data-table-row",
        isEditing && "data-table-row--editing",
        onClick && "data-table-row--clickable",
        className
      )}
      onClick={onClick}
    >
      <div className="data-table-row-content">{children}</div>
      <div className="data-table-row-divider" />
    </div>
  );
};

// === DataTableCell ===
interface DataTableCellProps {
  children: React.ReactNode;
  className?: string;
  align?: "left" | "center" | "right";
  width?: string | number;
}

export const DataTableCell: React.FC<DataTableCellProps> = ({
  children,
  className,
  align = "left",
  width,
}) => {
  const style: React.CSSProperties = {};
  if (width) {
    style.width = typeof width === "number" ? `${width}px` : width;
  }

  return (
    <div
      className={clsx(
        "data-table-cell",
        `data-table-cell--${align}`,
        className
      )}
      style={style}
    >
      {children}
    </div>
  );
};

// === DataTableActions ===
interface DataTableActionsProps {
  children: React.ReactNode;
  className?: string;
}

export const DataTableActions: React.FC<DataTableActionsProps> = ({
  children,
  className,
}) => {
  return (
    <div className={clsx("data-table-actions", className)}>{children}</div>
  );
};

// === DataTableEmpty ===
interface DataTableEmptyProps {
  children: React.ReactNode;
  className?: string;
}

export const DataTableEmpty: React.FC<DataTableEmptyProps> = ({
  children,
  className,
}) => {
  return <div className={clsx("data-table-empty", className)}>{children}</div>;
};
