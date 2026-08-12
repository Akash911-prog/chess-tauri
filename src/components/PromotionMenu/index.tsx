import { Color, PieceKind } from "../../types";
import PieceRenderer from "../Piece";

interface PromotionMenuProps {
    color: Color;
    isOpen: boolean;
    onSelect: (kind: "queen" | "rook" | "bishop" | "knight") => void;
}

const PromotionMenu = ({ color, isOpen, onSelect }: PromotionMenuProps) => {
    const options: {
        kind: PieceKind;
        value: "queen" | "rook" | "bishop" | "knight";
    }[] = [
        { kind: PieceKind.Queen, value: "queen" },
        { kind: PieceKind.Rook, value: "rook" },
        { kind: PieceKind.Bishop, value: "bishop" },
        { kind: PieceKind.Knight, value: "knight" },
    ];

    console.log(isOpen);

    return (
        isOpen && (
            <div className="w-screen h-screen bg-black/60">
                <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-[60%] h-fit grid-cols-4 grid gap-5 bg-sq-dark p-10!">
                    {options.map(({ kind, value }) => (
                        <button
                            key={kind}
                            type="button"
                            onClick={() => onSelect(value)}
                            className="bg-sq-light border border-black w-30 h-30 rounded-full flex justify-center items-center"
                        >
                            <PieceRenderer
                                piece={{ kind, color, square: "a1" }}
                            />
                        </button>
                    ))}
                </div>
            </div>
        )
    );
};

export default PromotionMenu;
