import { useState, useEffect } from "react";
import Sidebar from "./components/Sidebar";
import CrosswordGrid from "./components/CrosswordGrid";
import "./App.css";
import init from "../../pkg/crossy"

const DEFAULT_SETTINGS = {
  type: "Simple",
  difficulty: "Beginner",
  grid: "15x15",
  themes: [], // empty = random
};

export default function App() 
{
  const [sidebarOpen, setSidebarOpen] = useState(true);
  const [settings, setSettings] = useState(DEFAULT_SETTINGS);
  const [generated, setGenerated] = useState(false);
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
    setGenerated(true);
  }

  function handleReset() 
  {
    setSettings(DEFAULT_SETTINGS);
    setGenerated(false);
  }

  return (
    <div className="app-layout">
      <div className="app-body">
        <aside className={`sidebar ${sidebarOpen ? "sidebar--open" : "sidebar--closed"}`}>
          <Sidebar
            settings={settings}
            onUpdate={updateSetting}
            onGenerate={handleGenerate}
            onReset={handleReset}
            sidebarOpen={sidebarOpen}
            onToggle={() => setSidebarOpen((o) => !o)}
          />
        </aside>

        <main className="main-area">
          <CrosswordGrid settings={settings} generated={generated} />
        </main>
      </div>
    </div>
  );
}