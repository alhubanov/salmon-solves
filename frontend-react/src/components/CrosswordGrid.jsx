import { Fragment, useState, useEffect, useMemo, useRef, useCallback } from "react";
import { build_crossword_grid } from "../../../pkg/crossy";

// Parse "15x15" → { cols: 15, rows: 15 }
function parseGrid(gridStr)
{
  const match = gridStr.match(/(\d+)x(\d+)/);
  if (!match) return { cols: 15, rows: 15 };
  return { cols: parseInt(match[1]), rows: parseInt(match[2]) };
}

// Both arrow shapes on an axis belong to the same clue list: the "OnSide" variants only differ
// in where the arrow turns, not in the direction the word is read.
const HORIZONTAL_DIRECTIONS = ["Right", "RightOnBottomSide"];
const VERTICAL_DIRECTIONS = ["Down", "DownOnRightSide"];

// Kept at module scope so typing in the grid re-renders the lists in place rather than
// remounting them, which would throw away how far the user had scrolled.
function ClueList({ title, clues })
{
  return (
    <section className="clue-list">
      <h3>{title} <span className="clue-list-count">{clues.length}</span></h3>

      {clues.length === 0
        ? <p className="clue-list-empty">No clues in this direction.</p>
        : (
          <ol>
            {clues.map(({ number, description }) => (
              <li key={number}>
                <span className="clue-list-number">{number}</span>
                <span className="clue-list-text">{description}</span>
              </li>
            ))}
          </ol>
        )}
    </section>
  );
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

  // Walk every clue square once and split its clues into the two lists shown beside the grid.
  const { horizontalClues, verticalClues } = useMemo(() =>
  {
    const horizontal = [];
    const vertical = [];

    if (gridType === "Scandi")
    {
      for (const cell of cells)
      {
        if (cell?.cell == null || !("Clue" in cell.cell)) continue;

        for (const [description, number, direction] of dedupeClues(cell.cell.Clue))
        {
          if (HORIZONTAL_DIRECTIONS.includes(direction)) { horizontal.push({ number, description }); }
          else if (VERTICAL_DIRECTIONS.includes(direction)) { vertical.push({ number, description }); }
        }
      }
    }

    horizontal.sort((a, b) => a.number - b.number);
    vertical.sort((a, b) => a.number - b.number);

    return { horizontalClues: horizontal, verticalClues: vertical };
  }, [cells, gridType]);

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

  // A slot that gets backtracked and re-placed during generation has its clue appended to the
  // cell more than once, so the same (number, direction) pair can arrive repeated. Drawing it
  // twice stacks identical text and makes that number look bolder, so collapse them here.
  function dedupeClues(clue_vector)
  {
    const seen = new Set();

    return clue_vector.filter(([, number, direction]) =>
    {
      const key = `${number}:${direction}`;
      if (seen.has(key)) return false;

      seen.add(key);
      return true;
    });
  }

  function normalizeCell(cell, gridType)
  {
    // Scandi grid
    if (gridType === "Scandi") 
    {
      if (cell.cell == null) { return { kind: "null" }; } // this should never be the case
      
      if ("Clue" in cell.cell)
      {
        var clue_vector = dedupeClues(cell.cell.Clue);
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

  // The slot number, printed inside the clue square next to the arrow it belongs to.
  function ClueNumber({ number, direction, cellSize }) {
    const inset = "8%";
    const common = {
      position: "absolute",
      pointerEvents: "none",
      zIndex: 3,
      color: "#fff",
      fontSize: Math.max(8, cellSize * 0.21),
      fontWeight: 600,
      lineHeight: 1,
      fontVariantNumeric: "tabular-nums",
    };

    switch (direction) {
      // arrow leaves the middle of the right edge
      case "Right":
        return <span style={{ ...common, right: inset, top: "50%", transform: "translateY(-50%)" }}>{number}</span>;

      // arrow leaves the middle of the bottom edge
      case "Down":
        return <span style={{ ...common, bottom: inset, left: "50%", transform: "translateX(-50%)" }}>{number}</span>;

      // arrow leaves the top of the right edge
      case "DownOnRightSide":
        return <span style={{ ...common, right: inset, top: inset }}>{number}</span>;

      // arrow leaves the left of the bottom edge
      case "RightOnBottomSide":
        return <span style={{ ...common, left: inset, bottom: inset }}>{number}</span>;

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

  const hasClues = horizontalClues.length > 0 || verticalClues.length > 0;

  return (
    <div className="grid-and-clues">
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
                  {/* each clue is a (description, slot number, direction) tuple */}
                  {normalizedCell.clue_vector?.map(([, number, direction], i) => (
                    <Fragment key={i}>
                      <ClueArrow direction={direction} cellSize={cellSize} />
                      <ClueNumber number={number} direction={direction} cellSize={cellSize} />
                    </Fragment>
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

      {hasClues && (
        <aside className="clue-panel" style={{ height: Math.max(280, rows * cellSize) }}>
          <ClueList title="Horizontal" clues={horizontalClues} />
          <ClueList title="Vertical" clues={verticalClues} />
        </aside>
      )}
    </div>
  );
}