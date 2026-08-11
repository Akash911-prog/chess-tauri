# Response contract cleanup — TODO

Two categories: bugs to fix regardless, and the structural change (uniform
`changes` list) that removes the need for most of the per-`moveType` branching.
Bugs are cheap and independent — do those first even if the structural
refactor gets pushed out.

---

## 1. Bugs (do these regardless of the refactor)

### 1.1 Promotion is never reported to the frontend
**File:** `src-tauri/src/engine/board/moves.rs`
**Where:** all three `Response { ... }` construction sites — `make_move`,
`do_ep_capture`, `do_castle`
**Problem:** `promotion: None` is hardcoded in every branch, even when
`self.promotion` was set earlier in `parse_react_move`. Promoted piece never
reaches the frontend.
**Fix:** populate `promotion` from `self.promotion` (map the `u8` back to
`PromotionPiece`) in all three sites, or better — fold it into the `changes`
list from item 2 below so there's only one place this can be wrong.

### 1.2 `check: Option<Color>` doesn't mean "check" — it's "who just moved"
**File:** `src-tauri/src/engine/board/moves.rs`
**Where:** all three `Response` sites —
`check: Some(Color::from(self.player_turn ^ 1))`
**Problem:** set unconditionally on every move, not just when a king is
actually in check. It's really being used as `winColor` plumbing on the
frontend.
**Fix:** rename to `winner: Option<Color>`, only `Some(...)` when
`condition == GameState::Checkmate`.

### 1.3 Stalemate/Draw get mislabeled as Checkmate
**File:** `src/gameStates/board.ts`
**Where:** `updateBoard`
```ts
if (data?.condition != "inprogress") {
    this.condition = "checkmate";   // ← wrong for stalemate/draw
    ...
}
```
**Fix:** switch on `data.condition` explicitly (`"checkmate"` /
`"stalemate"` / `"draw"`), set `this.condition` and the message per-branch
instead of collapsing everything non-`inprogress` into checkmate.

### 1.4 `GameState` casing mismatch between Rust and TS
**File:** `src/gameStates/board.ts`
**Where:** the local `Response` type —
`condition: "inprogress" | "Draw" | "checkmate" | "stalemate"`
**Problem:** Rust's `GameState` is `#[serde(rename_all = "lowercase")]`, so
it always sends `"draw"`, not `"Draw"`. Hasn't surfaced yet only because
draw conditions (50-move, repetition) aren't implemented.
**Fix:** change to `"inprogress" | "draw" | "checkmate" | "stalemate"`.

### 1.5 `finished` in the hook isn't real React state
**File:** `src/hooks/BoardSrc.ts`
**Where:**
```ts
const finished = boardRef.current.finished;
```
**Problem:** read directly off a mutable ref, not `useState`. Only appears
reactive because every mutating method happens to call `sync()` afterward.
**Fix:** add a real `useState<boolean>` for `finished`, update it alongside
`setFinished`/wherever `.finished` changes, don't rely on the ref-read
coincidence.

---

## 2. Structural refactor — uniform `changes` list

Removes the need for the frontend to interpret `from`/`to` differently per
`moveType` (rook squares for castling, captured-pawn square for en passant,
garbage sentinel for en passant's `from`, etc).

### 2.1 New DTO shapes
**File:** `src-tauri/src/dto.rs`
```rust
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PieceInfo {
    pub kind: PieceKindDto,
    pub color: Color,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SquareChange {
    pub square: String,
    pub piece: Option<PieceInfo>, // None = now empty
}
```
Update `Response`:
```rust
pub struct Response {
    pub move_type: MoveType,
    pub changes: Vec<SquareChange>,
    pub condition: GameState,
    pub winner: Option<Color>,   // see 1.2
}
```
Drop `from`/`to`/`promotion` off `Response` — folded into `changes`.

### 2.2 Build `changes` in each move function
**File:** `src-tauri/src/engine/board/moves.rs`
- `make_move` (normal + promotion): `changes = [{from: None}, {to: Some(piece)}]`
  (piece kind = promoted piece if `self.promotion != 0`, else the moved piece)
- `do_castle`: `changes` = 4 entries — king from/to, rook from/to
- `do_ep_capture`: `changes` = 3 entries — pawn from/to, captured pawn square → `None`

### 2.3 Frontend consumes `changes` directly
**File:** `src/gameStates/board.ts`
**Where:** `updateBoard`
Replace all the `moveType`-branched patch logic with:
```ts
for (const change of data.changes) {
    this._board[change.square] = change.piece
        ? { type: change.piece.kind, color: change.piece.color }
        : null;
}
```
`moveType` stays on `Response` for animation/sound triggers only — no
longer load-bearing for board correctness.

### 2.4 Update the local `Response` type to match
**File:** `src/gameStates/board.ts`
Replace the current `Response` type (`from`/`to`/`promotion`/`check`) with
one matching the new Rust shape (`changes`, `winner`).

---

## Suggested order

1. 1.1 → 1.5 (bugs, independent, safe to ship immediately)
2. 2.1 → 2.4 together (one PR — DTO shape change touches both sides at once,
   don't split it or you'll have a broken contract mid-refactor)