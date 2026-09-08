import { useEffect, useRef, useCallback, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export function useChat() {
  const [messages, setMessages] = useState([]);
  const [loading, setLoading] = useState(false);
  const [progress, setProgress] = useState([]);
  const [error, setError] = useState(null);
  const [sessionUsage, setSessionUsage] = useState({ input: 0, output: 0, cost: 0 });
  const [stepCount, setStepCount] = useState(0);
  const [maxSteps, setMaxSteps] = useState(10);
  const [liveBrief, setLiveBrief] = useState("");
  const unlistenRef = useRef([]);
  const typingTimerRef = useRef(null);
  // 异步输入队列: agent 回答时用户继续输入 → 排队，回答完自动发送
  const loadingRef = useRef(false);
  const queueRef = useRef([]);

  useEffect(() => {
    let cancelled = false;
    const setup = async () => {
      const handlers = [
        await listen("agent-step", (e) => {
          if (!cancelled) {
            setProgress((p) => [...p, { type: "step", text: e.payload }].slice(-2));
            setStepCount((c) => c + 1);
          }
        }),
        await listen("agent-thinking", (e) => {
          if (!cancelled) setProgress((p) => [...p, { type: "thinking", text: e.payload }].slice(-2));
        }),
        await listen("agent-error", (e) => {
          if (!cancelled) {
            setProgress((p) => [...p, { type: "error", text: e.payload }].slice(-2));
            setError(e.payload);
          }
        }),
        await listen("agent-usage", (e) => {
          if (!cancelled) setLiveBrief(e.payload);
        }),
        await listen("agent-done", () => {
          // Don't clear — only clear when final reply arrives
        }),
      ];
      if (cancelled) {
        handlers.forEach((h) => h());
      } else {
        unlistenRef.current = handlers;
      }
    };
    setup();
    return () => {
      cancelled = true;
      unlistenRef.current.forEach((fn) => fn());
      unlistenRef.current = [];
    };
  }, []);

  const sendImmediate = useCallback(async (text) => {
    setError(null);
    setProgress([]);
    setStepCount(0);
    if (typingTimerRef.current) {
      clearInterval(typingTimerRef.current);
      typingTimerRef.current = null;
    }
    setMessages((m) => [...m, { role: "user", text }]);
    loadingRef.current = true;
    setLoading(true);
    try {
      const reply = await invoke("chat", { message: text });
      loadingRef.current = false;
      setLoading(false);
      setProgress([]);

      // Refresh session usage after chat
      try {
        const u = await invoke("get_usage_detail");
        setSessionUsage({ input: u.input_tokens, output: u.output_tokens, cost: u.cost });
      } catch (_) {}

      // If reply is a schedule JSON, render directly without typing effect
      const trimmed = reply.trim();
      if (trimmed.startsWith("{") && trimmed.includes('"summary"') && trimmed.includes('"tasks"')) {
        setMessages((m) => [...m, { role: "assistant", text: reply, typing: false }]);
      } else {
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
      }
    } catch (e) {
      setError(String(e));
      setMessages((m) => [
        ...m,
        { role: "assistant", text: `出错: ${e}` },
      ]);
      loadingRef.current = false;
      setLoading(false);
      setProgress([]);
    }

    // 队列里有下一条 → 自动发送
    const next = queueRef.current.shift();
    if (next) {
      sendImmediate(next);
    }
  }, []);

  const send = useCallback(
    (text) => {
      if (!text.trim()) return;
      if (loadingRef.current) {
        // agent 正在回答: 入队等待
        queueRef.current.push(text);
        setMessages((m) => [...m, { role: "user", text, queued: true }]);
        return;
      }
      sendImmediate(text);
    },
    [sendImmediate]
  );

  const interrupt = useCallback(async () => {
    if (typingTimerRef.current) {
      clearInterval(typingTimerRef.current);
      typingTimerRef.current = null;
      setMessages((m) => {
        const copy = [...m];
        const last = copy[copy.length - 1];
        if (last && last.typing) {
          copy[copy.length - 1] = { ...last, typing: false };
        }
        return copy;
      });
    }
    await invoke("interrupt");
  }, []);

  const newSession = useCallback(async () => {
    if (typingTimerRef.current) {
      clearInterval(typingTimerRef.current);
      typingTimerRef.current = null;
    }
    queueRef.current = [];
    setProgress([]);
    setMessages([]);
    setSessionUsage({ input: 0, output: 0, cost: 0 });
    setLiveBrief("");
    await invoke("new_session");
  }, []);

  const loadSession = useCallback(async (name) => {
    setError(null);
    setProgress([]);
    setMessages([]);
    setLoading(true);
    try {
      const result = await invoke("load_session", { name });
      const msgs = await invoke("get_messages");
      setMessages(msgs.map((m) => ({ ...m, typing: false })));
      setSessionUsage({
        input: result.session_input_tokens,
        output: result.session_output_tokens,
        cost: result.session_cost,
      });
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

  return { messages, loading, progress, error, send, interrupt, loadSession, newSession, sessionUsage, stepCount, maxSteps, liveBrief, queuedCount: queueRef.current.length };
}
