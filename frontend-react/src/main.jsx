import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "./App.css";
import App from "./App.jsx";

// execute "npm run dev" to run on localhost.
createRoot(document.getElementById("root")).render(
  <StrictMode>
    <App />
  </StrictMode>
);