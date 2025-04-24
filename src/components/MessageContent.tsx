import React from 'react';
import './MessageContent.css';

interface MessageContentProps {
  content: string;
}

// Simple regex to match common emoji patterns
const emojiRegex = /(\p{Emoji_Presentation}|\p{Extended_Pictographic})/gu;

const MessageContent: React.FC<MessageContentProps> = ({ content }) => {
  // Split content into segments of text and emojis
  const parts = content.split(emojiRegex);
  
  return (
    <div className="message-text">
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