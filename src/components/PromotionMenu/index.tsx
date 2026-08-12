import { Color, Piece, PieceKind } from "../../types";
import PieceRenderer from "../Piece";

interface PromotionMenuProps {
    color: Color;
    isOpen: boolean;
}

const PromotionMenu = ({ color, isOpen }: PromotionMenuProps) => {
    let pieces: Piece[] = [
        { kind: PieceKind.Queen, color, square: "a1" },
        { kind: PieceKind.Rook, color, square: "a1" },
        { kind: PieceKind.Bishop, color, square: "a1" },
        { kind: PieceKind.Knight, color, square: "a1" },
    ];

    return (
        isOpen && (
            <div className="w-[50%] h-12.5 grid-cols-4 grid relative top-1/2 -translate-y-1/2 left-1/2 -translate-x-1/2">
                {pieces.map((piece) => (
                    <div key={piece.kind} className="bg-sq-light">
                        <PieceRenderer piece={piece} />
                    </div>
                ))}
            </div>
        )
    );
};

export default PromotionMenu;
