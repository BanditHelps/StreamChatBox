import React, { useEffect, useRef } from 'react';
import youtubeIcon from '../assets/youtube32.png';
import twitchIcon from '../assets/twitch32.png';
import MessageContent from './MessageContent';
import './ChatBox.css';

export interface Message {
  id: string;
  author: string;
  source: 'youtube' | 'twitch';
  content: string;
  timestamp: Date;
}

// This is where all the settings for the chatbox go
interface ChatBoxProps {
  messages: Message[];
  autoScroll?: boolean;
}

const ChatBox: React.FC<ChatBoxProps> = ({ 
  messages, 
  autoScroll = true 
}) => {
  const messagesEndRef = useRef<HTMLDivElement>(null);

  // Get the proper image based on source
  const getSourceIcon = (source: 'youtube' | 'twitch') => {
    switch (source) {
      case 'youtube':
        return youtubeIcon;
      case 'twitch':
        return twitchIcon;
      default:
        return youtubeIcon;
    }
  };

  useEffect(() => {
    if (autoScroll && messagesEndRef.current) {
      messagesEndRef.current.scrollIntoView({ behavior: 'smooth' });
    }
  }, [messages, autoScroll]);

  return (
    <div className="chat-container">
      <div className="messages-container">
        {messages.map((message) => (
          <div key={message.id} className={`message-item ${message.source}`}>
            <div className="message-header">
              <span className="author-name">{message.author}</span>
              <img
                src={getSourceIcon(message.source)}
                alt={`${message.source}`}
                className="source-icon"
              />              
              <span className="message-time">
                {message.timestamp.toLocaleTimeString()}
              </span>
            </div>
            <div className="message-content">
              <MessageContent content={message.content} />
            </div>
          </div>
        ))}
        <div ref={messagesEndRef} />
      </div>
    </div>
  );
};

export default ChatBox; 