# Salmon Solves
Welcome to Salmon Solves - a generator for crossword puzzles, currently only of the Scandi variant. 

You can find the project hosted here: [Salmon Solves](https://www.salmonsolves.com/)

It features a fully human-built Rust backend. <br>
It also features an interactive UI, designed by [Ana Hubanova](https://www.linkedin.com/in/ana-hubanova/), but largely built with Claude Opus 5.

## How to run as dev

To generate a (solved) puzzle in the terminal:

```
git clone git@github.com:alhubanov/salmon-solves.git
cd salmon-solves/
cargo run
```

To run the UI on localhost:

```
git clone git@github.com:alhubanov/salmon-solves.git
cd salmon-solves/
wasm-pack build --target web
cd frontend-react/
npm run dev
```

## Notable features of the Rust backend

- **Words are grouped by length.** When looking to fill a five-letter slot, the algorithm only ever looks at five-letter words.

- **An index is built ahead of time of "which words have letter X in position N".** 
  Within one length group, the words are numbered `0, 1, 2, …`. For every combination of a position
  and a letter, the dictionary stores a *bitmap*: a long row of bits, one per word, where bit `i` is
  `1` if word `i` has that letter in that position. So the bitmap for "letter `A` in position 2"
  might look like:

  ```
  word index:  0  1  2  3  4  5  6  7  ...
  A at pos 2:  0  1  0  0  1  1  0  0  ...
                  ^        ^  ^
                  words with an A as their second letter
  ```

  If a slot is partly filled and reads `_ A _ E _`, the algorithm ANDs together two bitmaps - 
  the ones for "`A` in position 2" and "`E` in position 4".
  
  A bit survives only if it was set in both, which means that word has an `A` in position 2 *and* an
  `E` in position 4. The bits still set at the end are exactly the words that fit, and positions that
  are still blank are simply skipped rather than tested.

- **Most constrained slot is filled first.** The generator always fills the slot with the fewest remaining options.
  This is a strategy to expose dead ends early and cheaply.

- **Slots are preemtively sorted by how many other slots cross them.** The most constrained parts of the
  grid are filled first.

- **The algorightm looks ahead after every placement.** Having chosen a word for a slot, the generator checks the other slots crossing
  it, then the slots crossing *those*, and so on for a limited number of steps. If any of them is
  left with zero possible words, the choice of word for the initial slot is rejected immediately. 
  Notably, this look-ahead stops descending when a slot's options stop shrinking (or when a pre-set depth is reached),
  and it never visits the same slot twice in one pass.

  For small grids, this look-ahead does not bring a performance improvement. For ones bigger than roughly ~14x14, this makes a difference.
  The allowed depth for this look-ahead for grid 15x15 or bigger is 3 steps currently.

- **Backtracking is targeted.** When a slot is not yet filled but there are no more word options available for it, the generator undoes the most
  recently placed crossing word, rather than unwinding everything.

- **A threshold determines when to give up and restart.** After backtracking more than 100 times without yet achieving a successfully-filled grid, the
  generator throws the layout away and starts over with a fresh one. This is faster than attempting to resolve a nearly-impossible layout.

- **Crossing words share the same cells in memory.** A letter placed by an across word is instantly
  visible to the down word through it, with no copying or syncing between the two.

- **A faster hash function than the default is used.** `ahash` is used for the internal lookup tables instead of the standard
  library's default, which is built for resisting attacks rather than raw speed.

- **The word list is compiled into the binary.** There is no file I/O at runtime, which is also what allows the exact same code to run in a browser.

- **It runs as WebAssembly.** The same Rust generator that runs in the terminal is compiled to wasm
  and runs directly in the browser at close to native speed, so no server is involved in making a
  puzzle.
