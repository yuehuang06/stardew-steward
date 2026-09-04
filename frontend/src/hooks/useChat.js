import { useEffect, useRef, useCallback, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export function useChat() {
  const [messages, setMessages] = useState([]);
  const [loading, setLoading] = useState(false);
  const [progress, setProgress] = useState([]);
  const [error, setError] = useState(null);
  const unlistenRef = useRef([]);
  const typingTimerRef = useRef(null);

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
          // Don't clear progress — keep steps visible
        }),
      ];
      unlistenRef.current = handlers.map((h) => h);
    };
    setup();
    return () => {
      unlistenRef.current.forEach((fn) => fn());
      if (typingTimerRef.current) clearInterval(typingTimerRef.current);
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
        setLoading(false);

        // Typewriter effect: reveal characters gradually
        const chars = [...reply];
        const totalTicks = Math.min(chars.length, 150);
        const charsPerTick = Math.ceil(chars.length / totalTicks);
        let i = 0;

        setMessages((m) => [...m, { role: "assistant", text: "", typing: true }]);

        typingTimerRef.current = setInterval(() => {
          i += charsPerTick;
          if (i >= chars.length) {
            clearInterval(typingTimerRef.current);
            typingTimerRef.current = null;
            setMessages((m) => {
              const copy = [...m];
              copy[copy.length - 1] = { role: "assistant", text: reply, typing: false };
              return copy;
            });
          } else {
            setMessages((m) => {
              const copy = [...m];
              copy[copy.length - 1] = { role: "assistant", text: chars.slice(0, i).join(""), typing: true };
              return copy;
            });
          }
        }, 18);
      } catch (e) {
        setError(String(e));
        setMessages((m) => [
          ...m,
          { role: "assistant", text: `出错: ${e}` },
        ]);
        setLoading(false);
      }
    },
    [loading]
  );

  const interrupt = useCallback(async () => {
    // If typing, finish immediately
    if (typingTimerRef.current) {
      clearInterval(typingTimerRef.current);
      typingTimerRef.current = null;
      setMessages((m) => {
        const copy = [...m];
        const last = copy[copy.length - 1];
        if (last && last.typing) {
          // Keep partial text, just stop typing
          copy[copy.length - 1] = { ...last, typing: false };
        }
        return copy;
      });
    }
    await invoke("interrupt");
  }, []);

  const loadSession = useCallback(async (name) => {
    setError(null);
    setProgress([]);
    setMessages([]);
    setLoading(true);
    try {
      await invoke("load_session", { name });
      const msgs = await invoke("get_messages");
      setMessages(msgs.map((m) => ({ ...m, typing: false })));
    } catch (e) {
      setError(String(e));
      setMessages((m) => [
        ...m,
        { role: "assistant", text: `加载会话失败: ${e}` },
      ]);
    } finally {
      setLoading(false);
    }
  }, []);

  return { messages, loading, progress, error, send, interrupt, loadSession };
}
