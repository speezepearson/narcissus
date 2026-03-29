import { useCallback, useEffect, useRef, useState } from "react";
import {
  copySnapshot,
  getStatus,
  State,
  step,
  type TuringMachineSnapshot,
} from "./types";
import { usePlayPause } from "./usePlayPause";

export function useTuringMachine(
  init: TuringMachineSnapshot,
  opts?: {
    onStateChange?: (oldState: State, cur: TuringMachineSnapshot) => void;
    initialFps?: number;
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

  const publish = useCallback((snap: TuringMachineSnapshot) => {
    const st = getStatus(snap);
    while (snapRef.current.tape.length < snapRef.current.pos + 1) {
      snapRef.current.tape.push(snapRef.current.spec.blank);
    }
    snapRef.current = snap;
    statusRef.current = st;
    setSnapshot({ ...snap, tape: snap.tape.slice() });
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
    const oldState = snapRef.current.state;
    const snap = copySnapshot(init);
    if (snap.state !== oldState) {
      onStateChangeRef.current?.(oldState, snap);
    }
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

  const playPause = usePlayPause({ onSteps, initialFps: opts?.initialFps });

  return {
    snapshot,
    status,
    doStep,
    reset,
    playPause,
  };
}
