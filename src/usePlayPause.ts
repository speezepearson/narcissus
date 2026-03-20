import { useCallback, useEffect, useRef, useState } from "react";

const MAX_RENDER_FPS = 30;

export function usePlayPause({
  onSteps,
}: {
  onSteps: (count: number) => boolean; // return true if still running
}) {
  const [playing, setPlaying] = useState(false);
  const [fps, setFps] = useState(5);

  const fpsRef = useRef(fps);
  useEffect(() => {
    fpsRef.current = fps;
  }, [fps]);

  const onStepsRef = useRef(onSteps);
  useEffect(() => {
    onStepsRef.current = onSteps;
  }, [onSteps]);

  const accumRef = useRef(0);

  useEffect(() => {
    if (!playing) {
      accumRef.current = 0;
      return;
    }
    const interval = setInterval(() => {
      accumRef.current += fpsRef.current / MAX_RENDER_FPS;
      const stepsThisFrame = Math.floor(accumRef.current);
      accumRef.current -= stepsThisFrame;
      if (stepsThisFrame === 0) return;
      const stillRunning = onStepsRef.current(stepsThisFrame);
      if (!stillRunning) {
        setPlaying(false);
      }
    }, 1000 / MAX_RENDER_FPS);
    return () => clearInterval(interval);
  }, [playing]);

  const toggle = useCallback(() => setPlaying((p) => !p), []);

  return { playing, setPlaying, toggle, fps, setFps };
}
