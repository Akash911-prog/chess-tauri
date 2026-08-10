import { useCallback, useRef, useState } from "react";
import { Piece } from "../types";
import Board from "../gameStates/board";

export function useBoard() {
    const boardRef = useRef<Board>(new Board());
    const [board, setBoard] = useState<Record<string, Piece | null>>(
        boardRef.current.board,
    );
    const finished = boardRef.current.finished;

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
        async (from: string, to: string) => {
            await boardRef.current.updateBoard(from, to);
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

    const setFinished = useCallback(
        (flag: boolean) => {
            boardRef.current.finished = flag;
            sync();
        },
        [sync],
    );

    const getFinishedState = useCallback(() => {
        return {
            condition: boardRef.current.condition,
            winColor: boardRef.current.winColor,
            msg: boardRef.current.msg,
        };
    }, []);

    return {
        board,
        getPiece,
        setPiece,
        removePiece,
        clear,
        updateBoard,
        undoMove,
        reset,
        finished,
        setFinished,
        getFinishedState,
    };
}
