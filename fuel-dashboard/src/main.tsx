import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { BrowserRouter } from "react-router-dom";

import "./index.css";
import "./styles/investigation.css";
import "leaflet/dist/leaflet.css";
import "./styles/map-intelligence.css";

import App from "./App.tsx";
import { initializeAuthentication } from "./auth/authBootstrap";
import { resolveStartupDestination } from "./auth/resolveStartupDestination";
import { ClientOnboardingPage } from "./pages/ClientOnboardingPage";

const rootElement = document.getElementById("root");

if (!rootElement) {
  throw new Error("ORBI root element was not found.");
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

function renderStartupError() {
  root.render(
    <StrictMode>
      <main style={{ padding: "2rem" }} role="alert">
        <h1>Unable to start ORBI</h1>
        <p>
          We could not verify your account. Please refresh the page or try again
          later.
        </p>
      </main>
    </StrictMode>,
  );
}

function renderAccessDenied() {
  root.render(
    <StrictMode>
      <main style={{ padding: "2rem" }} role="alert">
        <h1>Access Denied</h1>
        <p>Your ORBI account is not authorized to access this application.</p>
      </main>
    </StrictMode>,
  );
}

async function startOrbi() {
  const authenticated = await initializeAuthentication();

  if (!authenticated) {
    throw new Error(
      "Keycloak initialization completed without authentication.",
    );
  }

  await resolveAndRenderDestination();
}

async function resolveAndRenderDestination() {
  const destination = await resolveStartupDestination();

  switch (destination) {
    case "dashboard":
    case "platform":
      renderApplication();
      return;

    case "onboarding":
      root.render(
        <StrictMode>
          <ClientOnboardingPage
            onComplete={() => {
              void resolveAndRenderDestination().catch((error: unknown) => {
                console.error("Failed to verify completed onboarding.", error);

                renderStartupError();
              });
            }}
          />
        </StrictMode>,
      );
      return;

    case "access_denied":
      renderAccessDenied();
      return;
  }
}

void startOrbi().catch((error: unknown) => {
  console.error("Failed to initialize ORBI.", error);
  renderStartupError();
});
