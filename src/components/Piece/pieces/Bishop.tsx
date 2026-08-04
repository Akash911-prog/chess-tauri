// Bishop.tsx
import BishopWhite from "../../../assets/pieces/bishop-w.svg?react";
import BishopBlack from "../../../assets/pieces/bishop-b.svg?react";
import { Color } from "../../../types";

const Bishop = ({ color }: { color: Color }) => {
    switch (color) {
        case Color.White:
            return <BishopWhite width={100} height={100} />;
        case Color.Black:
            return (
                <BishopBlack className="rotate-180" width={100} height={100} />
            );
    }
};

export default Bishop;
