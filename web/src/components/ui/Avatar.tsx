import React from "react";
import clsx from "clsx";
import "@/styles/components/avatar.css";

export type AvatarSize = "small" | "big";

interface AvatarProps {
  size?: AvatarSize;
  src?: string;
  alt?: string;
  initials?: string;
  className?: string;
}

export const Avatar: React.FC<AvatarProps> = ({
  size = "small",
  src,
  alt = "Avatar",
  initials,
  className,
}) => {
  return (
    <div
      className={clsx(
        "avatar",
        size === "small" ? "avatar-small" : "avatar-big",
        className
      )}
    >
      {src ? (
        <img
          src={src}
          alt={alt}
          className="absolute inset-0 max-w-none object-cover pointer-events-none rounded-[360px] size-full"
          style={{ objectPosition: "50% 50%" }}
        />
      ) : initials ? (
        <span
          className={clsx(
            "avatar-initials",
            size === "small" ? "avatar-initials-small" : "avatar-initials-big"
          )}
        >
          {initials}
        </span>
      ) : (
        <div className="w-full h-full bg-lines-gray" />
      )}
    </div>
  );
};
