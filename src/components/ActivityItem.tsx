import React from 'react';
import twitchIcon from '../assets/twitch32.png';
import './ActivityItem.css';

export interface Activity {
  id: string;
  type: 'follow' | 'donation' | 'subscription';
  username: string;
  source: 'twitch' | 'youtube';
  amount?: number;
  message?: string;
  timestamp: Date;
}

interface ActivityItemProps {
  activity: Activity;
}

const ActivityItem: React.FC<ActivityItemProps> = ({ activity }) => {
  const getActivityIcon = () => {
    return twitchIcon; // Currently only supporting Twitch
  };

  const getActivityMessage = () => {
    switch (activity.type) {
      case 'follow':
        return `${activity.username} followed!`;
      case 'donation':
        return `${activity.username} donated $${activity.amount?.toFixed(2)}!`;
      case 'subscription':
        return `${activity.username} subscribed!`;
      default:
        return '';
    }
  };

  return (
    <div className={`activity-item ${activity.type}`}>
      <div className="activity-header">
        <img
          src={getActivityIcon()}
          alt={activity.source}
          className="source-icon"
        />
        <span className="activity-time">
          {activity.timestamp.toLocaleTimeString()}
        </span>
      </div>
      <div className="activity-content">
        <div className="activity-message">{getActivityMessage()}</div>
        {activity.message && (
          <div className="activity-user-message">{activity.message}</div>
        )}
      </div>
    </div>
  );
};

export default ActivityItem; 