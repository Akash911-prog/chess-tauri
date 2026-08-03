// Rook.tsx
import RookWhite from "../../../assets/pieces/rook-w.svg?react";
import RookBlack from "../../../assets/pieces/rook-b.svg?react";
import { Color } from "../../../types";

const Rook = ({ color }: { color: Color }) => {
    switch (color) {
        case Color.White:
            return <RookWhite width={128} height={128} />;
        case Color.Black:
            return (
                <RookBlack className="rotate-180" width={128} height={128} />
            );
    }
};

export default Rook;
