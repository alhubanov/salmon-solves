import { jsPDF } from "jspdf";

// A4 portrait in points, which is the unit jsPDF works in here.
const PAGE = { width: 595.28, height: 841.89 };
const MARGIN = 40;
const CONTENT_WIDTH = PAGE.width - MARGIN * 2;
const PAGE_BOTTOM = PAGE.height - MARGIN;

const COLUMN_GAP = 24;
const COLUMN_WIDTH = (CONTENT_WIDTH - COLUMN_GAP) / 2;

const INK = 26;          // grey level of the black squares, matching --text-primary
const RULE = 150;        // cell borders

// The arrow geometry mirrors ClueArrow in CrosswordGrid: same fractions of a cell, so the printed
// sheet points the same way the screen does.
function drawArrow(doc, direction, x, y, size)
{
  const head = (tipX, tipY, dx, dy) =>
  {
    // isosceles head pointing along (dx, dy)
    const half = size * 0.12;
    const back = size * 0.125;
    doc.triangle(
      tipX, tipY,
      tipX - dx * back - dy * half, tipY - dy * back - dx * half,
      tipX - dx * back + dy * half, tipY - dy * back + dx * half,
      "F"
    );
  };

  doc.setLineWidth(size * 0.06);
  doc.setLineCap("round");
  doc.setLineJoin("round");

  switch (direction)
  {
    case "Right":
      doc.line(x + size * 0.85, y + size * 0.5, x + size * 1.075, y + size * 0.5);
      head(x + size * 1.225, y + size * 0.5, 1, 0);
      break;

    case "Down":
      doc.line(x + size * 0.5, y + size * 0.85, x + size * 0.5, y + size * 1.075);
      head(x + size * 0.5, y + size * 1.225, 0, 1);
      break;

    // drops into the cell below, then turns right
    case "RightOnBottomSide":
      doc.line(x + size * 0.10, y + size, x + size * 0.10, y + size * 1.25);
      doc.line(x + size * 0.10, y + size * 1.25, x + size * 0.275, y + size * 1.25);
      head(x + size * 0.40, y + size * 1.25, 1, 0);
      break;

    // runs into the cell to the right, then turns down
    case "DownOnRightSide":
      doc.line(x + size, y + size * 0.10, x + size * 1.25, y + size * 0.10);
      doc.line(x + size * 1.25, y + size * 0.10, x + size * 1.25, y + size * 0.275);
      head(x + size * 1.25, y + size * 0.40, 0, 1);
      break;

    default:
      break;
  }
}

// Slot number inside its black square, tucked beside the arrow it belongs to.
function drawClueNumber(doc, number, direction, x, y, size)
{
  const inset = size * 0.08;
  const label = String(number);

  doc.setFontSize(Math.max(4.5, size * 0.26));
  doc.setTextColor(255);

  switch (direction)
  {
    case "Right":
      doc.text(label, x + size - inset, y + size * 0.5, { align: "right", baseline: "middle" });
      break;
    case "Down":
      doc.text(label, x + size * 0.5, y + size - inset, { align: "center", baseline: "bottom" });
      break;
    case "DownOnRightSide":
      doc.text(label, x + size - inset, y + inset, { align: "right", baseline: "top" });
      break;
    case "RightOnBottomSide":
      doc.text(label, x + inset, y + size - inset, { align: "left", baseline: "bottom" });
      break;
    default:
      break;
  }
}

