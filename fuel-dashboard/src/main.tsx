import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { BrowserRouter } from "react-router-dom";

import "./index.css";
import "./styles/investigation.css";
import "leaflet/dist/leaflet.css";
import "./styles/map-intelligence.css";

import App from "./App.tsx";
import { initializeAuthentication } from "./auth/authBootstrap";

const rootElement = document.getElementById("root");

if (!rootElement) {
  throw new Error("ORBI root element was not found.");
}

const root = createRoot(rootElement);

/*
 * Authentication is established before the operational application mounts.
 *
 * This is important because App starts shared telemetry and geofence
 * orchestration immediately. Those services must not begin protected API
 * requests before the user's Keycloak session is known.
 */
initializeAuthentication()
  .then((authenticated) => {
    if (!authenticated) {
      throw new Error(
        "Keycloak initialization completed without authentication.",
      );
    }

    root.render(
      <StrictMode>
        <BrowserRouter>
          <App />
        </BrowserRouter>
      </StrictMode>,
    );
  })
  .catch((error: unknown) => {
    console.error("Failed to initialize ORBI authentication.", error);

    root.render(
      <StrictMode>
        <main style={{ padding: "2rem" }}>
          <h1>Unable to start ORBI</h1>
          <p>
            Authentication could not be initialized. Please refresh the page or
            try again later.
          </p>
        </main>
      </StrictMode>,
    );
  });
