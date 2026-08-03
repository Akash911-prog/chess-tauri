import { useRef, useState } from "react";
import { useBoard } from "../../hooks/BoardSrc";
import Square from "../Square";

const Board = ({ boardData }: { boardData: ReturnType<typeof useBoard> }) => {
    let {
        board,
        clear,
        getPiece,
        removePiece,
        setPiece,
        undoMove,
        updateBoard,
    } = boardData;

    const boardRef = useRef<HTMLDivElement>(null);

    const [boardVersion, setBoardVersion] = useState(0);

    const entries = Object.entries(board);

    const ordered = [];

    for (let i = 56; i >= 0; i -= 8) {
        ordered.push(...entries.slice(i, i + 8));
    }

    return (
        <div
            className="board w-[80vw] h-[80vw] grid grid-cols-8 grid-rows-8"
            ref={boardRef}
        >
            {board &&
                ordered.map(([square, piece], idx) => (
                    <Square
                        key={square}
                        square={square}
                        piece={piece}
                        idx={idx}
                        boardRef={boardRef}
                        updateBoard={updateBoard}
                        boardVersion={boardVersion}
                        setBoardVersion={setBoardVersion}
                    />
                ))}
        </div>
    );
};

export default Board;
