import React, { useEffect, useRef, useState } from 'react';
import ActivityItem, { Activity } from './ActivityItem';
import './ActivityFeed.css';

interface ActivityFeedProps {
  activities: Activity[];
  autoScroll?: boolean;
  setAutoScroll?: (autoScroll: boolean) => void;
}

const ActivityFeed: React.FC<ActivityFeedProps> = ({
  activities,
  autoScroll = true,
  setAutoScroll
}) => {
  const activitiesEndRef = useRef<HTMLDivElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const [userHasScrolled, setUserHasScrolled] = useState(false);
  const [isNearBottom, setIsNearBottom] = useState(true);

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
    if (autoScroll && activitiesEndRef.current && isNearBottom) {
      activitiesEndRef.current.scrollIntoView({ behavior: 'smooth' });
    }
  }, [activities, autoScroll, isNearBottom]);

  return (
    <div className="activity-container">
      <div 
        className="activities-container" 
        ref={containerRef}
      >
        {activities.map((activity) => (
          <ActivityItem key={activity.id} activity={activity} />
        ))}
        <div ref={activitiesEndRef} />
      </div>
    </div>
  );
};

export default ActivityFeed; 