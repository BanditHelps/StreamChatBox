import React, { useState } from 'react';
import './SettingsPanel.css';
import APIKeysWindow from './APIKeysWindow';

type DockPosition = 'left' | 'right' | 'top' | 'bottom' | 'none';

interface SettingsPanelProps {
  showActivityFeed: boolean;
  setShowActivityFeed: (show: boolean) => void;
  dockPosition: DockPosition;
  setDockPosition: (position: DockPosition) => void;
  dockSize: number;
  setDockSize: (size: number) => void;
  autoScroll: boolean;
  setAutoScroll: (autoScroll: boolean) => void;
}

const SettingsPanel: React.FC<SettingsPanelProps> = ({
  showActivityFeed,
  setShowActivityFeed,
  dockPosition,
  setDockPosition,
  dockSize,
  setDockSize,
  autoScroll,
  setAutoScroll,
}) => {
  const [showApiKeysWindow, setShowApiKeysWindow] = useState(false);

  const openApiKeysWindow = () => {
    setShowApiKeysWindow(true);
  };

  const closeApiKeysWindow = () => {
    setShowApiKeysWindow(false);
  };

  return (
    <div className="settings-panel">
      <div className="settings-section">
        <label className="settings-label">
          <input
            type="checkbox"
            checked={autoScroll}
            onChange={() => setAutoScroll(!autoScroll)}
          />
          Auto-scroll
        </label>
      </div>

      <div className="settings-section">
        <label className="settings-label">
          <input
            type="checkbox"
            checked={showActivityFeed}
            onChange={() => setShowActivityFeed(!showActivityFeed)}
          />
          Show Activity Feed
        </label>
      </div>

      {showActivityFeed && (
        <>
          <div className="settings-section">
            <label className="settings-label">Dock Position:</label>
            <select
              value={dockPosition}
              onChange={(e) => setDockPosition(e.target.value as DockPosition)}
              className="settings-select"
            >
              <option value="none">Disabled</option>
              <option value="left">Left</option>
              <option value="right">Right</option>
              <option value="top">Top</option>
              <option value="bottom">Bottom</option>
            </select>
          </div>

          {dockPosition !== 'none' && (
            <div className="settings-section">
              <label className="settings-label">
                Dock Size: {dockSize}%
              </label>
              <input
                type="range"
                min="20"
                max="60"
                value={dockSize}
                onChange={(e) => setDockSize(parseInt(e.target.value))}
                className="settings-slider"
              />
            </div>
          )}
        </>
      )}

      <div className="settings-section api-keys-section">
        <button 
          className="api-keys-button"
          onClick={openApiKeysWindow}
        >
          Configure API Keys
        </button>
      </div>

      {showApiKeysWindow && (
        <div className="modal-overlay" onClick={closeApiKeysWindow}>
          <div className="modal-content" onClick={(e) => e.stopPropagation()}>
            <button className="close-modal" onClick={closeApiKeysWindow}>×</button>
            <APIKeysWindow />
          </div>
        </div>
      )}
    </div>
  );
};

export default SettingsPanel; 