import React, { useRef } from "react";
import { AnimatePresence, motion } from "framer-motion";
import { useCqrs } from "../../cqrs/store";
import { CollapsedNotch } from "./CollapsedNotch";
import { ExpandedDashboard } from "./ExpandedDashboard";
import { NotchEars } from "./NotchEars";
import { PeekNotch } from "./PeekNotch";

export const NotchShell: React.FC = () => {
  const { dashboard, preferences, isSyncing, transitionNotch, syncAll } = useCqrs();
  const hoverTimerRef = useRef<number | null>(null);

  if (!dashboard || !preferences) {
    return null;
  }

  const state = dashboard.notch_state;
  const widthOffset = preferences.notch_width_offset_px;

  const dimensions =
    state === "expanded"
      ? { width: 668, borderRadius: 28 }
      : state === "peek"
        ? { width: 452, borderRadius: 24 }
        : { width: Math.max(296, 344 + widthOffset), borderRadius: 18 };

  const clearTimer = () => {
    if (hoverTimerRef.current !== null) {
      window.clearTimeout(hoverTimerRef.current);
      hoverTimerRef.current = null;
    }
  };

  const handleMouseEnter = () => {
    clearTimer();
    if (state === "collapsed" && preferences.hover_to_peek) {
      hoverTimerRef.current = window.setTimeout(() => {
        void transitionNotch("peek");
      }, 140);
    }
  };

  const handleMouseLeave = () => {
    clearTimer();
    if (state === "peek") {
      hoverTimerRef.current = window.setTimeout(() => {
        void transitionNotch("collapsed");
      }, 220);
    }
  };

  return (
    <div className="flex w-full justify-center bg-transparent">
      <motion.div
        layout
        onMouseEnter={handleMouseEnter}
        onMouseLeave={handleMouseLeave}
        animate={{
          width: dimensions.width,
          borderBottomLeftRadius: dimensions.borderRadius,
          borderBottomRightRadius: dimensions.borderRadius,
        }}
        transition={{
          type: "spring",
          stiffness: 390,
          damping: 32,
          mass: 0.85,
        }}
        className="notch-glass relative border-x border-b border-white/[0.09] shadow-notch"
        style={{
          borderTopLeftRadius: 0,
          borderTopRightRadius: 0,
        }}
      >
        <NotchEars radius={state === "collapsed" ? 12 : 16} />

        <motion.div
          aria-hidden
          className="pointer-events-none absolute inset-x-10 top-0 h-[1.5px]"
          animate={{
            opacity: state === "collapsed" ? 0.25 : 0.7,
          }}
          style={{
            background: `linear-gradient(90deg, transparent 0%, ${dashboard.headline.primary_accent_hex} 50%, transparent 100%)`,
          }}
        />

        <AnimatePresence mode="wait" initial={false}>
          {state === "collapsed" && (
            <CollapsedNotch
              key="collapsed"
              dashboard={dashboard}
              preferences={preferences}
              isSyncing={isSyncing}
              onExpand={() => void transitionNotch("expanded")}
            />
          )}
          {state === "peek" && (
            <PeekNotch
              key="peek"
              dashboard={dashboard}
              isSyncing={isSyncing}
              onExpand={() => void transitionNotch("expanded")}
              onSyncAll={() => void syncAll()}
            />
          )}
          {state === "expanded" && (
            <ExpandedDashboard
              key="expanded"
              onCollapse={() => void transitionNotch("collapsed")}
            />
          )}
        </AnimatePresence>
      </motion.div>
    </div>
  );
};
