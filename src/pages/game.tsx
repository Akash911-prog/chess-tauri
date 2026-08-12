import { ChevronLeft } from "lucide-react";
import { useNavigate } from "react-router";
import Board from "../components/Board";
import Button from "../components/Button";
import { useBoard } from "../hooks/BoardSrc";
import PromotionMenu from "../components/PromotionMenu";
import { useState } from "react";
import { Color } from "../types";

const Game = () => {
    let navigate = useNavigate();

    let board = useBoard();

    let [needPromotion, setNeedPromotion] = useState<[boolean, Color]>([
        false,
        Color.White,
    ]);

    return (
        <div className="w-screen h-screen">
            <div className="heading w-screen max-w-350 min-w-75">
                <ChevronLeft
                    onClick={() => navigate("/")}
                    className="size-10"
                />
            </div>

            <div className="relative w-[80vw] left-1/2 -translate-x-1/2 top-1/2 -translate-y-1/2 flex-col flex gap-5">
                <Board boardData={board} setNeedPromotion={setNeedPromotion} />

                <div className="flex justify-around">
                    <Button scheme="secondary" className="w-[200px] h-[70px]">
                        Resign
                    </Button>
                    <Button
                        scheme="secondary"
                        className="w-[200px] h-[70px]"
                        onClick={() => board.reset()}
                    >
                        reset
                    </Button>
                    <Button
                        scheme="secondary"
                        className="w-[200px] h-[70px]"
                        onClick={() => {
                            board.undoMove();
                            console.log("undo");
                        }}
                    >
                        Undo
                    </Button>
                </div>
            </div>

            <PromotionMenu color={needPromotion[1]} isOpen={needPromotion[0]} />
        </div>
    );
};

export default Game;
