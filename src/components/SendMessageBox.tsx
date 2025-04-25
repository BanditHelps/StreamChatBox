import { useState, useRef, useEffect } from 'react';
import { Send } from 'lucide-react';
import './SendMessageBox.css';

export default function SendBox() {
  const [value, setValue] = useState('');
  const textareaRef = useRef(null);
  const containerRef = useRef(null);
  
  // Function to adjust the height and scroll position
  const adjustHeight = () => {
    const textarea = textareaRef.current;
    const container = containerRef.current;
    
    if (textarea && container) {
      // Get the current height
      const currentHeight = textarea.clientHeight;
      
      // Reset height to calculate the actual scrollHeight
      textarea.style.height = 'auto';
      
      // Calculate new height
      const newHeight = textarea.scrollHeight;
      
      // Apply the new height
      textarea.style.height = `${newHeight}px`;
      
      // Adjust the scroll position of the container to expand upward
      if (newHeight > currentHeight) {
        container.scrollTop += (newHeight - currentHeight);
      }
    }
  };

  // Adjust height when value changes
  useEffect(() => {
    adjustHeight();
  }, [value]);

  // relative w-full max-w-md mx-auto h-40 send-container

  return (
    <div className="textarea-wrapper">
      {/* Container with fixed height and overflow that will scroll up */}
      <div 
        ref={containerRef}
        className="textarea-container"
      >
        <textarea
          ref={textareaRef}
          className="annoying-input-box"
          placeholder="Chat Message here"
          value={value}
          onChange={(e) => setValue(e.target.value)}
          rows={1}
        />
        <button className="send-button"><Send/></button>
        
      </div>
    </div>
  );
}