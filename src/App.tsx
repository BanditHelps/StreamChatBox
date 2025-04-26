import { useState, useEffect } from "react";
import { v4 as uuidv4 } from "uuid";
import ChatBox, { Message } from "./components/ChatBox";
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';
import "./App.css";
import ActivityFeed from "./components/ActivityFeed";
import { Activity } from "./components/ActivityItem";
import DockableLayout from "./components/DockableLayout";
import SettingsPanel from "./components/SettingsPanel";

type DockPosition = 'left' | 'right' | 'top' | 'bottom' | 'none';

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
  const [activities, setActivities] = useState<Activity[]>([]);
  const [autoScroll, setAutoScroll] = useState(true);
  const [showActivityFeed, setShowActivityFeed] = useState(false);
  const [dockPosition, setDockPosition] = useState<DockPosition>('right');
  const [dockSize, setDockSize] = useState(30);

  // Load initial messages and start listeners
  useEffect(() => {
    // Start twitch listener and mock events
    invoke("start_twitch_listener");
    invoke("start_mock_events");

    // Listen for chat messages
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

    // Listen for follow events
    const unlistenFollow = listen("twitch-follow", (event) => {
      const { user } = event.payload as any;

      const newActivity: Activity = {
        id: uuidv4(),
        type: "follow",
        username: user,
        source: "twitch",
        timestamp: new Date()
      };

      // Add to activities
      setActivities(prev => [...prev, newActivity]);
    });

    // Listen for donation events
    const unlistenDonation = listen("twitch-donation", (event) => {
      const { username, amount, message } = event.payload as any;

      const newActivity: Activity = {
        id: uuidv4(),
        type: "donation",
        username: username,
        source: "twitch",
        amount: amount,
        message: message,
        timestamp: new Date()
      };

      // Add to activities
      setActivities(prev => [...prev, newActivity]);
    });

    // Listen for subscription events
    const unlistenSubscription = listen("twitch-subscription", (event) => {
      const { username, tier, is_gift } = event.payload as any;

      const newActivity: Activity = {
        id: uuidv4(),
        type: "subscription",
        username: username,
        source: "twitch",
        message: `Tier ${tier} ${is_gift ? '(gifted)' : ''}`,
        timestamp: new Date()
      };

      // Add to activities
      setActivities(prev => [...prev, newActivity]);
    });

    return () => {
      unlistenChat.then(unlisten => unlisten());
      unlistenFollow.then(unlisten => unlisten());
      unlistenDonation.then(unlisten => unlisten());
      unlistenSubscription.then(unlisten => unlisten());
    };
  }, []);

  const chatBox = (
    <div className="chat-wrapper">
      <div className="chat-header">
        <h1>Stream Chat Box</h1>
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
      <ChatBox messages={messages} autoScroll={autoScroll} />
    </div>
  );

  const activityFeed = (
    <div className="activity-wrapper">
      <div className="activity-header">
        <h2>Activity Feed</h2>
      </div>
      <ActivityFeed activities={activities} autoScroll={autoScroll} />
    </div>
  );

  return (
    <main className="container">
      {showActivityFeed ? (
        <DockableLayout
          mainContent={chatBox}
          dockContent={activityFeed}
          dockPosition={dockPosition}
          showDock={showActivityFeed}
          dockSize={dockSize}
        />
      ) : (
        chatBox
      )}
    </main>
  );
}

export default App;
