import React from "react";
import clsx from "clsx";
import "@/styles/components/list.css";

interface ListProps {
  children: React.ReactNode;
  className?: string;
}

export const List: React.FC<ListProps> = ({ children, className }) => {
  return <div className={clsx("list", className)}>{children}</div>;
};
