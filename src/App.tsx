// import { useState } from "react";
// import { invoke } from "@tauri-apps/api/core";
import { RouterProvider } from "react-router/dom";
import { router } from "./routes";

function App() {
    return <RouterProvider router={router} />;
}

export default App;
