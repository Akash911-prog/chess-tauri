// Knight.tsx
import KnightWhite from "../../../assets/pieces/knight-w.svg?react";
import KnightBlack from "../../../assets/pieces/knight-b.svg?react";
import { Color } from "../../../types";

const Knight = ({ color }: { color: Color }) => {
    switch (color) {
        case Color.White:
            return <KnightWhite width={128} height={128} />;
        case Color.Black:
            return (
                <KnightBlack className="rotate-180" width={128} height={128} />
            );
    }
};

export default Knight;
