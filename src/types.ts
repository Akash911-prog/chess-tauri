enum Color {
    White = "w",
    Black = "b",
}
enum PieceKind {
    Pawn = "p",
    Knight = "n",
    Bishop = "b",
    Rook = "r",
    Queen = "q",
    King = "k",
}

type File = "a" | "b" | "c" | "d" | "e" | "f" | "g" | "h";
type Rank = "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8";
type Square = `${File}${Rank}`;

export type Piece = {
    color: Color;
    kind: PieceKind;
    square: Square;
};
