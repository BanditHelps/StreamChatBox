import React, { useEffect, useRef } from 'react';
import MessageContent from './MessageContent';
import './ChatBox.css';

export interface Message {
  id: string;
  author: string;
  source: 'youtube' | 'twitch';
  content: string;
  timestamp: Date;
}

interface ChatBoxProps {
  messages: Message[];
  autoScroll?: boolean;
}

const ChatBox: React.FC<ChatBoxProps> = ({ 
  messages, 
  autoScroll = true 
}) => {
  const messagesEndRef = useRef<HTMLDivElement>(null);

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
              <span className="message-source">{message.source}</span>
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