import { useState } from 'react';
import { Header } from './components/Header';
import { Sidebar } from './components/Sidebar';
import { MainContent } from './components/MainContent';
import type { AnalysisResult } from './types';
import './App.css';

function App() {
  const [sidebarOpen, setSidebarOpen] = useState(true);
  const [isLoading, setIsLoading] = useState(false);
  const [data, setData] = useState<AnalysisResult | null>(null);

  const toggleSidebar = () => setSidebarOpen(!sidebarOpen);

  const analyzeUrl = async (url: string) => {
    setIsLoading(true);
    setData(null);
    try {
      // Ensure URL is encoded
      // Need strict check for protocol in frontend or backend? Backend checks.
      // But passing encoded url to backend:
      // http://localhost:3000/api/report/https://example.com
      // Wait, my backend implementation in main.rs:
      // .route("/api/report/*url", get(handler_report))
      // Path<String> will capture the rest.
      // If I encode it, say http%3A%2F%2F..., axum might treat it as one segment or decode it?
      // If I don't encode it:
      // /api/report/https://example.com
      // Axum *url will capture "https://example.com".

      const endpoint = `http://localhost:3000/api/report/${url}`;
      const response = await fetch(endpoint);

      if (!response.ok) {
        throw new Error(`Error: ${response.statusText}`);
      }

      const result: AnalysisResult = await response.json();
      setData(result);
    } catch (error) {
      console.error('Analysis failed', error);
      alert('Failed to analyze URL. Ensure scapi is running on port 3000.');
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="app">
      <Header onToggleSidebar={toggleSidebar} />
      <div className="app-body">
        <Sidebar
          isOpen={sidebarOpen}
          onAnalyze={analyzeUrl}
          isLoading={isLoading}
          data={data}
        />
        <MainContent />
      </div>
    </div>
  );
}

export default App;
