import { useCallback, useEffect, useState } from "react";
import axios from "axios";

import { getPlatformIdentity, type PlatformIdentity } from "../api/platformApi";

export type PlatformAccessState =
  | { status: "loading" }
  | { status: "authorized"; identity: PlatformIdentity }
  | { status: "denied" }
  | { status: "authentication_error" }
  | { status: "connection_error" };

async function resolvePlatformIdentity(): Promise<PlatformAccessState> {
  try {
    const identity = await getPlatformIdentity();

    if (identity.platform_role !== "SUPER_ADMIN") {
      return { status: "denied" };
    }

    return {
      status: "authorized",
      identity,
    };
  } catch (error: unknown) {
    if (axios.isAxiosError(error)) {
      if (error.response?.status === 403) {
        return { status: "denied" };
      }

      if (error.response?.status === 401) {
        return { status: "authentication_error" };
      }
    }

    console.error("Failed to verify platform identity.", error);

    return { status: "connection_error" };
  }
}

export function usePlatformIdentity() {
  const [access, setAccess] = useState<PlatformAccessState>({
    status: "loading",
  });

  const [retryCount, setRetryCount] = useState(0);

  const verifyAccess = useCallback(() => {
    setAccess({ status: "loading" });
    setRetryCount((previousCount) => previousCount + 1);
  }, []);

  useEffect(() => {
    let cancelled = false;

    void resolvePlatformIdentity().then((result) => {
      if (!cancelled) {
        setAccess(result);
      }
    });

    return () => {
      cancelled = true;
    };
  }, [retryCount]);

  return {
    access:
      retryCount > 0 && access.status === "connection_error" ? access : access,
    verifyAccess,
  };
}
