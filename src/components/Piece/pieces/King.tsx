// King.tsx
import KingWhite from "../../../assets/pieces/king-w.svg?react";
import KingBlack from "../../../assets/pieces/king-b.svg?react";
import { Color } from "../../../types";

const King = ({ color }: { color: Color }) => {
    switch (color) {
        case Color.White:
            return <KingWhite width={100} height={100} />;
        case Color.Black:
            return (
                <KingBlack className="rotate-180" width={100} height={100} />
            );
    }
};

export default King;
