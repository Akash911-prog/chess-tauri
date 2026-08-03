import { motion } from "motion/react";
import { PieceKind, Piece as PieceType } from "../../types";
import Piece from "../Piece/index";

type Props = {
    square: string;
    piece: PieceType | null;
    idx: number;
    boardRef: React.RefObject<HTMLDivElement | null>;
    updateBoard: (from: string, to: string) => void;
    boardVersion: number;
    setBoardVersion: React.Dispatch<React.SetStateAction<number>>;
};

const SNAPPY_TRANSITION = {
    type: "spring",
    stiffness: 700,
    damping: 30,
} as const;

const FILES = ["a", "b", "c", "d", "e", "f", "g", "h"];

const Square = ({
    square,
    piece,
    idx,
    boardRef,
    updateBoard,
    boardVersion,
    setBoardVersion,
}: Props) => {
    const file = idx % 8;
    const rank = Math.floor(idx / 8);
    const isLight = (file + rank) % 2 === 0;

    const onDragEnd = (event, info) => {
        const boardRect = boardRef.current?.getBoundingClientRect();
        if (!boardRect) return;

        const x = info.point.x - boardRect.left;
        const y = info.point.y - boardRect.top;
        const squareSize = boardRect.width / 8;

        const fileIdx = Math.min(7, Math.max(0, Math.floor(x / squareSize)));

        const rankIdx = Math.min(7, Math.max(0, Math.floor(y / squareSize))); // 0 = top row (rank 8, if using your RANKS order)

        console.log(fileIdx, rankIdx);

        // clamp to board bounds
        if (fileIdx < 0 || fileIdx > 7 || rankIdx < 0 || rankIdx > 7) {
            setBoardVersion((prev) => prev + 1);
            return; // dropped off board — piece will spring back automatically
        }

        const targetSquare = `${FILES[7 - fileIdx]}${8 - rankIdx}`;

        console.log(targetSquare, square);

        if (targetSquare === square) {
            setBoardVersion((prev) => prev + 1);
            return;
        }
        // call your move logic here, e.g. updateBoard(currentSquare, targetSquare)
        updateBoard(square, targetSquare);
        setBoardVersion((prev) => prev + 1);
        return;
    };

    return (
        <div
            className={`
                relative
                aspect-square
                w-full
                flex items-center justify-center
                ${isLight ? "bg-sq-light" : "bg-sq-dark"}
                group
            `}
        >
            {/* subtle inset shading for depth, wood-board feel */}
            <div className="absolute inset-0 shadow-[inset_0_0_6px_rgba(0,0,0,0.15)] pointer-events-none" />

            {/* coordinate label, only on edge squares */}
            {file === 0 && (
                <span
                    className={`
                        absolute top-0.5 left-1
                        text-[0.6rem] font-mono font-medium
                        ${isLight ? "text-(--sq-dark)" : "text-(--sq-light)"}
                        opacity-60
                        select-none
                    `}
                >
                    {square[1]}
                </span>
            )}
            {rank === 7 && (
                <span
                    className={`
                        absolute bottom-0.5 right-1
                        text-[0.6rem] font-mono font-medium
                        ${isLight ? "text-(--sq-dark)" : "text-(--sq-light)"}
                        opacity-60
                        select-none
                    `}
                >
                    {square[0]}
                </span>
            )}

            {/* hover highlight ring, purely visual — no click logic yet */}
            <div
                className="
                    absolute inset-0
                    ring-inset ring-0
                    group-hover:ring-2
                    group-hover:ring-(--accent-brass,#c9a24b)
                    transition-all duration-150
                    pointer-events-none
                "
            />

            {piece && (
                <motion.div
                    key={`${square}-${boardVersion}`}
                    drag
                    dragConstraints={boardRef}
                    dragMomentum={false}
                    dragElastic={0.15}
                    onDragEnd={onDragEnd}
                    whileDrag={{
                        scale: piece.kind === PieceKind.King ? 1.5 : 1.1,
                        zIndex: 50,
                    }}
                    transition={SNAPPY_TRANSITION}
                    className="relative z-10 w-[85%] h-[85%] flex items-center justify-center"
                >
                    <Piece piece={piece} />
                </motion.div>
            )}
        </div>
    );
};

export default Square;
