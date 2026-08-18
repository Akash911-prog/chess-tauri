import { createBrowserRouter } from "react-router";
import MainMenu from "./pages/mainMenu";
import Game from "./pages/game";
import Vsai from "./pages/vsai";

export const router = createBrowserRouter([
    {
        path: "/",
        element: <MainMenu />,
    },
    {
        path: "/game",
        element: <Game />,
    },
    {
        path: "/vsai",
        element: <Vsai />,
    },
]);
