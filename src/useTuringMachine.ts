import { useCallback, useEffect, useRef, useState } from "react";
import {
  copySnapshot,
  getStatus,
  step,
  type TuringMachineSnapshot,
} from "./types";
import { usePlayPause } from "./usePlayPause";

export function useTuringMachine<State extends string, Symbol extends string>(
  init: TuringMachineSnapshot<State, Symbol>,
  opts?: {
    onStateChange?: (
      oldState: State,
      cur: TuringMachineSnapshot<State, Symbol>,
    ) => void;
  },
) {
  const onStateChangeRef = useRef(opts?.onStateChange);
  useEffect(() => {
    onStateChangeRef.current = opts?.onStateChange;
  }, [opts?.onStateChange]);
  const [snapshot, setSnapshot] = useState(() => copySnapshot(init));
  const [status, setStatus] = useState<"accept" | "reject" | "running">(
    "running",
  );

  const snapRef = useRef(snapshot);
  const statusRef = useRef(status);

  useEffect(() => {
    snapRef.current = snapshot;
  }, [snapshot]);
  useEffect(() => {
    statusRef.current = status;
  }, [status]);

  const publish = useCallback((snap: TuringMachineSnapshot<State, Symbol>) => {
    const st = getStatus(snap);
    snapRef.current = snap;
    statusRef.current = st;
    setSnapshot({ ...snap });
    setStatus(st);
  }, []);

  const doStep = useCallback(() => {
    if (statusRef.current !== "running") return;
    const snap = snapRef.current;
    const oldState = snap.state;
    step(snap);
    if (snap.state !== oldState) {
      onStateChangeRef.current?.(oldState, snap);
    }
    publish(snap);
  }, [publish]);

  const reset = useCallback(() => {
    const snap = copySnapshot(init);
    publish(snap);
  }, [init, publish]);

  const onSteps = useCallback(
    (count: number, stopAtMs: number) => {
      if (statusRef.current !== "running") return false;
      const snap = snapRef.current;
      for (let i = 0; i < count; i++) {
        const oldState = snap.state;
        step(snap);
        if (snap.state !== oldState) {
          onStateChangeRef.current?.(oldState, snap);
        }
        if (getStatus(snap) !== "running") break;
        if (i % 1e4 === 0 && performance.now() >= stopAtMs) break;
      }
      publish(snap);
      return getStatus(snap) === "running";
    },
    [publish],
  );

  const playPause = usePlayPause({ onSteps });

  return {
    snapshot,
    status,
    doStep,
    reset,
    playPause,
  };
}
