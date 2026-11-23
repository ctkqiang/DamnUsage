import React from "react";
import { MotionConfig } from "framer-motion";
import { NotchShell } from "./components/notch/NotchShell";
import { CqrsProvider } from "./cqrs/store";

export const App: React.FC = () => {
  return (
    <MotionConfig reducedMotion="user">
      <CqrsProvider>
        <main className="h-screen w-screen overflow-hidden bg-transparent">
          <NotchShell />
        </main>
      </CqrsProvider>
    </MotionConfig>
  );
};

export default App;
