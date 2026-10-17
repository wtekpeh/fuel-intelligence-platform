import { Route, Routes } from "react-router-dom";

import { PlatformAdminGuard } from "./guards/PlatformAdminGuard";
import { ManagementLayout } from "./layouts/ManagementLayout";

import { DashboardPage } from "./pages/DashboardPage";
import { ComingSoonPage } from "./pages/ComingSoonPage";

function App() {
  return (
    <PlatformAdminGuard>
      <ManagementLayout>
        <Routes>
          <Route path="/" element={<DashboardPage />} />

          <Route
            path="/inventory"
            element={
              <ComingSoonPage
                title="Device Inventory"
                description="Manage ORBI hardware inventory and device lifecycle."
              />
            }
          />

          <Route
            path="/provisioning"
            element={
              <ComingSoonPage
                title="Device Provisioning"
                description="Prepare and provision ORBI devices for deployment."
              />
            }
          />

          <Route
            path="/organizations"
            element={
              <ComingSoonPage
                title="Organizations"
                description="Manage customer organizations and their resources."
              />
            }
          />

          <Route
            path="/deployments"
            element={
              <ComingSoonPage
                title="Deployments"
                description="Manage deployed ORBI devices and customer installations."
              />
            }
          />

          <Route
            path="/device-models"
            element={
              <ComingSoonPage
                title="Device Models"
                description="Manage ORBI device models and hardware configurations."
              />
            }
          />

          <Route
            path="*"
            element={
              <ComingSoonPage
                title="Page Not Found"
                description="The requested management page does not exist."
              />
            }
          />
        </Routes>
      </ManagementLayout>
    </PlatformAdminGuard>
  );
}

export default App;
