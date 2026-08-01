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
      if (cell.cell == null) { return { kind: "null" }; } // this should never be the case
      
      if ("Clue" in cell.cell) 
      { 
        var clue_vector = cell.cell.Clue;
        return { kind: "black", clue_vector: clue_vector }; 
      } 
      
      if ("Letter" in cell.cell) 
      { 
        return { kind: "letter", value: cell.cell.Letter[0] }; 
      }
      
      return { kind: "black" };
    }

    // Simple grid
    return {
      kind: cell.cell_state === "NotFilled" ? "black" : "letter",
      value: cell.cell_value ?? "",
    };
  }

  function ClueArrow({ direction, cellSize }) {
    const stroke = "#000";
    const common = { position: "absolute", pointerEvents: "none", zIndex: 2 };

    switch (direction) {
      case "Right":
        return (
          <svg
            style={{ ...common, top: "50%", left: "85%", width: cellSize * 0.6, height: cellSize * 0.4, transform: "translateY(-50%)" }}
            viewBox="0 0 60 40"
          >
            <line x1="0" y1="20" x2="22.5" y2="20" stroke={stroke} strokeWidth="6" />
            <polygon points="22.5,8 37.5,20 22.5,32" fill={stroke} />
          </svg>
        );

      case "Down":
        return (
          <svg
            style={{ ...common, left: "50%", top: "85%", width: cellSize * 0.4, height: cellSize * 0.6, transform: "translateX(-50%)" }}
            viewBox="0 0 40 60"
          >
            <line x1="20" y1="0" x2="20" y2="22.5" stroke={stroke} strokeWidth="6" />
            <polygon points="8,22.5 20,37.5 32,22.5" fill={stroke} />
          </svg>
        );

      // enters the cell BELOW, drops down, then turns right
      case "RightOnBottomSide":
        return (
          <svg
            style={{ ...common, left: 0, top: "100%", width: cellSize, height: cellSize }}
            viewBox="0 0 100 100"
          >
            <polyline points="10,0 10,25 27.5,25" fill="none" stroke={stroke} strokeWidth="8" strokeLinecap="round" strokeLinejoin="round" />
            <polygon points="27.5,13 40,25 27.5,37" fill={stroke} />
          </svg>
        );

      // enters the cell to the RIGHT, moves in, then turns down
      case "DownOnRightSide":
        return (
          <svg
            style={{ ...common, left: "100%", top: 0, width: cellSize, height: cellSize }}
            viewBox="0 0 100 100"
          >
            <polyline points="0,10 25,10 25,27.5" fill="none" stroke={stroke} strokeWidth="8" strokeLinecap="round" strokeLinejoin="round" />
            <polygon points="13,27.5 25,40 37,27.5" fill={stroke} />
          </svg>
        );

      default:
        return null;
    }
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

          if (normalizedCell.kind === "null") 
          {
            return <div key={idx} className="grid-cell null" style={{ width: 0, height: 0 }} />;
          }

          if (normalizedCell.kind === "black") 
          {
            return (
              <div key={idx} className="grid-cell black" style={{ width: cellSize, height: cellSize, position: "relative" }}>
                {normalizedCell.clue_vector?.map(([label, number, direction], i) => (
                  <ClueArrow key={i} direction={direction} cellSize={cellSize} />
                ))}
              </div>
            );
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