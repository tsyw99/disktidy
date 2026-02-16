import { useEffect, useRef } from "react";
import "./Aurora.css";

// 简化版Aurora组件，使用CSS动画实现类似效果
interface AuroraProps {
  colorStops?: string[];
  amplitude?: number;
  blend?: number;
  speed?: number;
}

export default function Aurora({
  colorStops = ["#6366f1", "#8b5cf6", "#a855f7"],
  amplitude = 1.0,
  blend = 0.5,
  speed = 1.0,
}: AuroraProps) {
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (!containerRef.current) return;
    
    const container = containerRef.current;
    container.style.setProperty('--aurora-color-1', colorStops[0] || '#6366f1');
    container.style.setProperty('--aurora-color-2', colorStops[1] || '#8b5cf6');
    container.style.setProperty('--aurora-color-3', colorStops[2] || '#a855f7');
    container.style.setProperty('--aurora-amplitude', String(amplitude));
    container.style.setProperty('--aurora-blend', String(blend));
    container.style.setProperty('--aurora-speed', `${speed * 15}s`);
  }, [colorStops, amplitude, blend, speed]);

  return (
    <div ref={containerRef} className="aurora-container">
      <div className="aurora-layer aurora-layer-1"></div>
      <div className="aurora-layer aurora-layer-2"></div>
      <div className="aurora-layer aurora-layer-3"></div>
    </div>
  );
}
