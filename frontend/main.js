import init, { build_crossword_grid } from '../pkg/crossy.js';

async function run() {
    await init();

    const result = build_crossword_grid(12, 12);

    const display = document.getElementById('output');
    let output = "";

    for (const row of result.layout) {
        for (const cell of row) {
            output += cell.cell_value + " ";
        }
        output += '\n';
    }

    display.textContent = output;
    
    console.log("Data from Rust:", result);
}

run();