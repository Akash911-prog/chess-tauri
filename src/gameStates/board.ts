import { invoke } from "@tauri-apps/api/core";
import { Piece } from "../types";
import { INITIAL_BOARD } from "./constants";
import { MoveInfo } from "../dto";

class Board {
    private _board: Record<string, Piece | null>;
    private _prevboardStack: Record<string, Piece | null>[];

    constructor() {
        this._board = INITIAL_BOARD;
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
        this._board = INITIAL_BOARD;
        this._prevboardStack = [];
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
        if (!legal) {
            return;
        }
        this.prevboardStack.push({ ...this._board });
        this._board[to] = this._board[from];
        this._board[from] = null;
    }

    private async checkMove(from: string, to: string): Promise<boolean> {
        const moveInfo: MoveInfo = { from, to };

        try {
            const result = await invoke<"Legal" | "Illegal">("update", {
                moveInfo,
            });

            console.log(result);
            return result === "Legal";
        } catch (error) {
            console.error(error);
            return false;
        }
    }
    undoMove(): void {
        if (this.prevboardStack.length === 0) return;
        this._board = this.prevboardStack.pop() as Record<string, Piece | null>;
    }
}

export default Board;
