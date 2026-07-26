// All the settings that adapt per crossword type
const TYPE_OPTIONS = ["Simple", "Scandi"];
const GRID_OPTIONS = ["15x15", "21x21"];
const DIFFICULTIES = ["Beginner", "Medium", "Expert"];
const THEMES = ["Arts & Culture", "Nature & Science", "Sports & Games", "History & Society"];

export default function Sidebar({ settings, onUpdate, onGenerate, onReset, sidebarOpen, onToggle }) 
{
  const { type, difficulty, grid, themes } = settings;

  function toggleTheme(theme) 
  {
    const already = themes.includes(theme);
    onUpdate("themes", already ? themes.filter((t) => t !== theme) : [...themes, theme]);
  }

  const grids = GRID_OPTIONS;

  return (
  <>
    <div className="sidebar-inner">
      <button className="sidebar-toggle" onClick={onToggle} title="Toggle sidebar">
        {sidebarOpen ? "← hide" : "→ show"}
      </button>

      {sidebarOpen && 
      ( 
        <div className="sidebar-content">
          <h2 style={{ marginBottom: "24px" }}>Create your crossword!</h2>

          {/* Type */}
          <div className="field-group">
          <div className="field-label">Type</div>
            <select className="field-select" value={type} onChange={(e) => onUpdate("type", e.target.value)}>
                {TYPE_OPTIONS.map((g) => ( <option key={g} value={g}>{g}</option> ))}
            </select>
          </div>

          {/* Grid */}
          <div className="field-group">
          <div className="field-label">Grid</div>
            <select className="field-select" value={grid} onChange={(e) => onUpdate("grid", e.target.value)} >
                {GRID_OPTIONS.map((g) => ( <option key={g} value={g}>{g}</option> ))}
            </select>
          </div>

          {/* Difficulty 
          
          <div className="field-group">
          <div className="field-label">Difficulty</div>
          <div className="diff-row">
              {DIFFICULTIES.map((d) => 
                (
                  <button
                      key={d}
                      className={`diff-btn ${difficulty === d.toLowerCase() ? "active" : ""}`}
                      onClick={() => onUpdate("difficulty", d.toLowerCase())}
                  >
                      {d}
                  </button>
                ))
              }
          </div>
          </div>

          */}

          {/* Theme */}
          <div className="field-group">
            <div className="field-label">Theme</div>
            <div className="chip-grid">
              {THEMES.map((theme) => (
                <button key={theme} className={`chip ${themes.includes(theme) ? "active" : ""}`} onClick={() => toggleTheme(theme)}>
                  {theme}
                </button>
              ))}
              <button className={`chip ${themes.length === 0 ? "active" : ""}`} onClick={() => onUpdate("themes", [])}>
                Random
              </button>
            </div>
            {themes.length > 0 && (
              <p className="theme-hint">
                {themes.length} theme{themes.length > 1 ? "s" : ""} selected
              </p>
            )}
          </div>

          {/* Actions */}
          <button className="btn-generate" onClick={onGenerate}> Generate crossword → </button>
          <button className="btn-reset" onClick={onReset}> Reset </button>

        </div>
      )}
    </div>
  </>
);}
