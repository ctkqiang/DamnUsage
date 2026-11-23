import React from "react";
import { motion } from "framer-motion";
import type { SparklinePoint } from "../../cqrs/types";

interface SparklineChartProps {
  points: SparklinePoint[];
  accentHex?: string;
  secondaryHex?: string;
  width?: number;
  height?: number;
}

export const SparklineChart: React.FC<SparklineChartProps> = ({
  points,
  accentHex = "#7C5CFF",
  secondaryHex = "#00D4FF",
  width = 160,
  height = 42,
}) => {
  const safePoints =
    points.length >= 2
      ? points
      : [
          { timestamp: "", spend_usd: 1, tokens: 0 },
          { timestamp: "", spend_usd: 1.2, tokens: 0 },
        ];

  const values = safePoints.map((p) => p.spend_usd);
  const minVal = Math.min(...values);
  const maxVal = Math.max(...values);
  const range = Math.max(0.01, maxVal - minVal);
  const padX = 4;
  const padY = 6;
  const usableW = width - padX * 2;
  const usableH = height - padY * 2;

  const coords = safePoints.map((pt, idx) => {
    const x = padX + (idx / (safePoints.length - 1)) * usableW;
    const y = padY + usableH - ((pt.spend_usd - minVal) / range) * usableH;
    return { x, y };
  });

  const linePath = coords
    .map((pt, idx) => {
      if (idx === 0) return `M ${pt.x.toFixed(1)} ${pt.y.toFixed(1)}`;
      const prev = coords[idx - 1];
      const cx = ((prev.x + pt.x) / 2).toFixed(1);
      return `C ${cx} ${prev.y.toFixed(1)}, ${cx} ${pt.y.toFixed(1)}, ${pt.x.toFixed(1)} ${pt.y.toFixed(1)}`;
    })
    .join(" ");

  const last = coords[coords.length - 1];
  const areaPath = `${linePath} L ${last.x.toFixed(1)} ${height} L ${coords[0].x.toFixed(1)} ${height} Z`;
  const strokeId = `spark-stroke-${accentHex.replace("#", "")}`;
  const fillId = `spark-fill-${accentHex.replace("#", "")}`;

  return (
    <svg width={width} height={height} className="overflow-visible">
      <defs>
        <linearGradient id={strokeId} x1="0%" y1="0%" x2="100%" y2="0%">
          <stop offset="0%" stopColor={accentHex} />
          <stop offset="100%" stopColor={secondaryHex} />
        </linearGradient>
        <linearGradient id={fillId} x1="0%" y1="0%" x2="0%" y2="100%">
          <stop offset="0%" stopColor={accentHex} stopOpacity="0.34" />
          <stop offset="100%" stopColor={accentHex} stopOpacity="0" />
        </linearGradient>
      </defs>
      <motion.path
        d={areaPath}
        fill={`url(#${fillId})`}
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ duration: 0.6 }}
      />
      <motion.path
        d={linePath}
        pathLength={1}
        fill="none"
        stroke={`url(#${strokeId})`}
        strokeWidth="2"
        strokeLinecap="round"
        strokeDasharray="1"
        initial={{ strokeDashoffset: 1, opacity: 0.4 }}
        animate={{ strokeDashoffset: 0, opacity: 1 }}
        transition={{ duration: 0.9, ease: "easeOut" }}
      />
      <motion.circle
        cx={last.x}
        cy={last.y}
        r="3"
        fill={secondaryHex}
        animate={{ scale: [1, 1.45, 1], opacity: [0.85, 1, 0.85] }}
        transition={{ duration: 2, repeat: Infinity, ease: "easeInOut" }}
        style={{ filter: `drop-shadow(0 0 6px ${secondaryHex})` }}
      />
    </svg>
  );
};
