// frontend/src/components/LoginForm.tsx
import { useState } from "react";

export function LoginForm({ onLoggedIn }: { onLoggedIn: () => void }) {
  const [username, setUsername] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);

    const res = await fetch("/api/auth/login", {
      method: "POST",
      credentials: "include",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ username, password }),
    });

    const body = await res.json();
    if (body.error) {
      setError(body.error.message);
      return;
    }
    onLoggedIn();
  };

  return (
    <form onSubmit={handleSubmit}>
      <div>
        <label>ユーザ名 <input value={username} onChange={(e) => setUsername(e.target.value)} /></label>
      </div>
      <div>
        <label>パスワード <input type="password" value={password} onChange={(e) => setPassword(e.target.value)} /></label>
      </div>
      {error && <p style={{ color: "red" }}>{error}</p>}
      <button type="submit">ログイン</button>
    </form>
  );
}