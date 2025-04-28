import { useState, useEffect } from "react";
import { v4 as uuidv4 } from "uuid";
import ChatBox, { Message } from "./components/ChatBox";
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';
import { appWindow } from '@tauri-apps/api/window';
import { LogicalSize } from '@tauri-apps/api/window';
import "./App.css";
import ActivityFeed from "./components/ActivityFeed";
import { Activity } from "./components/ActivityItem";
import DockableLayout from "./components/DockableLayout";
import Toolbar from "./components/Toolbar";

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
  const [originalWindowSize, setOriginalWindowSize] = useState<{width: number, height: number} | null>(null);

  // Store original window size and adjust window when activity feed visibility changes
  useEffect(() => {
    // Get and store the original window size only once
    const getOriginalSize = async () => {
      if (originalWindowSize === null) {
        const size = await appWindow.outerSize();
        setOriginalWindowSize({
          width: size.width,
          height: size.height
        });
      }
    };
    
    getOriginalSize();
  }, []);

  // Handle window resizing when activity feed is shown/hidden
  useEffect(() => {
    const adjustWindowSize = async () => {
      if (!originalWindowSize) return;
      
      if (showActivityFeed) {
        // Calculate new size based on dock position
        let newWidth = originalWindowSize.width;
        let newHeight = originalWindowSize.height;
        
        if (dockPosition === 'left' || dockPosition === 'right') {
          // For horizontal docks, increase width by dockSize percentage
          // but limit the increase to maintain reasonable proportions
          const calculatedWidth = Math.round(originalWindowSize.width * (100 / (100 - dockSize)));
          const maxAllowedWidth = originalWindowSize.width * 1.6; // Maximum 60% increase
          newWidth = Math.min(calculatedWidth, maxAllowedWidth);
        } else if (dockPosition === 'top' || dockPosition === 'bottom') {
          // For vertical docks, increase height by dockSize percentage
          // but limit the increase to maintain reasonable proportions
          const calculatedHeight = Math.round(originalWindowSize.height * (100 / (100 - dockSize)));
          const maxAllowedHeight = originalWindowSize.height * 1.6; // Maximum 60% increase
          newHeight = Math.min(calculatedHeight, maxAllowedHeight);
        }
        
        await appWindow.setSize(new LogicalSize(newWidth, newHeight));
      } else {
        // Reset to original size
        await appWindow.setSize(new LogicalSize(
          originalWindowSize.width,
          originalWindowSize.height
        ));
      }
    };
    
    if (originalWindowSize) {
      adjustWindowSize();
    }
  }, [showActivityFeed, dockPosition, dockSize, originalWindowSize]);

  // Load initial messages and start listeners
  useEffect(() => {
    // Start twitch listener and mock events
    invoke("start_twitch_listener");
    invoke("start_mock_events");

    // Initialize badges
    const initBadges = async () => {
      try {
        // Use environment variables or hardcoded values for testing
        // In a real app, you'd want to securely store and retrieve these values
        const clientId = import.meta.env.VITE_TWITCH_CLIENT_ID || "";
        const accessToken = import.meta.env.VITE_TWITCH_ACCESS_TOKEN || "";
        const broadcasterId = import.meta.env.VITE_TWITCH_BROADCASTER_ID || "";
        
        if (clientId && accessToken && broadcasterId) {
          console.log("Initializing Twitch badges...");
          await invoke("initialize_twitch_badges", {
            client_id: clientId,
            access_token: accessToken,
            broadcaster_id: broadcasterId
          });
          console.log("Badges initialized successfully");
        } else {
          console.warn("Missing Twitch API credentials for badge initialization");
        }
      } catch (error) {
        console.error("Failed to initialize badges:", error);
      }
    };

    initBadges();

    // Listen for chat messages
    const unlistenChat = listen("twitch-chat-message", (event) => {
      const { user, color, message, badges } = event.payload as any;

      const newMessage: Message = {
        id: uuidv4(),
        author: user,
        source: "twitch",
        content: message,
        timestamp: new Date(),
        color: color,
        badges: badges
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
        <Toolbar
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
      <ChatBox 
        messages={messages} 
        autoScroll={autoScroll} 
        setAutoScroll={setAutoScroll}
      />
    </div>
  );

  const activityFeed = (
    <div className="activity-wrapper">
      <div className="activity-header">
        <h2>Activity Feed</h2>
      </div>
      <ActivityFeed 
        activities={activities} 
        autoScroll={autoScroll}
        setAutoScroll={setAutoScroll}
      />
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
