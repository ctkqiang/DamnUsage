import React from "react";
import { motion } from "framer-motion";

interface RadialGaugeProps {
  percent: number;
  size?: number;
  strokeWidth?: number;
  accentHex?: string;
  secondaryHex?: string;
  showLabel?: boolean;
}

export const RadialGauge: React.FC<RadialGaugeProps> = ({
  percent,
  size = 56,
  strokeWidth = 5,
  accentHex = "#7C5CFF",
  secondaryHex = "#00D4FF",
  showLabel = true,
}) => {
  const clamped = Math.max(0, Math.min(100, percent));
  const radius = (size - strokeWidth) / 2;
  const circumference = 2 * Math.PI * radius;
  const offset = circumference - (clamped / 100) * circumference;
  const gradId = `radial-${size}-${accentHex.replace("#", "")}`;

  return (
    <div
      className="relative inline-flex items-center justify-center"
      style={{ width: size, height: size }}
    >
      <svg width={size} height={size} className="-rotate-90">
        <defs>
          <linearGradient id={gradId} x1="0%" y1="0%" x2="100%" y2="100%">
            <stop offset="0%" stopColor={clamped >= 90 ? "#EF4444" : accentHex} />
            <stop offset="100%" stopColor={clamped >= 90 ? "#FB7185" : secondaryHex} />
          </linearGradient>
        </defs>
        <circle
          cx={size / 2}
          cy={size / 2}
          r={radius}
          fill="none"
          stroke="rgba(255,255,255,0.09)"
          strokeWidth={strokeWidth}
        />
        <motion.circle
          cx={size / 2}
          cy={size / 2}
          r={radius}
          fill="none"
          stroke={`url(#${gradId})`}
          strokeWidth={strokeWidth}
          strokeLinecap="round"
          strokeDasharray={circumference}
          initial={{ strokeDashoffset: circumference }}
          animate={{ strokeDashoffset: offset }}
          transition={{ type: "spring", stiffness: 150, damping: 22 }}
          style={{
            filter: `drop-shadow(0 0 6px ${accentHex}66)`,
          }}
        />
      </svg>
      {showLabel && (
        <span className="absolute font-mono text-[11px] font-semibold tracking-tight text-white">
          {Math.round(clamped)}%
        </span>
      )}
    </div>
  );
};
