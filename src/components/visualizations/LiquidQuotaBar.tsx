import React from "react";
import { motion } from "framer-motion";

interface LiquidQuotaBarProps {
  percent: number;
  accentHex: string;
  secondaryHex: string;
  heightClass?: string;
  warningThreshold?: number;
  criticalThreshold?: number;
}

export const LiquidQuotaBar: React.FC<LiquidQuotaBarProps> = ({
  percent,
  accentHex,
  secondaryHex,
  heightClass = "h-1.5",
  warningThreshold = 75,
  criticalThreshold = 90,
}) => {
  const clamped = Math.max(2, Math.min(100, percent));
  const gradient =
    percent >= criticalThreshold
      ? "linear-gradient(90deg, #EF4444 0%, #FB7185 100%)"
      : percent >= warningThreshold
        ? "linear-gradient(90deg, #F59E0B 0%, #FBBF24 100%)"
        : `linear-gradient(90deg, ${accentHex} 0%, ${secondaryHex} 100%)`;

  const glow =
    percent >= criticalThreshold
      ? "rgba(239, 68, 68, 0.55)"
      : percent >= warningThreshold
        ? "rgba(245, 158, 11, 0.45)"
        : `${accentHex}66`;

  return (
    <div
      className={`relative w-full overflow-hidden rounded-full bg-white/[0.08] ${heightClass}`}
    >
      <motion.div
        className="relative h-full rounded-full"
        initial={{ width: 0 }}
        animate={{ width: `${clamped}%` }}
        transition={{ type: "spring", stiffness: 180, damping: 24, mass: 0.8 }}
        style={{
          background: gradient,
          boxShadow: `0 0 12px ${glow}`,
        }}
      >
        <div className="shimmer-bar absolute inset-y-0 w-1/2 bg-gradient-to-r from-transparent via-white/40 to-transparent" />
      </motion.div>
    </div>
  );
};
