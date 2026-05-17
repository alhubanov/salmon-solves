import { useState } from "react";
import Sidebar from "./components/Sidebar";
import CrosswordGrid from "./components/CrosswordGrid";
import TopBar from "./components/TopBar";
import "./App.css";

const DEFAULT_SETTINGS = {
  type: "american",
  difficulty: "beginner",
  grid: "15x15",
  themes: [], // empty = random
};

export default function App() {
  const [sidebarOpen, setSidebarOpen] = useState(true);
  const [settings, setSettings] = useState(DEFAULT_SETTINGS);
  const [generated, setGenerated] = useState(false);

  function updateSetting(key, value) {
    setSettings((prev) => ({ ...prev, [key]: value }));
  }

  function handleGenerate() {
    // Later: call your crossword generation API here
    setGenerated(true);
  }

  function handleReset() {
    setSettings(DEFAULT_SETTINGS);
    setGenerated(false);
  }

  return (
    <div className="app-layout">
      <TopBar sidebarOpen={sidebarOpen} onToggle={() => setSidebarOpen((o) => !o)} />

      <div className="app-body">
        {sidebarOpen && (
          <aside className="sidebar">
            <Sidebar
              settings={settings}
              onUpdate={updateSetting}
              onGenerate={handleGenerate}
              onReset={handleReset}
            />
          </aside>
        )}

        <main className="main-area">
          <CrosswordGrid settings={settings} generated={generated} />
        </main>
      </div>
    </div>
  );
}