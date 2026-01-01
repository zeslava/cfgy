import React, { useRef, useEffect, useState } from "react";
import clsx from "clsx";
import "@/styles/components/list-item.css";

interface ListItemProps {
  children: React.ReactNode;
  showDivider?: boolean;
  onClick?: () => void;
  className?: string;
  isEditing?: boolean;
}

export const ListItem: React.FC<ListItemProps> = ({
  children,
  showDivider = true,
  onClick,
  className,
  isEditing = false,
}) => {
  const headerRef = useRef<HTMLDivElement>(null);
  const [isClickable, setIsClickable] = useState(!!onClick);

  useEffect(() => {
    if (headerRef.current && !isEditing) {
      const hasLink = !!headerRef.current.querySelector("a");
      setIsClickable(!!onClick || hasLink);
    } else {
      setIsClickable(false);
    }
  }, [children, onClick, isEditing]);

  const handleHeaderClick = (e: React.MouseEvent<HTMLDivElement>) => {
    // В режиме редактирования не обрабатываем клики по строке
    if (isEditing) {
      return;
    }

    // Если клик был на кнопке, не обрабатываем
    const target = e.target as HTMLElement;
    if (target.closest("button")) {
      return;
    }

    // Если клик был на ссылке, позволяем ей обработать клик
    if (target.closest("a")) {
      return;
    }

    // Если есть кастомный onClick, вызываем его
    if (onClick) {
      onClick();
      return;
    }

    // Ищем первую ссылку внутри header и кликаем на неё
    if (headerRef.current) {
      const link = headerRef.current.querySelector("a");
      if (link) {
        link.click();
      }
    }
  };

  return (
    <div
      className={clsx("list-item", className, isEditing && "list-item-editing")}
    >
      <div
        ref={headerRef}
        className={clsx(
          "list-item-header",
          isClickable && !isEditing && "list-item-clickable",
          isEditing && "list-item-header-editing"
        )}
        onClick={handleHeaderClick}
      >
        {children}
      </div>
      {showDivider && <div className="list-item-divider" />}
    </div>
  );
};
