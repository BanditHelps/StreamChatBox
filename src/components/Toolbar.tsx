import React, { useState } from 'react';
import { ArrowDownCircle, Layout, Settings, X } from 'lucide-react';
import './Toolbar.css';
import SettingsPanel from './SettingsPanel';

type DockPosition = 'left' | 'right' | 'top' | 'bottom' | 'none';

interface ToolbarProps {
  showActivityFeed: boolean;
  setShowActivityFeed: (show: boolean) => void;
  dockPosition: DockPosition;
  setDockPosition: (position: DockPosition) => void;
  dockSize: number;
  setDockSize: (size: number) => void;
  autoScroll: boolean;
  setAutoScroll: (autoScroll: boolean) => void;
}

const Toolbar: React.FC<ToolbarProps> = ({
  showActivityFeed,
  setShowActivityFeed,
  dockPosition,
  setDockPosition,
  dockSize,
  setDockSize,
  autoScroll,
  setAutoScroll,
}) => {
  const [showSettings, setShowSettings] = useState(false);

  return (
    <div className="toolbar">
      <div className="toolbar-buttons">
        <button 
          className={`toolbar-button ${autoScroll ? 'active' : ''}`}
          onClick={() => setAutoScroll(!autoScroll)}
          title="Auto Scroll"
        >
          <ArrowDownCircle size={18} />
        </button>
        
        <button 
          className={`toolbar-button ${showActivityFeed ? 'active' : ''}`}
          onClick={() => setShowActivityFeed(!showActivityFeed)}
          title="Activity Feed"
        >
          <Layout size={18} />
        </button>
        
        <button 
          className={`toolbar-button ${showSettings ? 'active' : ''}`}
          onClick={() => setShowSettings(!showSettings)}
          title="Settings"
        >
          <Settings size={18} />
        </button>
      </div>
      
      {showSettings && (
        <div className="settings-dropdown">
          <div className="settings-dropdown-header">
            <h3>Settings</h3>
            <button 
              className="close-button"
              onClick={() => setShowSettings(false)}
            >
              <X size={16} />
            </button>
          </div>
          
          <SettingsPanel 
            showActivityFeed={showActivityFeed}
            setShowActivityFeed={setShowActivityFeed}
            dockPosition={dockPosition}
            setDockPosition={setDockPosition}
            dockSize={dockSize}
            setDockSize={setDockSize}
            autoScroll={autoScroll}
            setAutoScroll={setAutoScroll}
          />
        </div>
      )}
    </div>
  );
};

export default Toolbar; 