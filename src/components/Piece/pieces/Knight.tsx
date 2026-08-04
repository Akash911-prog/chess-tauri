// Knight.tsx
import KnightWhite from "../../../assets/pieces/knight-w.svg?react";
import KnightBlack from "../../../assets/pieces/knight-b.svg?react";
import { Color } from "../../../types";

const Knight = ({ color }: { color: Color }) => {
    switch (color) {
        case Color.White:
            return <KnightWhite width={100} height={100} />;
        case Color.Black:
            return (
                <KnightBlack className="rotate-180" width={100} height={100} />
            );
    }
};

export default Knight;
