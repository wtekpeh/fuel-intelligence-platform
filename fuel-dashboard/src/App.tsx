import { DashboardPage } from "./pages/DashboardPage";
import { FleetOverviewPage } from "./pages/FleetOverviewPage";
import { LandingPage } from "./pages/LandingPage";
import { useTelemetryPolling } from "./services/useTelemetryPolling";
import { useAppViewStore } from "./store/appViewStore";
import { useGeofenceData } from "./services/useGeofenceData";
import PlatformManagementPage from "./platform/pages/PlatformManagementPage";

import "./styles/global.css";

function App() {
  const activeView = useAppViewStore((state) => state.activeView);

  useTelemetryPolling();
  useGeofenceData();

  if (activeView === "fleet") {
    return <FleetOverviewPage />;
  }

  if (activeView === "dashboard") {
    return <DashboardPage />;
  }

  if (activeView === "platform") {
    return <PlatformManagementPage />;
  }

  return <LandingPage />;
}

export default App;
