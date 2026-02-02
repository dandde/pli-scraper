import React, { useState } from 'react';
import { Search, ChevronRight, ChevronDown, Tag as TagIcon } from 'lucide-react';
import type { AnalysisResult, TagStats } from '../types';
import './Sidebar.css';

interface SidebarProps {
    isOpen: boolean;
    onAnalyze: (url: string) => void;
    isLoading: boolean;
    data: AnalysisResult | null;
}

export const Sidebar: React.FC<SidebarProps> = ({ isOpen, onAnalyze, isLoading, data }) => {
    const [url, setUrl] = useState('');

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault();
        if (url) onAnalyze(url);
    };

    return (
        <aside className={`sidebar ${isOpen ? 'open' : 'closed'}`}>
            <div className="sidebar-search">
                <form onSubmit={handleSubmit} className="search-form">
                    <input
                        type="text"
                        value={url}
                        onChange={(e) => setUrl(e.target.value)}
                        placeholder="Enter URL..."
                        className="search-input"
                    />
                    <button type="submit" className="search-btn" disabled={isLoading}>
                        <Search className="icon" />
                    </button>
                </form>
            </div>

            <div className="sidebar-content">
                {isLoading && <div className="loading">Analyzing...</div>}

                {!isLoading && data && (
                    <div className="tree-view">
                        <h3>Analysis Result</h3>
                        <div className="stats-meta">
                            <p>Files: {data.files_analyzed}</p>
                            <p>Max Depth: {data.max_depth}</p>
                        </div>

                        <div className="tree-root">
                            {Object.entries(data.tags).map(([tagName, stats]) => (
                                <TagNode key={tagName} name={tagName} stats={stats} />
                            ))}
                        </div>
                    </div>
                )}
            </div>
        </aside>
    );
};

// Simple Tree Node for Tags
const TagNode: React.FC<{ name: string; stats: TagStats }> = ({ name, stats }) => {
    const [expanded, setExpanded] = useState(false);
    const hasAttributes = Object.keys(stats.attributes).length > 0;

    return (
        <div className="tree-node">
            <div
                className={`node-header ${expanded ? 'active' : ''}`}
                onClick={() => hasAttributes && setExpanded(!expanded)}
            >
                <span className="node-icon">
                    {hasAttributes ? (expanded ? <ChevronDown size={14} /> : <ChevronRight size={14} />) : <TagIcon size={14} />}
                </span>
                <span className="node-name">{name}</span>
                <span className="node-badge">{stats.count}</span>
            </div>

            {expanded && hasAttributes && (
                <div className="node-children">
                    {Object.entries(stats.attributes).map(([attrName, attrStats]) => (
                        <div key={attrName} className="attribute-row">
                            <span className="attr-name">{attrName}</span>
                            <span className="attr-count">{attrStats.count}</span>
                        </div>
                    ))}
                </div>
            )}
        </div>
    );
};
