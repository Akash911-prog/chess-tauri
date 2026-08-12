import { Color, Piece, PieceKind, Square } from "./types";

export type MoveInfo = {
    from: string;
    to: string;
    promotion: "queen" | "rook" | "bishop" | "knight" | null;
};
export type Response = {
    moveType: "normal" | "castling" | "promotion" | "enPassant";
    changes: SquareChange[];
    condition: "inprogress" | "Draw" | "checkmate" | "stalemate";
    winner: string;
};

export interface SquareChange {
    square: Square;
    piece: PieceInfo | null;
}

export interface PieceInfo {
    color: string;
    kind: string;
}

export function toPiece(info: SquareChange): Piece | null {
    if (!info.piece) return null;

    return {
        color: info.piece?.color as Color,
        kind: info.piece?.kind as PieceKind,
        square: info.square,
    };
}

export interface PendingPromotion {
    from: string;
    to: string;
    color: Color;
}
