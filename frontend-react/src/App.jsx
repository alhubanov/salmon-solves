import { useState, useEffect } from "react";
import Sidebar from "./components/Sidebar";
import CrosswordGrid from "./components/CrosswordGrid";
import "./App.css";
import init from "../../pkg/crossy"

const DEFAULT_SETTINGS = {
  type: "Scandi", // "Simple" is not selectable yet
  difficulty: "Beginner", // fixed while the difficulty picker is hidden
  grid: "15x15",
  themes: [], // empty = random
};

export default function App() 
{
  const [sidebarOpen, setSidebarOpen] = useState(true);
  const [settings, setSettings] = useState(DEFAULT_SETTINGS);
  // A counter rather than a flag: each press is a fresh request, so pressing Generate again with
  // the same settings still rebuilds, while editing settings on its own does nothing.
  const [generateRequest, setGenerateRequest] = useState(0);
  const [wasmReady, setWasmReady] = useState(false);

  useEffect(() => 
  {
    async function loadWasm() 
    {
      await init();
      setWasmReady(true);
    }

    loadWasm();
  }, []);

  function updateSetting(key, value) 
  {
    setSettings((prev) => ({ ...prev, [key]: value }));
  }

  function handleGenerate()
  {
    if (!wasmReady) return;
    setGenerateRequest((request) => request + 1);
  }

  return (
    <div className="app-layout">
      <div className="app-body">
        <aside className={`sidebar ${sidebarOpen ? "sidebar--open" : "sidebar--closed"}`}>
          <Sidebar
            settings={settings}
            onUpdate={updateSetting}
            onGenerate={handleGenerate}
            sidebarOpen={sidebarOpen}
            onToggle={() => setSidebarOpen((o) => !o)}
          />
        </aside>

        <main className="main-area">
          <CrosswordGrid settings={settings} generateRequest={generateRequest} />
        </main>
      </div>
    </div>
  );
}