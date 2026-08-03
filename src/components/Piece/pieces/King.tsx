// King.tsx
import KingWhite from "../../../assets/pieces/king-w.svg?react";
import KingBlack from "../../../assets/pieces/king-b.svg?react";
import { Color } from "../../../types";

const King = ({ color }: { color: Color }) => {
    switch (color) {
        case Color.White:
            return <KingWhite width={128} height={128} />;
        case Color.Black:
            return (
                <KingBlack className="rotate-180" width={128} height={128} />
            );
    }
};

export default King;
