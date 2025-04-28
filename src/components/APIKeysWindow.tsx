import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import './APIKeysWindow.css';

interface APIKey {
  name: string;
  key: string;
  placeholder: string;
  visible: boolean;
}

const APIKeysWindow: React.FC = () => {
  const [apiKeys, setApiKeys] = useState<APIKey[]>([
    { name: 'TWITCH_CLIENT_ID', key: '', placeholder: 'Enter Twitch Client ID', visible: false },
    { name: 'TWITCH_CLIENT_SECRET', key: '', placeholder: 'Enter Twitch Client Secret', visible: false },
    { name: 'TWITCH_BROADCASTER_ID', key: '', placeholder: 'Enter Twitch Broadcaster ID', visible: false },
    { name: 'YOUTUBE_CHANNEL_ID', key: '', placeholder: 'Enter YouTube Channel ID', visible: false },
    { name: 'YOUTUBE_API_KEY', key: '', placeholder: 'Enter YouTube API Key', visible: false },
  ]);

  const [isSaving, setIsSaving] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const [saveMessage, setSaveMessage] = useState('');

  useEffect(() => {
    loadApiKeys();
  }, []);

  const loadApiKeys = async () => {
    setIsLoading(true);
    try {
      const result = await invoke<Record<string, string>>('read_api_keys');
      
      const updatedKeys = [...apiKeys];
      updatedKeys[0].key = result.TWITCH_CLIENT_ID || '';
      updatedKeys[1].key = result.TWITCH_CLIENT_SECRET || '';
      updatedKeys[2].key = result.TWITCH_BROADCASTER_ID || '';
      updatedKeys[3].key = result.YOUTUBE_CHANNEL_ID || '';
      updatedKeys[4].key = result.YOUTUBE_API_KEY || '';
      
      setApiKeys(updatedKeys);
    } catch (error) {
      console.error('Failed to load API keys:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const toggleVisibility = (index: number) => {
    const newApiKeys = [...apiKeys];
    newApiKeys[index].visible = !newApiKeys[index].visible;
    setApiKeys(newApiKeys);
  };

  const handleKeyChange = (index: number, value: string) => {
    const newApiKeys = [...apiKeys];
    newApiKeys[index].key = value;
    setApiKeys(newApiKeys);
  };

  const saveApiKeys = async () => {
    setIsSaving(true);
    setSaveMessage('');
    
    try {
      await invoke('save_api_keys', {
        twitchClientId: apiKeys[0].key,
        twitchClientSecret: apiKeys[1].key,
        twitchBroadcasterId: apiKeys[2].key,
        youtubeChannelId: apiKeys[3].key,
        youtubeApiKey: apiKeys[4].key,
      });
      
      setSaveMessage('API keys saved successfully!');
      setTimeout(() => setSaveMessage(''), 3000);
    } catch (error) {
      setSaveMessage(`Error saving API keys: ${error}`);
    } finally {
      setIsSaving(false);
    }
  };

  if (isLoading) {
    return (
      <div className="api-keys-window loading">
        <p>Loading API keys...</p>
      </div>
    );
  }

  return (
    <div className="api-keys-window">
      <h2>API Keys Configuration</h2>
      
      <div className="api-keys-container">
        {apiKeys.map((apiKey, index) => (
          <div className="api-key-input" key={apiKey.name}>
            <label>{apiKey.name}</label>
            <div className="input-with-icon">
              <input
                type={apiKey.visible ? 'text' : 'password'}
                value={apiKey.key}
                placeholder={apiKey.placeholder}
                onChange={(e) => handleKeyChange(index, e.target.value)}
              />
              <button 
                className="toggle-visibility" 
                onClick={() => toggleVisibility(index)}
                aria-label={apiKey.visible ? 'Hide' : 'Show'}
              >
                {apiKey.visible ? '👁️' : '👁️‍🗨️'}
              </button>
            </div>
          </div>
        ))}
      </div>
      
      <div className="api-keys-actions">
        <button 
          className="save-button" 
          onClick={saveApiKeys} 
          disabled={isSaving}
        >
          {isSaving ? 'Saving...' : 'Save API Keys'}
        </button>
        
        {saveMessage && (
          <div className={`save-message ${saveMessage.includes('Error') ? 'error' : 'success'}`}>
            {saveMessage}
          </div>
        )}
      </div>
      
      <div className="api-keys-help">
        <p>These API keys are used to connect to Twitch and YouTube APIs.</p>
        <p>All keys are stored locally and securely on your device.</p>
      </div>
    </div>
  );
};

export default APIKeysWindow; 