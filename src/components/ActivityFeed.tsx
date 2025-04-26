import React, { useEffect, useRef } from 'react';
import ActivityItem, { Activity } from './ActivityItem';
import './ActivityFeed.css';

interface ActivityFeedProps {
  activities: Activity[];
  autoScroll?: boolean;
}

const ActivityFeed: React.FC<ActivityFeedProps> = ({ 
  activities, 
  autoScroll = true 
}) => {
  const activitiesEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (autoScroll && activitiesEndRef.current) {
      activitiesEndRef.current.scrollIntoView({ behavior: 'smooth' });
    }
  }, [activities, autoScroll]);

  return (
    <div className="activity-container">
      <div className="activities-container">
        {activities.map((activity) => (
          <ActivityItem key={activity.id} activity={activity} />
        ))}
        <div ref={activitiesEndRef} />
      </div>
    </div>
  );
};

export default ActivityFeed; 