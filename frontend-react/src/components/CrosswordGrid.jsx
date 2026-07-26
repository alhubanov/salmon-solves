import { useState, useEffect, useRef, useCallback } from "react";
import { build_crossword_grid } from "../../../pkg/crossy";

// Parse "15x15" → { cols: 15, rows: 15 }
function parseGrid(gridStr) 
{
  const match = gridStr.match(/(\d+)x(\d+)/);
  if (!match) return { cols: 15, rows: 15 };
  return { cols: parseInt(match[1]), rows: parseInt(match[2]) };
}

export default function CrosswordGrid({ settings, generated }) 
{
  const { cols, rows } = parseGrid(settings.grid);
  const [cells, setCells] = useState([]);
  const [userInput, setUserInput] = useState({});
  const gridType = settings.type;

  const containerRef = useRef(null);
  const [containerSize, setContainerSize] = useState({ width: 800, height: 800 });

  useEffect(() => 
  {
    const el = containerRef.current;
    if (!el) return;

    const observer = new ResizeObserver((entries) => {
      const { width, height } = entries[0].contentRect;
      setContainerSize({ width, height });
    });

    observer.observe(el);
    return () => observer.disconnect();
  }, []);

  const MAX_CELL_SIZE = 80;
  const MIN_CELL_SIZE = 20;
  const rawCellSize = Math.min(containerSize.width / cols, containerSize.height / rows);
  const cellSize = Math.max(MIN_CELL_SIZE, Math.min(MAX_CELL_SIZE, rawCellSize));

  useEffect(() => 
  {
    if (generated) 
    {
      // TODO: incorporate all settings properly
      let partial_settings = 
      {
        grid_type: settings.type,
        difficulty_level: settings.difficulty,
        themes: settings.themes
      }

      const grid = build_crossword_grid(cols, rows, partial_settings);
      // TODO: don't flatten
      setCells(grid.layout.flat());
      setUserInput({});
    }
  }, [generated, settings.grid, settings.type]);

  function normalizeCell(cell, gridType) 
  {
    // Scandi grid
    if (gridType === "Scandi") 
    {
      if (cell.cell == null) { return { kind: "black" }; }
      if ("Clue" in cell.cell) { return { kind: "black" }; } // clue text not rendered yet 
      if ("Letter" in cell.cell) { return { kind: "letter", value: cell.cell.Letter.cell_value }; }
      
      return { kind: "black" };
    }

    // Simple grid
    return {
      kind: cell.cell_state === "NotFilled" ? "black" : "letter",
      value: cell.cell_value ?? "",
    };
  }

  if (!generated || cells.length === 0) {
    return (
      <div className="empty-state">
        <svg width="48" height="48" viewBox="0 0 48 48" fill="none" xmlns="http://www.w3.org/2000/svg">
          <rect x="4" y="4" width="18" height="18" rx="2" fill="#e4e2db"/>
          <rect x="26" y="4" width="18" height="18" rx="2" fill="#e4e2db"/>
          <rect x="4" y="26" width="18" height="18" rx="2" fill="#e4e2db"/>
          <rect x="26" y="26" width="18" height="18" rx="2" fill="#6c63ff" opacity="0.3"/>
        </svg>
        <p style={{ fontSize: "20px" }}>Configure your settings and press<br /><strong>Generate crossword</strong> to start.</p>
      </div>
    );
  }

  return (
    <div className="grid-wrapper">
      <div
        className="crossword-grid"
        style={{
          gridTemplateColumns: `repeat(${cols}, ${cellSize}px)`,
          gridTemplateRows: `repeat(${rows}, ${cellSize}px)`,
        }}
      >
        {cells.map((cell, idx) => {
          const normalizedCell = normalizeCell(cell, gridType);

          if (normalizedCell.kind === "black") {
            return <div key={idx} className="grid-cell black" style={{ width: cellSize, height: cellSize }} />;
          }

          return (
            <div key={idx} className="grid-cell white" style={{ width: cellSize, height: cellSize }} >
              <input
                maxLength={1}
                value={userInput[idx] ?? normalizedCell.value ?? ""}
                onChange={(e) => setUserInput((prev) => ({ ...prev, [idx]: e.target.value })) }
                aria-label={`cell ${idx}`}
              />
            </div>
          );
        })}
      </div>

      <div className="grid-toolbar">
        <button onClick={() => setUserInput({})}>Restart ↺</button>
        <span>|</span>
        <button>Export to PDF ↑</button>
      </div>
    </div>
  );
}