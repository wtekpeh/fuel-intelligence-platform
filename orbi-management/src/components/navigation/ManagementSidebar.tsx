import { NavLink } from "react-router-dom";

import "./ManagementSidebar.css";

const navigationItems = [
  { label: "Dashboard", path: "/" },
  { label: "Device Inventory", path: "/inventory" },
  { label: "Provisioning", path: "/provisioning" },
  { label: "Organizations", path: "/organizations" },
  { label: "Deployments", path: "/deployments" },
  { label: "Device Models", path: "/device-models" },
];

export function ManagementSidebar() {
  return (
    <aside className="management-sidebar">
      <div className="sidebar-brand">
        <span className="sidebar-brand-name">ORBI</span>
        <span className="sidebar-brand-subtitle">MANAGEMENT</span>
      </div>

      <nav className="sidebar-navigation" aria-label="Management navigation">
        {navigationItems.map((item) => (
          <NavLink
            key={item.path}
            to={item.path}
            end={item.path === "/"}
            className={({ isActive }) =>
              `sidebar-link${isActive ? " sidebar-link-active" : ""}`
            }
          >
            {item.label}
          </NavLink>
        ))}
      </nav>
    </aside>
  );
}
