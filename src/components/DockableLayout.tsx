import React from 'react';
import './DockableLayout.css';

type DockPosition = 'left' | 'right' | 'top' | 'bottom' | 'none';

interface DockableLayoutProps {
  mainContent: React.ReactNode;
  dockContent: React.ReactNode;
  dockPosition: DockPosition;
  showDock: boolean;
  dockSize?: number;
}

const DockableLayout: React.FC<DockableLayoutProps> = ({
  mainContent,
  dockContent,
  dockPosition,
  showDock,
  dockSize = 30,
}) => {
  if (dockPosition === 'none' || !showDock) {
    return <div className="dockable-layout full">{mainContent}</div>;
  }

  const isHorizontal = dockPosition === 'left' || dockPosition === 'right';
  const dockStyleVar = isHorizontal ? '--dock-width' : '--dock-height';
  const mainStyleVar = isHorizontal ? '--main-width' : '--main-height';

  const layoutStyle = {
    [dockStyleVar]: `${dockSize}%`,
    [mainStyleVar]: `${100 - dockSize}%`,
  } as React.CSSProperties;

  return (
    <div className={`dockable-layout ${dockPosition}`} style={layoutStyle}>
      {(dockPosition === 'left' || dockPosition === 'top') && (
        <div className="dock-container">{dockContent}</div>
      )}
      <div className="main-container">{mainContent}</div>
      {(dockPosition === 'right' || dockPosition === 'bottom') && (
        <div className="dock-container">{dockContent}</div>
      )}
    </div>
  );
};

export default DockableLayout; 