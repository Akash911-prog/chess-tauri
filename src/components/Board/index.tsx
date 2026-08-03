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

    console.log(board);

    return (
        <div
            className="board w-[80vw] h-[80vw] grid grid-cols-8 grid-rows-8"
            ref={boardRef}
        >
            {board &&
                Object.entries(board)
                    .reverse()
                    .map(([square, piece], idx) => (
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
