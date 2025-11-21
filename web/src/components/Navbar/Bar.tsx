import React from "react";
import NavbarButton from "./NavbarButton";

const Navbar: React.FC = () => {

  return (
    <nav className="fixed inset-x-0 top-6 z-50 pointer-events-auto">
      <div className="relative mx-auto max-w-6xl px-4">
        <div className="flex items-center justify-between p-3 rounded-2xl glass backdrop-blur-lg shadow-2xl shadow-accent">
          {/* Logo Section */}
          <div className="flex items-center gap-3">
            <div
              style={{
                background: "linear-gradient(135deg,var(--accent),#D9CCEE)",
              }}
              className="w-10 h-10 rounded-lg flex items-center justify-center text-black font-bold shadow-sm logo"
            >
              V
            </div>
            <span className="text-white font-semibold text-lg tracking-widest">
              V.E.R.S.E
            </span>
          </div>

          <div className="flex items-center gap-2">
            <NavbarButton to="/">Home</NavbarButton>
            <NavbarButton to="/models">Models</NavbarButton>
            <NavbarButton to="/docs">Docs</NavbarButton>
          </div>
        </div>
      </div>
    </nav>
  );
};

export default Navbar;
