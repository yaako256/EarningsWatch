// frontend/src/App.tsx
import { useState } from "react";
import { LoginForm } from "./components/LoginForm";
import { ApiRequestForm } from "./components/ApiRequestForm";

export default function App() {
  const [loggedIn, setLoggedIn] = useState(false);

  return (
    <div>
      <h1>EarningsWatch</h1>
      {loggedIn ? <ApiRequestForm /> : <LoginForm onLoggedIn={() => setLoggedIn(true)} />}
    </div>
  );
}