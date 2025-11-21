import React from "react";
import { Link, useLocation } from "react-router-dom";
import { Button } from "../ui/button";

type NavbarButtonProps = {
  to: string;
  children: React.ReactNode;
};

export default function NavbarButton({ to, children }: NavbarButtonProps) {
  const location = useLocation();
  const isActive = location.pathname === to;

  return (
    <Button asChild>
      <Link
        to={to}
        className={
          isActive
            ? "bg-[--accent-soft] ring-1 ring-accent text-white rounded-md px-4 py-2 text-sm font-medium"
            : "text-gray-200/90 hover:bg-[rgba(255,255,255,0.03)] bg-transparent rounded-md px-4 py-2 text-sm font-medium"
        }
      >
        {children}
      </Link>
    </Button>
  );
}
