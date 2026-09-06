import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";

export function useSettings() {
  const [config, setConfig] = useState(null);
  const [usage, setUsage] = useState(null);
  const [saving, setSaving] = useState(false);

  const refresh = useCallback(async () => {
    try {
      const cfg = await invoke("get_config");
      setConfig(cfg);
      const u = await invoke("get_usage_detail");
      setUsage(u);
    } catch (e) {
      console.error("load settings failed:", e);
    }
  }, []);

  const update = useCallback(async (vals) => {
    setSaving(true);
    try {
      await invoke("update_config", vals);
      await refresh();
    } catch (e) {
      console.error("save config failed:", e);
      alert("保存配置失败: " + e);
    } finally {
      setSaving(false);
    }
  }, [refresh]);

  useEffect(() => {
    refresh();
  }, [refresh]);

  return { config, usage, saving, update, refresh };
}
