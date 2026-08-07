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

// The generator's Theme enum only knows these five. Chips outside it are offered in the sidebar but
// have no counterpart yet, so they are dropped here — passing one through fails the whole call with
// "unknown variant" and no grid comes back.
const BACKEND_THEMES = {
  "Arts & culture": "ArtsAndCulture",
  "Nature & science": "NatureAndScience",
  "Sports & games": "SportsAndGames",
  "History & society": "HistoryAndSociety",
  "Random": "Random",
};

// Kept at module scope so typing in the grid re-renders the lists in place rather than
// remounting them, which would throw away how far the user had scrolled.
function ClueList({ title, arrow, clues, selectedSlotId, selectionTick, onSelect })
{
  const selectedRow = useRef(null);

  // Selecting a slot in the grid can pick a clue that is scrolled out of view, so bring it back.
  // "nearest" keeps this to the minimum scroll inside the list itself — a row that is already
  // visible does not move, and the surrounding page is left alone. This keys off the tick rather
  // than the id so re-picking the slot that is already selected still scrolls back to it.
  useEffect(() =>
  {
    selectedRow.current?.scrollIntoView({ block: "nearest", behavior: "auto" });
  }, [selectedSlotId, selectionTick]);

  return (
    <section className="clue-list">
      <h3>{title} <span className="clue-list-arrow" aria-hidden="true">{arrow}</span></h3>

      {clues.length === 0
        ? <p className="clue-list-empty">No clues in this direction.</p>
        : (
          <ol>
            {clues.map(({ number, description }) => (
              <li key={number}>
                <button
                  type="button"
                  ref={number === selectedSlotId ? selectedRow : null}
                  className={`clue-row ${number === selectedSlotId ? "selected" : ""}`}
                  aria-pressed={number === selectedSlotId}
                  onClick={() => onSelect(number)}
                >
                  <span className="clue-list-number">{number}.</span>
                  <span className="clue-list-text">{description}</span>
                </button>
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
  const [selectedSlotId, setSelectedSlotId] = useState(null);
  // Bumped on every selection, so picking the already-selected slot still counts as an event.
  const [selectionTick, setSelectionTick] = useState(0);
  // The space hint only makes sense once there is a slot to switch away from.
  const [slotEverSelected, setSlotEverSelected] = useState(false);
  // { correct: Set, incorrect: Set } of cell indices while a check is on screen, else null.
  const [checkResults, setCheckResults] = useState(null);
  const gridType = settings.type;

  // Flat cell index → the <input> element, so selecting a clue can move focus into the grid.
  const inputRefs = useRef({});
  const focusFirstCell = useRef(false);

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
  const { horizontalClues, verticalClues, horizontalSlotIds } = useMemo(() =>
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

    return {
      horizontalClues: horizontal,
      verticalClues: vertical,
      horizontalSlotIds: new Set(horizontal.map((clue) => clue.number)),
    };
  }, [cells, gridType]);

  // Every letter cell carries the ids of the slots it belongs to, so one pass over the grid maps
  // each slot to its cells. The layout is flattened row-major, so ascending index is already
  // reading order for both axes — left-to-right across a row, top-to-bottom down a column.
  const slotCells = useMemo(() =>
  {
    const bySlot = new Map();

    cells.forEach((cell, idx) =>
    {
      const slotIds = cell?.cell?.Letter?.[1];
      if (!slotIds) return;

      for (const id of slotIds)
      {
        if (!bySlot.has(id)) bySlot.set(id, []);
        bySlot.get(id).push(idx);
      }
    });

    return bySlot;
  }, [cells]);

  // Slots of each axis in grid order, keyed off where the word starts, so stepping between them
  // moves down and across the grid the way it reads.
  const slotOrder = useMemo(() =>
  {
    const horizontal = [];
    const vertical = [];

    for (const id of slotCells.keys())
    {
      (horizontalSlotIds.has(id) ? horizontal : vertical).push(id);
    }

    const byStartPosition = (a, b) => slotCells.get(a)[0] - slotCells.get(b)[0];
    horizontal.sort(byStartPosition);
    vertical.sort(byStartPosition);

    return { horizontal, vertical };
  }, [slotCells, horizontalSlotIds]);

  const selectedSlotCells = useMemo(
    () => (selectedSlotId == null ? [] : slotCells.get(selectedSlotId) ?? []),
    [slotCells, selectedSlotId]
  );

  const highlightedCells = useMemo(() => new Set(selectedSlotCells), [selectedSlotCells]);

  const slotAxis = horizontalSlotIds.has(selectedSlotId) ? "horizontal" : "vertical";

  // Picking a clue jumps the caret to the slot's first letter, but clicking a cell in the grid must
  // leave the caret where it was clicked, so only the clue list arms this.
  useEffect(() =>
  {
    if (!focusFirstCell.current) return;

    focusFirstCell.current = false;
    if (selectedSlotCells.length > 0) focusCell(selectedSlotCells[0]);
  }, [selectedSlotCells, selectionTick]);

  function selectSlot(slotId, focusFirst)
  {
    focusFirstCell.current = focusFirst;
    setSelectedSlotId(slotId);
    setSelectionTick((tick) => tick + 1);
    setSlotEverSelected(true);
    clearCheck();
  }

  function selectSlotFromClue(slotId)
  {
    selectSlot(slotId, true);
  }

  // Clicking a letter cell selects the slot running through it. Crossing cells belong to two, and
  // the horizontal one wins; the native click leaves focus on the cell that was clicked.
  function handleCellClick(idx)
  {
    clearCheck();

    const slotIds = cells[idx]?.cell?.Letter?.[1];
    if (!slotIds?.length) return;

    const horizontalId = slotIds.find((id) => horizontalSlotIds.has(id));
    selectSlot(horizontalId ?? slotIds[0], false);
  }

  function focusCell(idx)
  {
    const input = inputRefs.current[idx];
    if (!input) return;

    input.focus();
    input.select(); // shows the letter as replaceable rather than parking the caret beside it
  }

  function clearCell(idx)
  {
    setUserInput((prev) => ({ ...prev, [idx]: "" }));
  }

  // The generator ships the solved letter in every cell, so that doubles as the answer key.
  function solutionLetter(idx)
  {
    return (cells[idx]?.cell?.Letter?.[0] ?? "").toUpperCase();
  }

  function currentLetter(idx)
  {
    return (userInput[idx] ?? cells[idx]?.cell?.Letter?.[0] ?? "").toUpperCase();
  }

  // Judged per cell rather than per slot: the letter either matches the solution or it does not,
  // and an empty cell counts as wrong. A correct letter stays green even when the words crossing
  // it are still unfinished.
  function checkAnswers()
  {
    const correct = new Set();
    const incorrect = new Set();

    cells.forEach((cell, idx) =>
    {
      if (cell?.cell == null || !("Letter" in cell.cell)) return;

      const letter = currentLetter(idx);
      (letter !== "" && letter === solutionLetter(idx) ? correct : incorrect).add(idx);
    });

    setCheckResults({ correct, incorrect });
  }

  // Any move back into the grid drops the check colouring. Returning the previous state unchanged
  // when there is nothing to clear lets React skip the re-render.
  function clearCheck()
  {
    setCheckResults((previous) => (previous === null ? previous : null));
  }

  // Arrows along the slot's own axis walk the caret through it; arrows across the axis jump to the
  // neighbouring slot of the same axis, so up/down on a horizontal word moves between horizontal
  // words rather than leaving the list the user is working through.
  function handleCellKeyDown(idx, event)
  {
    clearCheck(); // typing, arrows, backspace — any of them means the user is back at work

    const position = selectedSlotCells.indexOf(idx);
    if (position === -1) return;

    const alongAxis = slotAxis === "horizontal" ? ["ArrowLeft", "ArrowRight"] : ["ArrowUp", "ArrowDown"];
    const acrossAxis = slotAxis === "horizontal" ? ["ArrowUp", "ArrowDown"] : ["ArrowLeft", "ArrowRight"];

    if (alongAxis.includes(event.key))
    {
      event.preventDefault();

      const step = event.key === alongAxis[1] ? 1 : -1;
      const target = selectedSlotCells[position + step];
      if (target !== undefined) focusCell(target);

      return;
    }

    if (acrossAxis.includes(event.key))
    {
      event.preventDefault();

      const order = slotOrder[slotAxis];
      const step = event.key === acrossAxis[1] ? 1 : -1;
      const target = order[order.indexOf(selectedSlotId) + step];
      if (target !== undefined) selectSlot(target, true);

      return;
    }

    // Space swaps to the other slot through this cell, leaving the caret where it is. A cell with
    // no crossing slot has nothing to swap to, so the selection stays put.
    if (event.key === " ")
    {
      event.preventDefault();

      const slotIds = cells[idx]?.cell?.Letter?.[1] ?? [];
      const wantsHorizontal = slotAxis !== "horizontal";
      const crossingId = slotIds.find((id) => horizontalSlotIds.has(id) === wantsHorizontal);

      if (crossingId !== undefined) selectSlot(crossingId, false);

      return;
    }

    if (event.key === "Backspace")
    {
      event.preventDefault();

      // Clear a filled cell in place; on an already-empty one, step back and clear that instead.
      if (event.target.value !== "") { clearCell(idx); return; }

      const target = selectedSlotCells[position - 1];
      if (target === undefined) return;

      clearCell(target);
      focusCell(target);
    }
  }

  // Typing a letter walks the caret to the next cell of the selected slot. The browser reports the
  // character it actually inserted, so a keystroke replaces whatever the cell already held no
  // matter where the caret sat inside it; a deletion reports null and empties the cell.
  function handleCellChange(idx, inserted)
  {
    const letter = (inserted ?? "").slice(-1);

    setUserInput((prev) => ({ ...prev, [idx]: letter }));

    if (letter === "") return;

    const position = selectedSlotCells.indexOf(idx);
    if (position === -1 || position === selectedSlotCells.length - 1) return;

    focusCell(selectedSlotCells[position + 1]);
  }

  useEffect(() => 
  {
    if (generated) 
    {
      // TODO: incorporate all settings properly
      let partial_settings = 
      {
        grid_type: settings.type,
        difficulty_level: settings.difficulty,
        themes: settings.themes.map((theme) => BACKEND_THEMES[theme]).filter(Boolean)
      }

      const grid = build_crossword_grid(cols, rows, partial_settings);
      // TODO: don't flatten
      setCells(grid.layout.flat());
      setUserInput({});
      setSelectedSlotId(null);
      setSlotEverSelected(false);
      setCheckResults(null);
      inputRefs.current = {};
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
    <div className={`grid-and-clues ${hasClues ? "grid-and-clues--with-clues" : ""}`}>
      {/* always rendered, only made invisible, so revealing it does not shift the grid */}
      <p className={`grid-hint ${slotEverSelected ? "" : "grid-hint--hidden"}`}>
        Press <kbd>space</kbd> to switch between the horizontal and vertical word through the current cell.
      </p>

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

          // while a check is on screen its colouring replaces the blue selection band
          const checkState = checkResults?.correct.has(idx) ? "check-correct"
            : checkResults?.incorrect.has(idx) ? "check-incorrect"
            : highlightedCells.has(idx) ? "highlighted"
            : "";

          return (
            <div
              key={idx}
              className={`grid-cell white ${checkState}`}
              style={{ width: cellSize, height: cellSize }}
            >
              <input
                ref={(el) => { inputRefs.current[idx] = el; }}
                value={userInput[idx] ?? normalizedCell.value ?? ""}
                onChange={(e) => handleCellChange(idx, e.nativeEvent.data)}
                onKeyDown={(e) => handleCellKeyDown(idx, e)}
                onFocus={(e) => e.target.select()}
                onClick={() => handleCellClick(idx)}
                aria-label={`cell ${idx}`}
              />
            </div>
          );
        })}
      </div>

      <div className="grid-toolbar">
        <button>
          Export to PDF
          <svg width="14" height="14" viewBox="0 0 14 14" fill="none" aria-hidden="true">
            <path d="M7 9.5V1.5M7 1.5L4.25 4.25M7 1.5l2.75 2.75" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" strokeLinejoin="round" />
            <path d="M1.75 9v2.25a1.25 1.25 0 0 0 1.25 1.25h8a1.25 1.25 0 0 0 1.25-1.25V9" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" />
          </svg>
        </button>

        <button onClick={checkAnswers}>Check answers</button>
      </div>

      {hasClues && (
        <aside className="clue-panel" style={{ height: Math.max(280, rows * cellSize) }}>
          <h2 className="clue-panel-title">Clues</h2>

          <ClueList title="Across" arrow="→" clues={horizontalClues} selectedSlotId={selectedSlotId} selectionTick={selectionTick} onSelect={selectSlotFromClue} />
          <ClueList title="Down" arrow="↓" clues={verticalClues} selectedSlotId={selectedSlotId} selectionTick={selectionTick} onSelect={selectSlotFromClue} />
        </aside>
      )}
    </div>
  );
}