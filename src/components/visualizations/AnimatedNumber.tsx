import React, { useEffect } from "react";
import { motion, useMotionValue, useSpring, useTransform } from "framer-motion";

interface AnimatedNumberProps {
  value: number;
  prefix?: string;
  suffix?: string;
  decimals?: number;
  className?: string;
}

export const AnimatedNumber: React.FC<AnimatedNumberProps> = ({
  value,
  prefix = "",
  suffix = "",
  decimals = 2,
  className = "",
}) => {
  const motionVal = useMotionValue(value);
  const springVal = useSpring(motionVal, {
    stiffness: 220,
    damping: 26,
    mass: 0.7,
  });
  const display = useTransform(springVal, (latest) => {
    return `${prefix}${latest.toFixed(decimals)}${suffix}`;
  });

  useEffect(() => {
    motionVal.set(value);
  }, [motionVal, value]);

  return <motion.span className={className}>{display}</motion.span>;
};
