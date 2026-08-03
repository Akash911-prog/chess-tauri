export enum Color {
    White = "w",
    Black = "b",
}

export enum PieceKind {
    Pawn = "p",
    Knight = "n",
    Bishop = "b",
    Rook = "r",
    Queen = "q",
    King = "k",
}

export type File = "a" | "b" | "c" | "d" | "e" | "f" | "g" | "h";
export type Rank = "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8";
export type Square = `${File}${Rank}`;

export type Piece = {
    color: Color;
    kind: PieceKind;
    square: Square;
};
