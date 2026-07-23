// frontend/src/components/ApiRequestForm.tsx
import { useState } from "react";

const METHODS = ["GET", "POST", "PUT", "PATCH", "DELETE"] as const;

export function ApiRequestForm() {
  const [method, setMethod] = useState<(typeof METHODS)[number]>("GET");
  const [path, setPath] = useState("/api/health");
  const [body, setBody] = useState("");
  const [response, setResponse] = useState("");

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    const options: RequestInit = { method, credentials: "include" };
    if (method !== "GET" && body.trim() !== "") {
      options.headers = { "Content-Type": "application/json" };
      options.body = body;
    }

    try {
      const res = await fetch(path, options);
      const text = await res.text();

      let formattedText = text;
      try {
        const json = JSON.parse(text);
        formattedText = JSON.stringify(json, null, 2);
      } catch {
        // HTMLやテキストの場合
      }

      setResponse(`status: ${res.status}\n\n${formattedText}`);
    } catch (error) {
      setResponse(`Error: ${error instanceof Error ? error.message : String(error)}`);
    }
  };

  return (
    /* フォーム全体を画面中央に配置しつつ、中身は左揃えにするコンテナ */
    <div style={{ width: "100%", maxWidth: "650px", margin: "0 auto", textAlign: "left" }}>
      <form onSubmit={handleSubmit} style={{ display: "flex", flexDirection: "column", gap: "12px" }}>

        {/* メソッドとURL指定行 */}
        <div style={{ display: "flex", gap: "8px" }}>
          <select value={method} onChange={(e) => setMethod(e.target.value as typeof method)}>
            {METHODS.map((m) => (
              <option key={m} value={m}>{m}</option>
            ))}
          </select>
          <input
            value={path}
            onChange={(e) => setPath(e.target.value)}
            placeholder="/api/..."
            style={{ flex: 1 }}
          />
          <button type="submit">送信</button>
        </div>

        {/* リクエストボディ領域 */}
        <div>
          <textarea
            value={body}
            onChange={(e) => setBody(e.target.value)}
            placeholder='{"key": "value"}'
            rows={5}
            style={{ width: "100%", boxSizing: "border-box", fontFamily: "monospace" }}
          />
        </div>
      </form>

      {/* レスポンス表示領域（左揃え・背景色・横スクロール） */}
      {response && (
        <pre
          style={{
            marginTop: "16px",
            padding: "12px",
            backgroundColor: "#1e1e1e",
            color: "#d4d4d4",
            borderRadius: "6px",
            overflowX: "auto",
            textAlign: "left",
            fontFamily: "monospace",
            fontSize: "14px",
            whiteSpace: "pre-wrap",
            wordBreak: "break-word"
          }}
        >
          {response}
        </pre>
      )}
    </div>
  );
}