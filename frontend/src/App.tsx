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
  const [currentUrl, setCurrentUrl] = useState<string>('');

  const toggleSidebar = () => setSidebarOpen(!sidebarOpen);

  const analyzeUrl = async (url: string) => {
    setIsLoading(true);
    setData(null);
    setCurrentUrl(url); // Store the URL being analyzed
    try {
      const apiBase = import.meta.env.VITE_API_URL || 'http://localhost:3000';
      const endpoint = `${apiBase}/api/report/${url}`;
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
        <MainContent data={data} currentUrl={currentUrl} />
      </div>
    </div>
  );
}

export default App;
