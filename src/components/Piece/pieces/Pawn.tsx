// Pawn.tsx
import PawnWhite from "../../../assets/pieces/pawn-w.svg?react";
import PawnBlack from "../../../assets/pieces/pawn-b.svg?react";
import { Color } from "../../../types";

const Pawn = ({ color }: { color: Color }) => {
    switch (color) {
        case Color.White:
            return <PawnWhite width={128} height={128} />;
        case Color.Black:
            return (
                <PawnBlack className="rotate-180" width={128} height={128} />
            );
    }
};

export default Pawn;
