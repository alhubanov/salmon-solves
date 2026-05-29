import { useState, useEffect } from "react";
import { build_crossword_grid } from "../../../pkg/crossy";

// Parse "15x15" → { cols: 15, rows: 15 }
function parseGrid(gridStr) {
  const match = gridStr.match(/(\d+)x(\d+)/);
  if (!match) return { cols: 15, rows: 15 };
  return { cols: parseInt(match[1]), rows: parseInt(match[2]) };
}

export default function CrosswordGrid({ settings, generated }) {
  const { cols, rows } = parseGrid(settings.grid);
  const [cells, setCells] = useState([]);
  const [userInput, setUserInput] = useState({});

  useEffect(() => {
    if (generated) {
      const grid = build_crossword_grid(cols, rows);
      // TODO: don't flatten
      const cells = grid.layout.flat();

      setCells(cells);
      setUserInput({});
    }
  }, [generated, settings.grid, settings.type]);

  if (!generated || cells.length === 0) {
    return (
      <div className="empty-state">
        <svg width="48" height="48" viewBox="0 0 48 48" fill="none" xmlns="http://www.w3.org/2000/svg">
          <rect x="4" y="4" width="18" height="18" rx="2" fill="#e4e2db"/>
          <rect x="26" y="4" width="18" height="18" rx="2" fill="#e4e2db"/>
          <rect x="4" y="26" width="18" height="18" rx="2" fill="#e4e2db"/>
          <rect x="26" y="26" width="18" height="18" rx="2" fill="#6c63ff" opacity="0.3"/>
        </svg>
        <p style={{ fontSize: "20px" }}>Configure your settings and hit<br /><strong>Generate crossword</strong> to start.</p>
      </div>
    );
  }

  // Simple cell numbering: number a white cell if it starts an across or down word
  function getCellNumber(idx) {
    const r = Math.floor(idx / cols);
    const c = idx % cols;
    if (cells[idx].cell_state === "NotFilled") return null;
    const startsAcross = c === 0 || cells[idx - 1]?.cell_state === "NotFilled";
    const startsDown   = r === 0 || cells[idx - cols]?.cell_state === "NotFilled";
    const hasAcross    = c + 1 < cols && cells[idx + 1]?.cell_state !== "NotFilled";
    const hasDown      = r + 1 < rows && cells[idx + cols]?.cell_state !== "NotFilled";
    return (startsAcross && hasAcross) || (startsDown && hasDown) ? true : null;
  }

  let cellNumber = 0;

  return (
    <div className="grid-wrapper">
      <div
        className="crossword-grid"
        style={{ gridTemplateColumns: `repeat(${cols}, 34px)` }}
      >
        {cells.map((cell, idx) => {
          const num = getCellNumber(idx);
          if (num) cellNumber++;
          const n = num ? cellNumber : null;

          const isBlack = cell.cell_state === "NotFilled";

          return (
            <div key={idx} className={`grid-cell ${isBlack ? "black" : "white"}`}>
              {!isBlack && (
                <>
                  {n && <span className="cell-number">{n}</span>}
                  <input
                    maxLength={1}
                    value={userInput[idx] ?? cell.cell_value ?? ""}
                    onChange={(e) =>
                      setUserInput((prev) => ({ ...prev, [idx]: e.target.value }))
                    }
                    aria-label={`cell ${idx}`}
                  />
                </>
              )}
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