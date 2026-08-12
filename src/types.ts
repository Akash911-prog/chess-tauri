export enum Color {
    White = "white",
    Black = "black",
}

export enum PieceKind {
    Pawn = "pawn",
    Knight = "knight",
    Bishop = "bishop",
    Rook = "rook",
    Queen = "queen",
    King = "king",
}

export type File = "a" | "b" | "c" | "d" | "e" | "f" | "g" | "h";
export type Rank = "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8";
export type Square = `${File}${Rank}`;

export type Piece = {
    color: Color;
    kind: PieceKind;
    square: Square;
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
