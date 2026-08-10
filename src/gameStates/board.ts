import { invoke } from "@tauri-apps/api/core";
import { Piece } from "../types";
import { INITIAL_BOARD } from "./constants";
import { MoveInfo } from "../dto";

class Board {
    private _board: Record<string, Piece | null>;
    private _prevboardStack: Record<string, Piece | null>[];
    public finished: boolean = false;

    constructor() {
        this._board = { ...INITIAL_BOARD };
        this._prevboardStack = [];
    }

    get board(): Record<string, Piece | null> {
        return this._board;
    }

    set board(value: Record<string, Piece | null>) {
        this._board = value;
    }

    get prevboardStack(): Record<string, Piece | null>[] {
        return this._prevboardStack;
    }

    set prevboardStack(value: Record<string, Piece | null>[]) {
        this._prevboardStack = value;
    }

    reset(): void {
        this._board = { ...INITIAL_BOARD };
        this._prevboardStack = [];
        invoke("restart");
    }

    getPiece(square: string): Piece | null {
        return this._board[square];
    }

    setPiece(square: string, piece: Piece | null): void {
        this._board[square] = piece;
    }

    removePiece(square: string): void {
        this._board[square] = null;
    }

    clear(): void {
        this._board = {};
    }

    async updateBoard(from: string, to: string): Promise<void> {
        let legal = await this.checkMove(from, to);
        console.log(legal); // [true, [true, ["e2", "e4"]]]
        if (!legal[0]) {
            return;
        }
        this.prevboardStack.push({ ...this._board });
        let data = legal[1];
        if (data?.moveType == "castling") {
            this._board[data.to] = this._board[data.from];
            this._board[data.from] = null;
        }

        if (data?.moveType == "enPassant") {
            this._board[data.to] = null;
        }

        if (data?.condition != "inprogress") {
            this.finished = true;
            return;
        }
        this._board[to] = this._board[from];
        this._board[from] = null;
    }

    private async checkMove(
        from: string,
        to: string,
    ): Promise<[boolean, Response | null]> {
        const moveInfo: MoveInfo = { from, to };

        try {
            const result = await invoke<
                | { kind: string; data: Response }
                | { kind: string; data: undefined }
            >("update", {
                moveInfo,
            });

            return [result.kind === "Legal", result.data ? result.data : null];
        } catch (error) {
            console.error(error);
            return [false, null];
        }
    }
    undoMove(): void {
        if (this.prevboardStack.length === 0) return;
        this._board = this.prevboardStack.pop() as Record<string, Piece | null>;
        invoke("undo_move");
    }
}

type Response = {
    moveType: "normal" | "castling" | "promotion" | "enPassant";
    from: string;
    to: string;
    promotion: string | null;
    condition: "inprogress" | "Draw" | "checkmate" | "stalemate";
    check: number | null;
};

export default Board;
