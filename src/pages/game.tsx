import { ChevronLeft } from "lucide-react";
import { useNavigate } from "react-router";

const Game = () => {
    let navigate = useNavigate();
    return (
        <div>
            <div className="heading w-screen max-w-350 min-w-75">
                <ChevronLeft onClick={() => navigate("/")} />
            </div>
        </div>
    );
};

export default Game;
