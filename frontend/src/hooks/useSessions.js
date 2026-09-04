import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

export function useSessions() {
  const [sessions, setSessions] = useState([]);
  const [loading, setLoading] = useState(false);

  const refresh = useCallback(async () => {
    setLoading(true);
    try {
      const list = await invoke("list_sessions");
      setSessions(list);
    } catch (e) {
      console.error("list sessions failed:", e);
    } finally {
      setLoading(false);
    }
  }, []);

  const save = useCallback(async (name) => {
    await invoke("save_session", { name });
    await refresh();
  }, [refresh]);

  const load = useCallback(async (name) => {
    await invoke("load_session", { name });
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  return { sessions, loading, save, load, refresh };
}
