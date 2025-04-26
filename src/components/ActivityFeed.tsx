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
  const containerRef = useRef<HTMLDivElement>(null);
  const [userHasScrolled, setUserHasScrolled] = useState(false);
  const prevActivitiesLength = useRef(activities.length);

  const handleScroll = () => {
    if (!autoScroll || !containerRef.current) return;
    
    const { scrollTop } = containerRef.current;
    // User has scrolled away from the top
    if (scrollTop > 0 && !userHasScrolled) {
      setUserHasScrolled(true);
      if (setAutoScroll) {
        setAutoScroll(false);
      }
    }
  };

  useEffect(() => {
    const container = containerRef.current;
    if (container) {
      container.addEventListener('scroll', handleScroll);
      return () => container.removeEventListener('scroll', handleScroll);
    }
  }, [autoScroll, userHasScrolled]);

  // Handle new activities and auto-scrolling
  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    // If new activities were added and auto-scroll is enabled
    if (activities.length > prevActivitiesLength.current && autoScroll) {
      // Save original scroll position
      const originalScrollTop = container.scrollTop;
      
      // Calculate new scroll position - keep the view in the same relative position
      // by scrolling down by the height of the new content
      requestAnimationFrame(() => {
        container.scrollTop = 0;
      });
    }

    prevActivitiesLength.current = activities.length;
  }, [activities, autoScroll]);

  return (
    <div className="activity-container">
      <div 
        className="activities-container" 
        ref={containerRef}
      >
        {activities.slice().reverse().map((activity) => (
          <ActivityItem key={activity.id} activity={activity} />
        ))}
      </div>
    </div>
  );
};

export default ActivityFeed; 