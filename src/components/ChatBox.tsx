import React, { useEffect, useRef, useState } from 'react';
import youtubeIcon from '../assets/youtube32.png';
import twitchIcon from '../assets/twitch32.png';
import MessageContent from './MessageContent';
import './ChatBox.css';
import SendBox from './SendMessageBox';

// Interface for Twitch badges
export interface Badge {
  id: string;
  version: string;
}

export interface Message {
  id: string;
  author: string;
  source: 'youtube' | 'twitch';
  content: string;
  timestamp: Date;
  color: string;
  badges?: Badge[]; // Optional array of Badge objects
}

// This is where all the settings for the chatbox go
interface ChatBoxProps {
  messages: Message[];
  autoScroll?: boolean;
  setAutoScroll?: (autoScroll: boolean) => void;
}

const ChatBox: React.FC<ChatBoxProps> = ({ 
  messages, 
  autoScroll = true,
  setAutoScroll
}) => {
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const [userHasScrolled, setUserHasScrolled] = useState(false);
  const [isNearBottom, setIsNearBottom] = useState(true);

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
  
  const checkIfNearBottom = () => {
    if (!containerRef.current) return;
    
    const container = containerRef.current;
    const { scrollTop, scrollHeight, clientHeight } = container;
    const distanceFromBottom = scrollHeight - scrollTop - clientHeight;
    
    // If user is within 50px of the bottom, consider them at the bottom
    const nearBottom = distanceFromBottom < 50;
    setIsNearBottom(nearBottom);
    
    if (nearBottom && userHasScrolled && setAutoScroll) {
      setAutoScroll(true);
      setUserHasScrolled(false);
    }
  };

  const handleScroll = () => {
    if (!autoScroll) return;
    
    if (!userHasScrolled) {
      setUserHasScrolled(true);
    }
    
    checkIfNearBottom();
  };

  useEffect(() => {
    const container = containerRef.current;
    if (container) {
      container.addEventListener('scroll', handleScroll);
      return () => container.removeEventListener('scroll', handleScroll);
    }
  }, [autoScroll, userHasScrolled]);

  useEffect(() => {
    if (autoScroll && messagesEndRef.current && isNearBottom) {
      messagesEndRef.current.scrollIntoView({ behavior: 'smooth' });
    }
  }, [messages, autoScroll, isNearBottom]);

  return (
    <div className="chat-container">
      <div 
        className="messages-container"
        ref={containerRef}
      >
        {messages.map((message) => (
          <div key={message.id} className={`message-item ${message.source}`}>
            <div className="message-header">
              <img
                src={getSourceIcon(message.source)}
                alt={`${message.source}`}
                className="source-icon"
              />  
              <span style={{ color: message.color }} className="author-name">{message.author}</span>           
              <span className="message-time">
                {message.timestamp.toLocaleTimeString()}
              </span>
            </div>
            <div className="message-content">
              <MessageContent content={message.content} badges={message.badges} />
            </div>
          </div>
        ))}
        <div ref={messagesEndRef} />
      </div>
      <div><SendBox/></div>
    </div>
  );
};

export default ChatBox; 