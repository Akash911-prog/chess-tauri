import { Piece, PieceKind, Color } from "../../types";
import Bishop from "./pieces/Bishop";
import King from "./pieces/King";
import Knight from "./pieces/Knight";
import Pawn from "./pieces/Pawn";
import Queen from "./pieces/Queen";
import Rook from "./pieces/Rook";

const PIECE_COMPONENTS: Record<PieceKind, React.FC<{ color: Color }>> = {
    [PieceKind.Pawn]: Pawn,
    [PieceKind.Knight]: Knight,
    [PieceKind.Bishop]: Bishop,
    [PieceKind.Rook]: Rook,
    [PieceKind.Queen]: Queen,
    [PieceKind.King]: King,
};

// PieceRenderer.tsx
type Props = {
    piece: Piece | null;
};

const PieceRenderer = ({ piece }: Props) => {
    if (!piece) return null;
    const PieceComponent = PIECE_COMPONENTS[piece.kind];

    return (
        <div>
            <PieceComponent color={piece.color} />
        </div>
    );
};

export default PieceRenderer;
