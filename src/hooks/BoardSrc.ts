import { useCallback, useRef, useState } from "react";
import { Piece } from "../types";
import Board from "../gameStates/board";

export function useBoard() {
    const boardRef = useRef<Board>(new Board());
    const [board, setBoard] = useState<Record<string, Piece | null>>(
        boardRef.current.board,
    );

    const sync = useCallback(() => {
        setBoard({ ...boardRef.current.board });
    }, []);

    const getPiece = useCallback((square: string) => {
        return boardRef.current.getPiece(square);
    }, []);

    const setPiece = useCallback(
        (square: string, piece: Piece | null) => {
            boardRef.current.setPiece(square, piece);
            sync();
        },
        [sync],
    );

    const removePiece = useCallback(
        (square: string) => {
            boardRef.current.removePiece(square);
            sync();
        },
        [sync],
    );

    const clear = useCallback(() => {
        boardRef.current.clear();
        sync();
    }, [sync]);

    const updateBoard = useCallback(
        (from: string, to: string) => {
            boardRef.current.updateBoard(from, to);
            sync();
        },
        [sync],
    );

    const undoMove = useCallback(() => {
        boardRef.current.undoMove();
        sync();
    }, [sync]);

    const reset = useCallback(() => {
        boardRef.current.reset();
        sync();
    }, [sync]);

    return {
        board,
        getPiece,
        setPiece,
        removePiece,
        clear,
        updateBoard,
        undoMove,
        reset,
    };
}
