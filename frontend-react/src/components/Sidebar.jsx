import { useState } from "react";
import logo from "../assets/logo.png";

// All the settings that adapt per crossword type
const TYPE_OPTIONS = ["Simple", "Scandi"];
const PRESET_GRIDS = ["15x15", "21x21"];
const DIFFICULTIES = ["Beginner", "Medium", "Expert"];
const THEMES = [
  "Arts & culture",
  "Nature & science",
  "Sports & games",
  "Food & drink",
  "History & society",
  "Technology",
  "Seasonal",
  "Wordplay",
  "Random",
  "Mixed",
];

// Sizes offered by the W/H pickers when the grid is set to Custom.
const DIMENSIONS = Array.from({ length: 27 }, (_, i) => i + 4);

function PanelIcon() {
  return (
    <svg width="18" height="18" viewBox="0 0 18 18" fill="none" aria-hidden="true">
      <rect x="1.75" y="2.75" width="14.5" height="12.5" rx="2.5" stroke="currentColor" strokeWidth="1.5" />
      <line x1="6.75" y1="2.75" x2="6.75" y2="15.25" stroke="currentColor" strokeWidth="1.5" />
    </svg>
  );
}

export default function Sidebar({ settings, onUpdate, onGenerate, onReset, sidebarOpen, onToggle })
{
  const { type, difficulty, grid, themes } = settings;

  // "Custom" cannot be read back off the grid string alone — 15x15 typed by hand looks exactly like
  // the preset — so the mode the user picked is tracked here.
  const [customGrid, setCustomGrid] = useState(!PRESET_GRIDS.includes(grid));

  const dimensions = grid.match(/(\d+)x(\d+)/);
  const width = dimensions ? Number(dimensions[1]) : 15;
  const height = dimensions ? Number(dimensions[2]) : 15;

  function toggleTheme(theme)
  {
    const already = themes.includes(theme);
    onUpdate("themes", already ? themes.filter((t) => t !== theme) : [...themes, theme]);
  }

  function selectGrid(value)
  {
    if (value === "Custom") { setCustomGrid(true); return; }

    setCustomGrid(false);
    onUpdate("grid", value);
  }

  return (
    <div className="sidebar-inner">
      <div className="sidebar-header">
        <div className="sidebar-logo">
          <img src={logo} alt="Crossy" />
        </div>

        <button
          className="sidebar-toggle"
          onClick={onToggle}
          title={sidebarOpen ? "Hide sidebar" : "Show sidebar"}
          aria-label={sidebarOpen ? "Hide sidebar" : "Show sidebar"}
        >
          <PanelIcon />
        </button>
      </div>

      {sidebarOpen &&
      (
        <div className="sidebar-content">
          <h2 className="sidebar-title">Create your crossword!</h2>

          {/* Type */}
          <div className="field-row">
            <label className="field-label" htmlFor="type-select">Type</label>
            <select id="type-select" className="field-select" value={type} onChange={(e) => onUpdate("type", e.target.value)}>
              {TYPE_OPTIONS.map((g) => ( <option key={g} value={g}>{g}</option> ))}
            </select>
          </div>

          {/* Grid */}
          <div className="field-row">
            <label className="field-label" htmlFor="grid-select">Grid</label>
            <select
              id="grid-select"
              className="field-select"
              value={customGrid ? "Custom" : grid}
              onChange={(e) => selectGrid(e.target.value)}
            >
              {PRESET_GRIDS.map((g) => ( <option key={g} value={g}>{g}</option> ))}
              <option value="Custom">Custom</option>
            </select>
          </div>

          {customGrid && (
            <div className="dimension-row">
              <label className="dimension">
                <span>W</span>
                <select
                  className="field-select field-select--compact"
                  value={width}
                  onChange={(e) => onUpdate("grid", `${e.target.value}x${height}`)}
                >
                  {DIMENSIONS.map((d) => ( <option key={d} value={d}>{d}</option> ))}
                </select>
              </label>

              <label className="dimension">
                <span>H</span>
                <select
                  className="field-select field-select--compact"
                  value={height}
                  onChange={(e) => onUpdate("grid", `${width}x${e.target.value}`)}
                >
                  {DIMENSIONS.map((d) => ( <option key={d} value={d}>{d}</option> ))}
                </select>
              </label>
            </div>
          )}

          {/* Difficulty */}
          <div className="diff-row">
            {DIFFICULTIES.map((d) => (
              <button
                key={d}
                className={`diff-btn diff-btn--${d.toLowerCase()} ${difficulty === d ? "active" : ""}`}
                onClick={() => onUpdate("difficulty", d)}
              >
                {d}
              </button>
            ))}
          </div>

          {/* Theme */}
          <div className="field-group">
            <div className="field-label field-label--block">Theme (optional)</div>
            <div className="chip-grid">
              {THEMES.map((theme) => (
                <button key={theme} className={`chip ${themes.includes(theme) ? "active" : ""}`} onClick={() => toggleTheme(theme)}>
                  {theme}
                </button>
              ))}
            </div>
          </div>

          {/* Actions */}
          <button className="btn-generate" onClick={onGenerate}> Generate crossword → </button>
          <button className="btn-reset" onClick={onReset}> Reset </button>
        </div>
      )}
    </div>
  );
}
