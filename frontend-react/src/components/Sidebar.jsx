// All the settings that adapt per crossword type
const GRID_OPTIONS = {
  american: ["15x15", "21x21"],
  british:  ["15x15 blocked", "15x15 barred"],
  scandinavian: ["11x11", "13x13", "15x15"],
};

const DIFFICULTIES = {
  american:    ["Beginner", "Medium", "Expert"],
  british:     ["Beginner", "Standard", "Barred"],
  scandinavian:["Beginner", "Standard", "Expert"],
};

const THEMES = [
  "Arts & culture",
  "Nature & science",
  "Sports & games",
  "Food & drink",
  "History & society",
  "Technology",
  "Seasonal",
  "Wordplay",
];

export default function Sidebar({ settings, onUpdate, onGenerate, onReset }) {
  const { type, difficulty, grid, themes } = settings;

  // When type changes, reset grid and difficulty to first valid option
  function handleTypeChange(e) {
    const newType = e.target.value;
    onUpdate("type", newType);
    onUpdate("grid", GRID_OPTIONS[newType][0]);
    onUpdate("difficulty", DIFFICULTIES[newType][0].toLowerCase());
  }

  function toggleTheme(theme) {
    const already = themes.includes(theme);
    onUpdate(
      "themes",
      already ? themes.filter((t) => t !== theme) : [...themes, theme]
    );
  }

  const diffs = DIFFICULTIES[type];
  const grids = GRID_OPTIONS[type];

  return (
   <>
    <h1>Create your crossword!</h1>

    {/* Type */}
    <div className="field-group">
    <div className="field-label">Type</div>
    <select className="field-select" value={type} onChange={handleTypeChange}>
        <option value="american">American</option>
        <option value="british">British (cryptic)</option>
        <option value="scandinavian">Scandinavian</option>
    </select>
    </div>

    {/* Grid */}
    <div className="field-group">
    <div className="field-label">Grid</div>
    <select
        className="field-select"
        value={grid}
        onChange={(e) => onUpdate("grid", e.target.value)}
    >
        {grids.map((g) => (
        <option key={g} value={g}>{g}</option>
        ))}
    </select>
    </div>

    {/* Difficulty */}
    <div className="field-group">
    <div className="field-label">Difficulty</div>
    <div className="diff-row">
        {diffs.map((d) => (
        <button
            key={d}
            className={`diff-btn ${difficulty === d.toLowerCase() ? "active" : ""}`}
            onClick={() => onUpdate("difficulty", d.toLowerCase())}
        >
            {d}
        </button>

          ))}
        </div>
      </div>

      {/* Theme */}
      <div className="field-group">
        <div className="field-label">Theme (optional)</div>
        <div className="chip-grid">
          {THEMES.map((theme) => (
            <button
              key={theme}
              className={`chip ${themes.includes(theme) ? "active" : ""}`}
              onClick={() => toggleTheme(theme)}
            >
              {theme}
            </button>
          ))}
          <button
            className={`chip ${themes.length === 0 ? "active" : ""}`}
            onClick={() => onUpdate("themes", [])}
          >
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
      <button className="btn-generate" onClick={onGenerate}>
        Generate crossword →
      </button>
      <button className="btn-reset" onClick={onReset}>
        Reset
      </button>
    </>
  );}
