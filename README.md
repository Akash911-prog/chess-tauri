# Chess

A desktop chess application built with Tauri, React, TypeScript, and Rust.

The frontend handles the board and user interaction. The Rust side owns the actual chess rules and the engine. The goal is to keep the expensive parts of chess away from JavaScript, where they have no business being.

## Tech Stack

- **Frontend:** React 19, TypeScript
- **Styling:** Tailwind CSS
- **Desktop runtime:** Tauri 2
- **Chess engine:** Rust
- **Build tools:** Vite, Cargo
- **State / communication:** Tauri commands and events

## Architecture

```text
React / TypeScript
        │
        │ Tauri commands
        ▼
Rust application
        │
        ├── Game
        │    └── Board
        │
        └── Computer
             ├── Move generation
             ├── Search
             ├── Evaluation
             └── Transposition table
```

The frontend does not implement chess rules. It sends moves to the Rust backend and receives the resulting board changes and game state.

The Rust engine owns:

- Piece positions
- Legal move generation
- Check and pin detection
- Castling
- En passant
- Promotion
- Move history
- Zobrist hashing
- Game-state resolution
- Computer move selection

## Board Representation

The board uses **bitboards**.

There is a bitboard for each combination of:

```text
Color × Piece Type
```

giving 12 piece bitboards in total.

Additional cached bitboards track:

- Occupancy by color
- Total occupancy
- King positions
- Attack masks
- Attacks grouped by piece type
- Pinned pieces

This makes operations such as occupancy checks, attacks, and piece iteration cheap enough to use heavily during search.

Set bits are iterated using bit operations rather than scanning all 64 squares.

## Move Generation

Move generation is implemented in Rust and is based around precomputed attack information and bitboards.

The engine handles:

- Pawn pushes
- Pawn captures
- Double pawn pushes
- En passant
- Knights
- Bishops
- Rooks
- Queens
- Kings
- Castling
- Promotions
- Checks
- Pins

Move generation is split between pseudo-legal generation and legal-move validation.

The engine also maintains attack information incrementally after moves rather than rebuilding everything from scratch whenever possible.

## Move Representation

Moves are represented using a compact `Move` structure rather than strings such as `e2e4`.

A move contains information needed by the engine to apply and undo it, including:

- Source square
- Destination square
- Moving piece
- Captured piece
- Move flags

Special move types are represented through flags, including:

- Castling
- En passant
- Double pawn push
- Promotion

This keeps the search from repeatedly parsing or constructing human-readable move notation.

## Search

The computer player uses iterative deepening with a negamax search.

The search currently includes:

- Alpha-beta pruning
- Principal variation style searching
- Transposition-table lookups
- Null-move pruning
- Late Move Reductions
- Move ordering
- Time-limited iterative deepening

The general search flow is:

```text
Iterative Deepening
        │
        ▼
     Negamax
        │
        ├── Transposition Table
        │
        ├── Null Move Pruning
        │
        ├── Move Generation
        │
        ├── Move Ordering
        │
        └── Reduced / Full Searches
```

Search is time-limited rather than being run to a fixed depth.

The engine records the number of visited nodes and the last completed search depth, which makes it possible to measure changes to the search rather than judging performance solely by the displayed depth.

## Transposition Table

The search uses a transposition table keyed by the board's Zobrist hash.

A table entry stores:

- Position hash
- Search depth
- Score
- Bound type
- Best move

The three bound types are:

```text
Exact
Lower
Upper
```

Mate scores are adjusted using the current ply when stored and retrieved so that mate distances remain meaningful at different points in the tree.

## Zobrist Hashing

Each board position has a 64-bit Zobrist hash.

The hash includes:

- Piece placement
- Piece color
- Castling rights
- En passant square
- Side to move

The hash is updated incrementally when moves are made instead of recomputed from the entire board.

The previous hash is stored in the move history, allowing it to be restored when a move is undone.

## Move History

Moves are stored in a history manager.

