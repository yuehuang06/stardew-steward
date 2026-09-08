import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

export function SettingsPanel({ config, usage, saving, onUpdate, onClose }) {
  const [form, setForm] = useState(null);
  const [testing, setTesting] = useState(false);
  const [testResult, setTestResult] = useState(null);

  const handleTest = async () => {
    if (!form) return;
    setTesting(true);
    setTestResult(null);
    try {
      const r = await invoke("test_api", {
        endpoint: form.endpoint,
        apiKey: form.api_key,
        model: form.model,
      });
      setTestResult({ ok: true, msg: r });
    } catch (e) {
      setTestResult({ ok: false, msg: String(e) });
    } finally {
      setTesting(false);
    }
  };

  useEffect(() => {
    if (config) {
      const m = config.model;
      setForm({
        endpoint: m.endpoint || "",
        api_key: "",
        model: m.model || "",
        context_length: m.context_length || 8192,
        thinking_mode: m.thinking_mode || false,
        price_input: m.price_input || 0,
        price_output: m.price_output || 0,
        token_budget: (config.agent && config.agent.token_budget) || 200000,
      });
    }
  }, [config]);

  if (!form) return null;

  const set = (k, v) => setForm((f) => ({ ...f, [k]: v }));

  const handleSave = () => {
    onUpdate({
      endpoint: form.endpoint,
      apiKey: form.api_key,
      model: form.model,
      contextLength: Number(form.context_length),
      thinkingMode: form.thinking_mode,
      priceInput: Number(form.price_input),
      priceOutput: Number(form.price_output),
      tokenBudget: Number(form.token_budget),
    });
  };

  const labelStyle = { display: "block", marginBottom: "2px", color: "var(--sd-text-light)" };
  const inputStyle = { width: "100%", marginBottom: "8px" };
  const rowStyle = { display: "flex", gap: "6px", marginBottom: "8px" };
  const halfStyle = { flex: 1 };

  return (
    <div
      style={{
        position: "absolute",
        top: 0,
        right: 0,
        bottom: 0,
        width: "100%",
        background: "var(--sd-parchment)",
        borderLeft: "3px solid var(--sd-frame-outer)",
        display: "flex",
        flexDirection: "column",
        zIndex: 10,
      }}
      className="sd-slide-in"
    >
      <div
        data-tauri-drag-region
        style={{
          display: "flex",
          alignItems: "center",
          gap: "6px",
          padding: "4px 8px",
          background: "var(--sd-frame-outer)",
          color: "var(--sd-cream)",
          flexShrink: 0,
        }}
      >
        <span style={{ flex: 1 }}>设置</span>
        <button
          className="sd-btn"
          style={{ padding: "2px 6px" }}
          onClick={onClose}
        >
          X
        </button>
      </div>

      <div style={{ flex: 1, overflowY: "auto", padding: "8px" }}>
        {/* API Settings */}
        <div style={{ marginBottom: "12px" }}>
          <div style={{ fontWeight: 700, marginBottom: "6px" }}>API 配置</div>

          <label style={labelStyle}>Endpoint</label>
          <input
            className="sd-input"
            style={inputStyle}
            value={form.endpoint}
            onChange={(e) => set("endpoint", e.target.value)}
          />

          <label style={labelStyle}>API Key（留空=不修改）</label>
          <input
            className="sd-input"
            style={inputStyle}
            type="password"
            placeholder={config?.model?.api_key || "未设置"}
            value={form.api_key}
            onChange={(e) => set("api_key", e.target.value)}
          />

          <label style={labelStyle}>模型名称</label>
          <input
            className="sd-input"
            style={inputStyle}
            value={form.model}
            onChange={(e) => set("model", e.target.value)}
          />

          <div style={rowStyle}>
            <div style={halfStyle}>
              <label style={labelStyle}>上下文长度</label>
              <input
                className="sd-input"
                style={inputStyle}
                type="number"
                value={form.context_length}
                onChange={(e) => set("context_length", e.target.value)}
              />
            </div>
            <div style={halfStyle}>
              <label style={labelStyle}>思考模式</label>
              <button
                className="sd-btn"
                style={{
                  width: "100%",
                  background: form.thinking_mode ? "var(--sd-green)" : "var(--sd-wood)",
                }}
                onClick={() => set("thinking_mode", !form.thinking_mode)}
              >
                {form.thinking_mode ? "开启" : "关闭"}
              </button>
            </div>
          </div>

          <div style={rowStyle}>
            <div style={halfStyle}>
              <label style={labelStyle}>输入价格 (¥/千token)</label>
              <input
                className="sd-input"
                style={inputStyle}
                type="number"
                step="0.0001"
                value={form.price_input}
                onChange={(e) => set("price_input", e.target.value)}
              />
            </div>
            <div style={halfStyle}>
              <label style={labelStyle}>输出价格 (¥/千token)</label>
              <input
                className="sd-input"
                style={inputStyle}
                type="number"
                step="0.0001"
                value={form.price_output}
                onChange={(e) => set("price_output", e.target.value)}
              />
            </div>
          </div>

          <label style={labelStyle}>Token 预算（每会话）</label>
          <input
            className="sd-input"
            style={inputStyle}
            type="number"
            step="10000"
            value={form.token_budget}
            onChange={(e) => set("token_budget", e.target.value)}
          />

          <button
            className="sd-btn"
            style={{ width: "100%", marginBottom: "4px" }}
            onClick={handleTest}
            disabled={testing}
          >
            {testing ? "测试中..." : "测试 API 连接"}
          </button>
          {testResult && (
            <div
              style={{
                fontSize: "11px",
                padding: "3px 6px",
                marginBottom: "8px",
                wordBreak: "break-word",
                border: "1px solid var(--sd-wood-darker)",
                background: testResult.ok ? "var(--sd-green)" : "var(--sd-red)",
                color: "var(--sd-cream)",
              }}
            >
              {testResult.msg}
            </div>
          )}

          <button
            className="sd-btn sd-btn-green"
            style={{ width: "100%" }}
            onClick={handleSave}
            disabled={saving}
          >
            {saving ? "保存中..." : "保存配置"}
          </button>
        </div>

        {/* Usage Stats */}
        {usage && (
          <div style={{ marginBottom: "12px" }}>
            <div style={{ fontWeight: 700, marginBottom: "6px" }}>用量统计</div>
            <div style={{ background: "var(--sd-cream)", padding: "8px", border: "1px solid var(--sd-wood-darker)" }}>
              <div style={{ display: "flex", justifyContent: "space-between", marginBottom: "4px" }}>
                <span>输入 token</span>
                <span>{usage.input_tokens.toLocaleString()}</span>
              </div>
              <div style={{ display: "flex", justifyContent: "space-between", marginBottom: "4px" }}>
                <span>输出 token</span>
                <span>{usage.output_tokens.toLocaleString()}</span>
              </div>
              <div style={{ display: "flex", justifyContent: "space-between", marginBottom: "4px" }}>
                <span>预算</span>
                <span>{usage.budget.toLocaleString()}</span>
              </div>
              <div style={{ display: "flex", justifyContent: "space-between", fontWeight: 700 }}>
                <span>总花费</span>
                <span>¥{usage.cost.toFixed(4)}</span>
              </div>
            </div>
          </div>
        )}

        {/* Session Info */}
        <div>
          <div style={{ fontWeight: 700, marginBottom: "6px" }}>会话存储</div>
          <div style={{ background: "var(--sd-cream)", padding: "8px", border: "1px solid var(--sd-wood-darker)" }}>
            <div style={{ color: "var(--sd-text-light)", marginBottom: "4px" }}>
              会话自动保存，每次对话后生成
            </div>
            <div style={{ color: "var(--sd-text-light)" }}>
              在"历史"中查看和加载
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
