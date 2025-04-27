import React from 'react';
import './MessageContent.css';
import { Badge } from './ChatBox';

interface MessageContentProps {
  content: string;
  badges?: Badge[];
}

// Simple regex to match common emoji patterns
const emojiRegex = /(\p{Emoji_Presentation}|\p{Extended_Pictographic})/gu;

const MessageContent: React.FC<MessageContentProps> = ({ content, badges }) => {
  // Split content into segments of text and emojis
  const parts = content.split(emojiRegex);
  
  return (
    <div className="message-text">
      {badges && badges.length > 0 && (
        <div className="message-badges">
          {badges.map((badge, i) => (
            <span key={i} className="badge" title={badge.id}>
              <img 
                src={`https://static-cdn.jtvnw.net/badges/v1/${badge.id}/${badge.version}/1`} 
                alt={badge.id} 
                className="badge-icon" 
              />
            </span>
          ))}
        </div>
      )}
      {parts.map((part, index) => {
        // Check if this part is an emoji
        if (part.match(emojiRegex)) {
          return (
            <span key={index} className="emoji">
              {part}
            </span>
          );
        }
        return <span key={index}>{part}</span>;
      })}
    </div>
  );
};

export default MessageContent; 