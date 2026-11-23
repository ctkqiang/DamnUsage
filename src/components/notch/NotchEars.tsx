import React from "react";

interface NotchEarsProps {
  radius?: number;
}

export const NotchEars: React.FC<NotchEarsProps> = ({ radius = 14 }) => {
  return (
    <>
      <svg
        width={radius}
        height={radius}
        viewBox={`0 0 ${radius} ${radius}`}
        className="pointer-events-none absolute top-0"
        style={{ left: -radius + 0.5 }}
      >
        <path
          d={`M ${radius} 0 L 0 0 A ${radius} ${radius} 0 0 1 ${radius} ${radius} Z`}
          fill="#000000"
        />
      </svg>
      <svg
        width={radius}
        height={radius}
        viewBox={`0 0 ${radius} ${radius}`}
        className="pointer-events-none absolute top-0"
        style={{ right: -radius + 0.5 }}
      >
        <path
          d={`M 0 0 L ${radius} 0 A ${radius} ${radius} 0 0 0 0 ${radius} Z`}
          fill="#000000"
        />
      </svg>
    </>
  );
};