function drawBoard(doc, board, cols, rows, originX, originY, cellSize)
{
  // squares first, so arrows crossing into a neighbour are never painted over
  board.forEach((cell, idx) =>
  {
    if (cell.kind === "null") return;

    const x = originX + (idx % cols) * cellSize;
    const y = originY + Math.floor(idx / cols) * cellSize;

    if (cell.kind === "black")
    {
      doc.setFillColor(INK);
      doc.rect(x, y, cellSize, cellSize, "F");
      return;
    }

    doc.setDrawColor(RULE);
    doc.setLineWidth(0.5);
    doc.setFillColor(255);
    doc.rect(x, y, cellSize, cellSize, "FD");

    if (cell.letter)
    {
      doc.setTextColor(INK);
      doc.setFontSize(cellSize * 0.42);
      doc.text(cell.letter.toUpperCase(), x + cellSize / 2, y + cellSize * 0.55, {
        align: "center",
        baseline: "middle",
      });
    }
  });

  board.forEach((cell, idx) =>
  {
    if (cell.kind !== "black") return;

    const x = originX + (idx % cols) * cellSize;
    const y = originY + Math.floor(idx / cols) * cellSize;

    for (const [number, direction] of cell.clues)
    {
      doc.setFillColor(INK);
      doc.setDrawColor(INK);
      drawArrow(doc, direction, x, y, cellSize);
      drawClueNumber(doc, number, direction, x, y, cellSize);
    }
  });

  doc.setDrawColor(INK);
  doc.setLineWidth(1.4);
  doc.rect(originX, originY, cols * cellSize, rows * cellSize, "S");
}

export function exportCrosswordPdf({ cols, rows, board, horizontalClues, verticalClues })
{
  const doc = new jsPDF({ unit: "pt", format: "a4" });
  doc.setFont("helvetica", "normal");

  doc.setTextColor(INK);
  doc.setFont("helvetica", "bold");
  doc.setFontSize(18);
  doc.text("Crossword", MARGIN, MARGIN + 6, { baseline: "top" });

  doc.setFont("helvetica", "normal");
  doc.setFontSize(10);
  doc.setTextColor(120);
  doc.text(`${cols} x ${rows}`, PAGE.width - MARGIN, MARGIN + 9, { align: "right", baseline: "top" });

  // A quarter cell of slack on each side keeps arrows that stick out of the board on the page.
  const boardTop = MARGIN + 42;
  const cellSize = Math.min(CONTENT_WIDTH / (cols + 0.5), 34);
  const boardWidth = cols * cellSize;
  const originX = MARGIN + (CONTENT_WIDTH - boardWidth) / 2;

  drawBoard(doc, board, cols, rows, originX, boardTop, cellSize);

  // ── clues ──
  // Each direction owns a column and both start on the same line, so the two sections sit side by
  // side instead of one spilling into the other. A column that runs past the bottom continues in
  // the same column on the following page, reusing a page the other section already started.
  const clueTop = boardTop + rows * cellSize + 34;

  function writeColumn(columnIndex, heading, clues)
  {
    let page = 1;
    let cursorY = clueTop;
    const x = MARGIN + columnIndex * (COLUMN_WIDTH + COLUMN_GAP);

    doc.setPage(page);

    function reserve(height)
    {
      if (cursorY + height <= PAGE_BOTTOM) return;

      page += 1;
      if (page > doc.getNumberOfPages()) doc.addPage();

      doc.setPage(page);
      cursorY = MARGIN;
    }

    reserve(30);
    doc.setFont("helvetica", "bold");
    doc.setFontSize(11);
    doc.setTextColor(INK);
    doc.text(heading.toUpperCase(), x, cursorY, { baseline: "top" });

    cursorY += 13;
    doc.setDrawColor(INK);
    doc.setLineWidth(0.6);
    doc.line(x, cursorY, x + COLUMN_WIDTH, cursorY);
    cursorY += 9;

    const entries = clues.length > 0
      ? clues
      : [{ number: "-", description: "No clues in this direction." }];

    for (const { number, description } of entries)
    {
      const labelWidth = 22;
      const lines = doc.splitTextToSize(description, COLUMN_WIDTH - labelWidth);
      const height = lines.length * 12 + 3;

      reserve(height);

      doc.setFont("helvetica", "normal");
      doc.setFontSize(9.5);
      doc.setTextColor(INK);
      doc.text(`${number}.`, x, cursorY, { baseline: "top" });
      doc.text(lines, x + labelWidth, cursorY, { baseline: "top" });

      cursorY += height;
    }
  }

  writeColumn(0, "Across", horizontalClues);
  writeColumn(1, "Down", verticalClues);

  doc.save(`crossword-${cols}x${rows}.pdf`);
}
