// Queen.tsx
import QueenWhite from "../../../assets/pieces/queen-w.svg?react";
import QueenBlack from "../../../assets/pieces/queen-b.svg?react";
import { Color } from "../../../types";

const Queen = ({ color }: { color: Color }) => {
    switch (color) {
        case Color.White:
            return <QueenWhite width={100} height={100} />;
        case Color.Black:
            return (
                <QueenBlack className="rotate-180" width={100} height={100} />
            );
    }
};

export default Queen;
