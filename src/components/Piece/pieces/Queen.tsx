// Queen.tsx
import QueenWhite from "../../../assets/pieces/queen-w.svg?react";
import QueenBlack from "../../../assets/pieces/queen-b.svg?react";
import { Color } from "../../../types";

const Queen = ({ color }: { color: Color }) => {
    switch (color) {
        case Color.White:
            return <QueenWhite width={128} height={128} />;
        case Color.Black:
            return (
                <QueenBlack className="rotate-180" width={128} height={128} />
            );
    }
};

export default Queen;
