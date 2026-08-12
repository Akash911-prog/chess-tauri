import { useRef, useState } from "react";
import { useBoard } from "../../hooks/BoardSrc";
import Square from "../Square";
import GameOverModal from "../GameOverScreen";
import { useNavigate } from "react-router";
import { PendingPromotion } from "../../types";

const Board = ({
    boardData,
    setPendingPromotion,
}: {
    boardData: ReturnType<typeof useBoard>;
    setPendingPromotion: React.Dispatch<
        React.SetStateAction<PendingPromotion | null>
    >;
}) => {
    let {
        board,
        clear,
        getPiece,
        removePiece,
        setPiece,
        undoMove,
        reset,
        updateBoard,
        finished,
        setFinished,
        getFinishedState,
        needsPromotion,
    } = boardData;

    let navigate = useNavigate();

    const boardRef = useRef<HTMLDivElement>(null);

    const [boardVersion, setBoardVersion] = useState(0);

    const entries = Object.entries(board);

    const ordered = [];

    for (let i = 56; i >= 0; i -= 8) {
        ordered.push(...entries.slice(i, i + 8));
    }

    let { condition, msg } = getFinishedState();

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
                        needsPromotion={needsPromotion}
                        setPendingPromotion={setPendingPromotion}
                    />
                ))}

            <GameOverModal
                isOpen={finished}
                message={msg[condition]}
                title={condition.toUpperCase()}
                onRestart={() => {
                    setFinished(false);
                    reset();
                }}
                onMainMenu={() => {
                    reset();
                    navigate("/");
                }}
            />
        </div>
    );
};

export default Board;
