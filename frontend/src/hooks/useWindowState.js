import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export function useWindowState() {
  const [expanded, setExpanded] = useState(false);

  const toggle = useCallback(async () => {
    try {
      const isExpanded = await invoke("toggle_window_width");
      setExpanded(isExpanded);
    } catch (e) {
      console.error("toggle window failed:", e);
    }
  }, []);

  return { expanded, toggle };
}

export function useSaveStatus() {
  const [status, setStatus] = useState(null);
  const [loading, setLoading] = useState(true);

  const refresh = useCallback(async () => {
    setLoading(true);
    try {
      const json = await invoke("get_save_status");
      setStatus(JSON.parse(json));
    } catch (e) {
      console.error("get save status failed:", e);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  return { status, loading, refresh };
}

export function useUsage() {
  const [brief, setBrief] = useState("");

  const refresh = useCallback(async () => {
    try {
      const b = await invoke("get_usage_brief");
      setBrief(b);
    } catch (e) {
      console.error("get usage failed:", e);
    }
  }, []);

  // 实时监听每步 LLM 调用的用量事件
  useEffect(() => {
    let unlisten;
    listen("agent-usage", (e) => setBrief(e.payload))
      .then((fn) => (unlisten = fn))
      .catch(() => {});
    return () => {
      if (unlisten) unlisten();
    };
  }, []);

  return { brief, refresh };
}
