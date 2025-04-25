import { useState, useEffect } from "react";
import { v4 as uuidv4 } from "uuid";
import ChatBox, { Message } from "./components/ChatBox";
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';
import "./App.css";

// Sample messages for demonstration
// const sampleMessages: Message[] = [
//   {
//     id: uuidv4(),
//     author: "StreamUser123",
//     source: "youtube",
//     content: "Hello everyone! 👋 How's it going?",
//     timestamp: new Date(Date.now() - 300000)
//   },
//   {
//     id: uuidv4(),
//     author: "TwitchFan42",
//     source: "twitch",
//     content: "Just joined the stream! 🎮",
//     timestamp: new Date(Date.now() - 250000)
//   },
//   {
//     id: uuidv4(),
//     author: "GameLover99",
//     source: "youtube",
//     content: "This is so exciting! Can't wait to see what happens next 😮",
//     timestamp: new Date(Date.now() - 200000)
//   },
//   {
//     id: uuidv4(),
//     author: "StreamerBuddy",
//     source: "twitch",
//     content: "Wow that was an amazing play! 🔥",
//     timestamp: new Date(Date.now() - 150000)
//   },
//   {
//     id: uuidv4(),
//     author: "ViewerPro",
//     source: "youtube",
//     content: "I've been following for 3 years now, love your content!",
//     timestamp: new Date(Date.now() - 100000)
//   }
// ];



function App() {
  const [messages, setMessages] = useState<Message[]>([]);
  const [autoScroll, setAutoScroll] = useState(true);

  // Load initial messages
  useEffect(() => {
    // setMessages(sampleMessages);

    invoke("start_twitch_listener");

    // Listener for the twtich json messages
    const unlistenChat = listen("twitch-chat-message", (event) => {
      const { user, color, message } = event.payload as any;

      const newMessage: Message = {
        id: uuidv4(),
        author: user,
        source: "twitch",
        content: message,
        timestamp: new Date(),
        color: color
      };

      setMessages(prev => [...prev, newMessage]);
    });

    return () => {
      unlistenChat.then(unlisten => unlisten());
    };
    
    // Simulate new messages arriving every few seconds
    // const interval = setInterval(() => {
    //   const sources = ["youtube", "twitch"] as const;
    //   const newMessage: Message = {
    //     id: uuidv4(),
    //     author: `User${Math.floor(Math.random() * 1000)}`,
    //     source: sources[Math.floor(Math.random() * sources.length)],
    //     content: '',
    //     //content: `New message ${Math.floor(Math.random() * 100)} 😎 ${Math.random() > 0.5 ? "🚀" : "💯"}`,
    //     timestamp: new Date()
    //   };

    //   invoke('test').then(message => newMessage.content = String(message))
      
    //   setMessages(prev => [...prev, newMessage]);
    // }, 3000);
    
    // return () => clearInterval(interval);
  }, []);

  return (
    <main className="container">
      <div className="chat-wrapper">
        <div className="chat-header">
          <h1>Stream Chat Box</h1>
          <div className="auto-scroll-toggle">
            <label>
              <input 
                type="checkbox" 
                checked={autoScroll} 
                onChange={() => setAutoScroll(!autoScroll)} 
              />
              Auto-scroll
            </label>
          </div>
        </div>
        <ChatBox messages={messages} autoScroll={autoScroll} />
      </div>
    </main>
  );
}

export default App;
