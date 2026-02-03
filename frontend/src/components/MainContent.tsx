import React, { useState, useEffect } from 'react';
import { Table, Code, FileCode, Network } from 'lucide-react';
import type { AnalysisResult } from '../types';
import './MainContent.css';

interface MainContentProps {
    data: AnalysisResult | null;
    currentUrl: string;
}

export const MainContent: React.FC<MainContentProps> = ({ data, currentUrl }) => {
    const [view, setView] = useState<'csv' | 'json' | 'graph'>('json');
    const [csvContent, setCsvContent] = useState<string[][]>([]);

    useEffect(() => {
        if (data) {
            setView('json');
        }
    }, [data]);

    useEffect(() => {
        if (view === 'csv' && currentUrl) {
            const apiBase = import.meta.env.VITE_API_URL || 'http://localhost:3000';
            fetch(`${apiBase}/api/export/${currentUrl}?format=csv`)
                .then(res => res.text())
                .then(text => {
                    const rows = text.trim().split('\n').map(row => row.split(','));
                    setCsvContent(rows);
                })
                .catch(console.error);
        }
    }, [view, currentUrl]);

    if (!data) {
        return (
            <main className="main-content">
                <div className="welcome-card">
                    <h1>Welcome to Scapi</h1>
                    <p>Enter a URL in the sidebar to begin analyzing its HTML structure.</p>
                    <div className="features">
                        <div className="feature-item">
                            <h3>Tree Analysis</h3>
                            <p>Visualize the DOM structure and tag usage.</p>
                        </div>
                        <div className="feature-item">
                            <h3>Export Data</h3>
                            <p>View reports in CSV, HTML, JSON, Graphviz format.</p>
                        </div>
                    </div>
                </div>
            </main>
        );
    }

    const apiBase = import.meta.env.VITE_API_URL || 'http://localhost:3000';
    const iframeSrc = (format: string) => `${apiBase}/api/export/${currentUrl}?format=${format}`;

    return (
        <main className="main-content">
            <div className="view-controls">
                <div className="toggle-group">
                    <button className={view === 'csv' ? 'active' : ''} onClick={() => setView('csv')} title="CSV Table">
                        <Table size={18} /> CSV
                    </button>
                    <button className={view === 'json' ? 'active' : ''} onClick={() => setView('json')} title="JSON Source">
                        <Code size={18} /> JSON
                    </button>
                    <button className={view === 'graph' ? 'active' : ''} onClick={() => setView('graph')} title="Graphviz">
                        <Network size={18} /> Graph
                    </button>
                </div>
            </div>

            <div className="view-container">
                {view === 'json' && (
                    <pre className="code-view">
                        <code>{JSON.stringify(data, null, 2)}</code>
                    </pre>
                )}

                {view === 'csv' && (
                    <div className="table-view-container">
                        <table className="csv-table">
                            <thead>
                                <tr>
                                    {csvContent[0]?.map((header, i) => (
                                        <th key={i}>{header}</th>
                                    ))}
                                </tr>
                            </thead>
                            <tbody>
                                {csvContent.slice(1).map((row, i) => (
                                    <tr key={i}>
                                        {row.map((cell, j) => (
                                            <td key={j}>{cell}</td>
                                        ))}
                                    </tr>
                                ))}
                            </tbody>
                        </table>
                    </div>
                )}

                {view === 'graph' && (
                    <iframe
                        title="Report View"
                        src={iframeSrc(view)}
                        className="report-iframe"
                    />
                )}
            </div>
        </main>
    );
};