Each undo record contains the information required to restore important position state, including:

- Move
- Castling rights
- En passant square
- Halfmove clock
- Previous Zobrist hash

This history is used by both normal game operation and the engine's make/undo search path.

## Game State

The board resolves the current game state from the actual position.

Supported states include:

```text
InProgress
Checkmate
Stalemate
Draw
```

Checkmate and stalemate are determined by checking whether the side to move has any legal moves and whether its king is currently in check.

Draw handling is part of game-state resolution rather than being tied exclusively to the computer player, so local games and engine games use the same rules.

## Evaluation

The engine has a static position evaluator implemented in Rust.

The evaluation currently considers positional information in addition to material, including things such as:

- Material values
- Piece-square tables
- Mobility
- Pawn structure
- King safety
- Piece activity
- Game phase

Evaluation is performed from the perspective required by the negamax search.

The evaluator is intentionally kept separate from move generation and search so that changes to one do not require rewriting the others.

## Engine Development

Performance matters here.

Chess engines spend most of their time doing the same relatively small collection of things millions of times:

```text
generate moves
make move
search
undo move
evaluate
```

Consequently, changes to these paths should be measured rather than judged by how clever the code looks.

Useful metrics when working on the engine include:

- Nodes searched
- Nodes per second
- Search depth
- Transposition-table hit rate
- Cutoffs
- Reduced searches
- Evaluation count
- Move-generation cost

A higher displayed depth is not automatically an improvement. Searching fewer nodes to reach the same depth is usually more interesting.

## Building

### Prerequisites

Install:

- Node.js
- Rust
- Cargo
- Tauri prerequisites for your operating system

### Install frontend dependencies

```bash
npm install
```

### Development

Run the Tauri application in development mode:

```bash
npm run tauri dev
```

For frontend-only development:

```bash
npm run dev
```

### Production build

```bash
npm run build
```

Then build the Tauri application with:

```bash
npm run tauri build
```

The Rust release profile uses:

- `opt-level = 3`
- Fat LTO
- One codegen unit
- `panic = "abort"`

These settings are intentional because the engine spends a considerable amount of time in tight Rust loops.

## Project Structure

```text
.
├── src/
│   ├── components/
│   ├── ...
│   └── ...
│
├── src-tauri/
│   ├── src/
│   │   ├── engine/
│   │   │   ├── bitboard/
│   │   │   ├── board/
│   │   │   ├── computer/
│   │   │   │   ├── evaluator.rs
│   │   │   │   ├── negamax.rs
│   │   │   │   └── tt.rs
│   │   │   ├── history.rs
│   │   │   ├── movegen/
│   │   │   └── ...
│   │   │
│   │   ├── dto.rs
│   │   └── ...
│   │
│   └── Cargo.toml
│
├── package.json
└── README.md
```

The exact frontend structure may change as the UI develops. The engine is the less disposable part.

## Current Status

The project currently has a working Rust chess engine integrated with the Tauri application.

Implemented engine components include:

- Bitboard board representation
- Legal move generation
- Check detection
- Pin detection
- Castling
- En passant
- Promotions
- Make / undo move
- Incremental board state updates
- Zobrist hashing
- Transposition table
- Iterative deepening
- Alpha-beta negamax
- Null-move pruning
- Late Move Reductions
- Static evaluation
- Checkmate detection
- Stalemate detection
- Local game interaction
- Engine move generation

The engine is still under active development. Search strength and performance are being improved incrementally rather than by throwing increasingly complicated heuristics at it and hoping the CPU agrees.

## Philosophy

The engine is being developed from the bottom up.

Correctness comes first:

```text
board state
    ↓
legal moves
    ↓
make / undo
    ↓
perft
    ↓
search
    ↓
evaluation
    ↓
optimization
```

Performance work is measured against the actual search rather than based on assumptions.

If a change makes the code 30% more complicated and saves 0.4% runtime, it probably wasn't worth the keyboard wear.
