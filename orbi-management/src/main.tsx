import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { BrowserRouter } from "react-router-dom";

import App from "./App";
import { initializeAuthentication } from "./auth/authBootstrap";

import "./index.css";

const rootElement = document.getElementById("root");

if (!rootElement) {
  throw new Error("ORBI Management root element was not found.");
}

const root = createRoot(rootElement);

function renderApplication() {
  root.render(
    <StrictMode>
      <BrowserRouter>
        <App />
      </BrowserRouter>
    </StrictMode>,
  );
}

function renderAuthenticationError() {
  root.render(
    <StrictMode>
      <main style={{ padding: "2rem" }} role="alert">
        <h1>Unable to start ORBI Management</h1>
        <p>
          We could not verify your identity. Please refresh the page or try
          again later.
        </p>
      </main>
    </StrictMode>,
  );
}

async function startOrbiManagement() {
  const authenticated = await initializeAuthentication();

  if (!authenticated) {
    throw new Error("Keycloak authentication was not completed.");
  }

  renderApplication();
}

void startOrbiManagement().catch((error: unknown) => {
  console.error("Failed to initialize ORBI Management.", error);
  renderAuthenticationError();
});
