import type { ReactNode } from "react";

import { ManagementSidebar } from "../components/navigation/ManagementSidebar";

import "./ManagementLayout.css";

interface ManagementLayoutProps {
  children: ReactNode;
}

export function ManagementLayout({ children }: ManagementLayoutProps) {
  return (
    <div className="management-layout">
      <ManagementSidebar />

      <div className="management-main">
        <header className="management-header">
          <h1>ORBI Management</h1>
          <p>Internal Operations Platform</p>
        </header>

        <main className="management-content">{children}</main>
      </div>
    </div>
  );
}
