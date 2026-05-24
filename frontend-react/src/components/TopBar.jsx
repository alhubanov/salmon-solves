import logo from "../assets/logo.png";

export default function TopBar({ sidebarOpen, onToggle }) {
  return (
    <header className="topbar">
      <div className="topbar-logo">
        <img src={logo} alt="Logo" style={{ width: "100%", height: "100%" }}></img>
      </div>

      <button className="topbar-toggle" onClick={onToggle} title="Toggle sidebar">
        {sidebarOpen ? "← hide" : "→ show"}
      </button>
    </header>
  );
}