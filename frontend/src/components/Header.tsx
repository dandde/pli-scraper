import React from 'react';
import { Menu } from 'lucide-react';
import './Header.css';

interface HeaderProps {
    onToggleSidebar: () => void;
}

export const Header: React.FC<HeaderProps> = ({ onToggleSidebar }) => {
    return (
        <header className="header">
            <div className="header-left">
                <button onClick={onToggleSidebar} className="menu-btn" title="Toggle Sidebar">
                    <Menu className="icon" />
                </button>
                <span className="app-title">Scapi / Frontend</span>
            </div>
            <div className="header-right">
                {/* Placeholder for actions */}
            </div>
        </header>
    );
};
