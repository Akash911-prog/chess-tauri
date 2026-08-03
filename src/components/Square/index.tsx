import { Piece } from "../../types";

type Props = {
    square: string;
    piece: Piece | null;
    idx: number;
};

const Square = ({ square, piece, idx }: Props) => {
    return <div>Square</div>;
};

export default Square;
