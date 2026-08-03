import { Color, Piece, PieceKind, Square, File } from "../types";

const FILES: File[] = ["a", "b", "c", "d", "e", "f", "g", "h"];

const BACK_RANK: PieceKind[] = [
    PieceKind.Rook,
    PieceKind.Knight,
    PieceKind.Bishop,
    PieceKind.Queen,
    PieceKind.King,
    PieceKind.Bishop,
    PieceKind.Knight,
    PieceKind.Rook,
];

function buildInitialBoard(): Record<string, Piece | null> {
    const board: Record<string, Piece | null> = {};

    // all squares start empty
    for (const file of FILES) {
        for (let r = 1; r <= 8; r++) {
            board[`${file}${r}` as Square] = null;
        }
    }

    FILES.forEach((file, i) => {
        board[`${file}1` as Square] = {
            color: Color.White,
            kind: BACK_RANK[i],
            square: `${file}1` as Square,
        };
        board[`${file}2` as Square] = {
            color: Color.White,
            kind: PieceKind.Pawn,
            square: `${file}2` as Square,
        };

        board[`${file}7` as Square] = {
            color: Color.Black,
            kind: PieceKind.Pawn,
            square: `${file}7` as Square,
        };
        board[`${file}8` as Square] = {
            color: Color.Black,
            kind: BACK_RANK[i],
            square: `${file}8` as Square,
        };
    });

    return board;
}

export const INITIAL_BOARD: Record<string, Piece | null> = buildInitialBoard();
