import {
  type DashboardSection,
  useDashboardSectionStore,
} from "../../store/dashboardSectionStore";

const sections: {
  key: DashboardSection;
  label: string;
  description: string;
}[] = [
  {
    key: "operations",
    label: "Operations",
    description: "Live telemetry and alert handling",
  },
  {
    key: "device-health",
    label: "Device Health",
    description: "Connectivity and device status",
  },
  {
    key: "investigation",
    label: "Investigation",
    description: "Timeline and forensic review",
  },
  {
    key: "map",
    label: "Map Intelligence",
    description: "Spatial events and asset movement",
  },
  {
    key: "analytics",
    label: "Analytics",
    description: "Trends and intelligence summaries",
  },
];

export function DashboardTabs() {
  const activeSection = useDashboardSectionStore(
    (state) => state.activeSection,
  );

  const setActiveSection = useDashboardSectionStore(
    (state) => state.setActiveSection,
  );

  return (
    <nav className="dashboard-tabs">
      {sections.map((section) => (
        <button
          key={section.key}
          type="button"
          className={
            activeSection === section.key
              ? "dashboard-tabs__button dashboard-tabs__button--active"
              : "dashboard-tabs__button"
          }
          onClick={() => setActiveSection(section.key)}
        >
          <strong>{section.label}</strong>
          <span>{section.description}</span>
        </button>
      ))}
    </nav>
  );
}
