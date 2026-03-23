import { useCallback, useEffect, useRef, useState } from "react";

const TARGET_FRAME_MS = 1000 / 30;

export function usePlayPause({
  onSteps,
}: {
  onSteps: (count: number, stopAtMs: number) => boolean; // return true if still running
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

  useEffect(() => {
    if (!playing) return;

    let cancelled = false;
    let lastTime = performance.now();
    let accum = 0;

    function tick() {
      if (cancelled) return;
      const now = performance.now();
      const elapsed = now - lastTime;
      lastTime = now;

      accum += (fpsRef.current * elapsed) / 1000;
      const stepsThisFrame = Math.floor(accum);
      accum -= stepsThisFrame;

      if (stepsThisFrame > 0) {
        const deadline = now + TARGET_FRAME_MS;
        const stillRunning = onStepsRef.current(stepsThisFrame, deadline);
        if (!stillRunning) {
          setPlaying(false);
          return;
        }
      }

      const frameTime = performance.now() - now;
      const delay = Math.max(0, TARGET_FRAME_MS - frameTime);
      setTimeout(tick, delay);
    }

    const id = setTimeout(tick, TARGET_FRAME_MS);
    return () => {
      cancelled = true;
      clearTimeout(id);
    };
  }, [playing]);

  const toggle = useCallback(() => setPlaying((p) => !p), []);

  return { playing, setPlaying, toggle, fps, setFps };
}
