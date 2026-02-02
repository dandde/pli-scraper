import React from 'react';
import './MainContent.css';

export const MainContent: React.FC = () => {
    return (
        <main className="main-content">
            <div className="welcome-card">
                <h1>Welcome to Scapi</h1>
                <p>Enter a URL in the sidebar to begin analyzing its HTML structure.</p>
                <div className="features">
                    <div className="feature-item">
                        <h3>Tree Analysis</h3>
                        <p> visualize the DOM structure and tag usage.</p>
                    </div>
                    <div className="feature-item">
                        <h3>Export Data</h3>
                        <p>Download reports in CSV, HTML, or JSON formats.</p>
                    </div>
                </div>
            </div>
        </main>
    );
};
