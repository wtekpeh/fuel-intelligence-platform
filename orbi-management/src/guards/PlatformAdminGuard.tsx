import type { ReactNode } from "react";

import { usePlatformIdentity } from "../hooks/usePlatformIdentity";

interface PlatformAdminGuardProps {
  children: ReactNode;
}

export function PlatformAdminGuard({ children }: PlatformAdminGuardProps) {
  const { access, verifyAccess } = usePlatformIdentity();

  if (access.status === "loading") {
    return (
      <main>
        <h1>ORBI Management</h1>
        <p>Verifying administrator access...</p>
      </main>
    );
  }

  if (access.status === "denied") {
    return (
      <main role="alert">
        <h1>Access denied</h1>
        <p>Your account does not have permission to access ORBI Management.</p>
      </main>
    );
  }

  if (access.status === "authentication_error") {
    return (
      <main role="alert">
        <h1>Authentication required</h1>
        <p>Your session could not be verified. Please sign in again.</p>
      </main>
    );
  }

  if (access.status === "connection_error") {
    return (
      <main role="alert">
        <h1>Unable to connect</h1>
        <p>ORBI Management could not verify your administrator permissions.</p>
        <button type="button" onClick={verifyAccess}>
          Retry
        </button>
      </main>
    );
  }

  return <>{children}</>;
}
