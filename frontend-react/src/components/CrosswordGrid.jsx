import { useState, useEffect } from "react";

// Parse "15x15" → { cols: 15, rows: 15 }
function parseGrid(gridStr) {
  const match = gridStr.match(/(\d+)x(\d+)/);
  if (!match) return { cols: 15, rows: 15 };
  return { cols: parseInt(match[1]), rows: parseInt(match[2]) };
}

// Generate a simple placeholder grid pattern.
// Black squares are placed symmetrically (American-style for now).
// Later you'll replace this with real generated puzzle data.
function buildPlaceholderGrid(cols, rows, type) {
  const total = cols * rows;
  const cells = Array(total).fill("white");

  if (type === "american") {
    // Roughly 17% black squares, 180-degree symmetric
    const blackIndexes = [];
    for (let r = 0; r < rows; r++) {
      for (let c = 0; c < cols; c++) {
        const idx = r * cols + c;
        if (!blackIndexes.includes(idx) && Math.random() < 0.17) {
          blackIndexes.push(idx);
          const mirror = (rows - 1 - r) * cols + (cols - 1 - c);
          blackIndexes.push(mirror);
        }
      }
    }
    blackIndexes.forEach((i) => { if (i < total) cells[i] = "black"; });
  }
  // British: sparser, ~30% black
  else if (type === "british") {
    for (let i = 0; i < total; i++) {
      if (Math.random() < 0.3) cells[i] = "black";
    }
  }
  // Scandinavian: no black squares
  // (cells stay white — arrows & clue-in-cell come later)

  return cells;
}

export default function CrosswordGrid({ settings, generated }) {
  const { cols, rows } = parseGrid(settings.grid);
  const [cells, setCells] = useState([]);
  const [userInput, setUserInput] = useState({});

  useEffect(() => {
    if (generated) {
      setCells(buildPlaceholderGrid(cols, rows, settings.type));
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
        <p>Configure your settings and hit<br /><strong>Generate crossword</strong> to start.</p>
      </div>
    );
  }

  // Simple cell numbering: number a white cell if it starts an across or down word
  function getCellNumber(idx) {
    const r = Math.floor(idx / cols);
    const c = idx % cols;
    if (cells[idx] === "black") return null;
    const startsAcross = c === 0 || cells[idx - 1] === "black";
    const startsDown   = r === 0 || cells[idx - cols] === "black";
    const hasAcross    = c + 1 < cols && cells[idx + 1] !== "black";
    const hasDown      = r + 1 < rows && cells[idx + cols] !== "black";
    return (startsAcross && hasAcross) || (startsDown && hasDown) ? true : null;
  }

  let cellNumber = 0;

  return (
    <div className="grid-wrapper">
      <div
        className="crossword-grid"
        style={{ gridTemplateColumns: `repeat(${cols}, 34px)` }}
      >
        {cells.map((type, idx) => {
          const num = getCellNumber(idx);
          if (num) cellNumber++;
          const n = num ? cellNumber : null;

          return (
            <div key={idx} className={`grid-cell ${type}`}>
              {type === "white" && (
                <>
                  {n && <span className="cell-number">{n}</span>}
                  <input
                    maxLength={1}
                    value={userInput[idx] || ""}
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