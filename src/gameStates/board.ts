import { invoke } from "@tauri-apps/api/core";
import { Piece } from "../types";
import { INITIAL_BOARD } from "./constants";
import { MoveInfo } from "../dto";

class Board {
    private _board: Record<string, Piece | null>;
    private _prevboardStack: Record<string, Piece | null>[];

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
        console.log(legal);
        if (!legal[0]) {
            return;
        }
        this.prevboardStack.push({ ...this._board });
        if (legal[1] && legal[1][0]) {
            let from = legal[1][1][0];
            let to = legal[1][1][1];
            this._board[to] = this._board[from];
            this._board[from] = null;
        }
        this._board[to] = this._board[from];
        this._board[from] = null;
    }

    private async checkMove(
        from: string,
        to: string,
    ): Promise<[boolean, [boolean, [string, string]] | null]> {
        const moveInfo: MoveInfo = { from, to };

        try {
            const result = await invoke<
                | { kind: string; data: Array<boolean | Array<string>> }
                | { kind: string }
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

export default Board;
