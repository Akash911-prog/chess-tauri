import { invoke } from "@tauri-apps/api/core";
import { Piece, Response, toPiece } from "../types";
import { INITIAL_BOARD } from "./constants";
import { MoveInfo } from "../dto";

class Board {
    private _board: Record<string, Piece | null>;
    private _prevboardStack: Record<string, Piece | null>[];
    public finished: boolean = false;
    public condition: string = "inprogress";
    public winColor: string = "";
    public msg: Record<string, string> = {};

    constructor() {
        this._board = { ...INITIAL_BOARD };
        this._prevboardStack = [];
        this.msg = {
            stalemate: "Stalemate! It's a draw.",
        };
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

        if (!data) {
            return;
        }

        data.changes.forEach((change) => {
            this._board[change.square] = toPiece(change);
        });

        if (data.condition === "checkmate") {
            this.finished = true;
            this.condition = data.condition;
            this.winColor = data.winner;
            this.msg[data.condition] = `Checkmate! ${data.winner} wins.`;
        } else if (data.condition === "stalemate") {
            this.finished = true;
            this.condition = data.condition;
            this.winColor = "";
        }
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

            return [result.kind === "legal", result.data ? result.data : null];
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
