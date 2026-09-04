import { useEffect, useRef, useCallback, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export function useChat() {
  const [messages, setMessages] = useState([]);
  const [loading, setLoading] = useState(false);
  const [progress, setProgress] = useState([]);
  const [error, setError] = useState(null);
  const unlistenRef = useRef([]);

  useEffect(() => {
    const setup = async () => {
      const handlers = [
        await listen("agent-step", (e) => {
          setProgress((p) => [...p, { type: "step", text: e.payload }]);
        }),
        await listen("agent-thinking", (e) => {
          setProgress((p) => [...p, { type: "thinking", text: e.payload }]);
        }),
        await listen("agent-error", (e) => {
          setProgress((p) => [...p, { type: "error", text: e.payload }]);
          setError(e.payload);
        }),
        await listen("agent-done", () => {
          setProgress([]);
        }),
      ];
      unlistenRef.current = handlers.map((h) => h);
    };
    setup();
    return () => {
      unlistenRef.current.forEach((fn) => fn());
    };
  }, []);

  const send = useCallback(
    async (text) => {
      if (!text.trim() || loading) return;
      setError(null);
      setProgress([]);
      setMessages((m) => [...m, { role: "user", text }]);
      setLoading(true);
      try {
        const reply = await invoke("chat", { message: text });
        setMessages((m) => [...m, { role: "assistant", text: reply }]);
      } catch (e) {
        setError(String(e));
        setMessages((m) => [
          ...m,
          { role: "assistant", text: `出错: ${e}` },
        ]);
      } finally {
        setLoading(false);
        setProgress([]);
      }
    },
    [loading]
  );

  const interrupt = useCallback(async () => {
    await invoke("interrupt");
  }, []);

  return { messages, loading, progress, error, send, interrupt };
}
