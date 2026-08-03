// import { useState } from "react";
// import { invoke } from "@tauri-apps/api/core";
import { RouterProvider } from "react-router/dom";
import { router } from "./routes";
import { useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

function App() {
    useEffect(() => {
        invoke("show_window");
    }, []);
    return <RouterProvider router={router} />;
}

export default App;
