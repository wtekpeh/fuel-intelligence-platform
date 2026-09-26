import { Navigate, Route, Routes } from "react-router-dom";

import { DashboardPage } from "./pages/DashboardPage";
import { FleetOverviewPage } from "./pages/FleetOverviewPage";
import { LandingPage } from "./pages/LandingPage";
import PlatformManagementPage from "./platform/pages/PlatformManagementPage";

import { useTelemetryPolling } from "./services/useTelemetryPolling";
import { useGeofenceData } from "./services/useGeofenceData";

import "./styles/global.css";

function App() {
  useTelemetryPolling();
  useGeofenceData();

  return (
    <Routes>
      <Route path="/" element={<LandingPage />} />

      <Route path="/fleet" element={<FleetOverviewPage />} />

      <Route path="/dashboard" element={<DashboardPage />} />

      <Route path="/platform" element={<PlatformManagementPage />} />

      <Route path="*" element={<Navigate to="/" replace />} />
    </Routes>
  );
}

export default App;
