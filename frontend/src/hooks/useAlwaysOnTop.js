import { useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

export function useAlwaysOnTop() {
  const [onTop, setOnTop] = useState(true);

  const toggle = useCallback(async () => {
    try {
      const next = await invoke("toggle_always_on_top");
      setOnTop(next);
    } catch (e) {
      console.error("toggle always on top failed:", e);
    }
  }, []);

  return { onTop, toggle };
}
