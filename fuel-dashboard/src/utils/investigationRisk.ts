import type { InvestigationCluster } from "./investigationClusters";

export type InvestigationRiskLevel = "low" | "medium" | "high" | "critical";

export function calculateClusterRisk(
  cluster: InvestigationCluster,
): InvestigationRiskLevel {
  const fuelEvents = cluster.items.filter((item) => item.type === "fuel_event");

  const theftEvents = fuelEvents.filter((item) => item.title.includes("THEFT"));

  const refillEvents = fuelEvents.filter((item) =>
    item.title.includes("REFILL"),
  );

  const dangerEvents = cluster.items.filter(
    (item) => item.severity === "danger",
  );

  const warningEvents = cluster.items.filter(
    (item) => item.severity === "warning",
  );

  const sensorHealthEvents = cluster.items.filter(
    (item) => item.type === "sensor_health",
  );

  const hasMultipleFuelPatterns =
    theftEvents.length > 0 && refillEvents.length > 0;

  if (hasMultipleFuelPatterns && dangerEvents.length > 0) {
    return "critical";
  }

  if (theftEvents.length > 0 && sensorHealthEvents.length > 0) {
    return "high";
  }

  if (theftEvents.length > 0 && dangerEvents.length > 0) {
    return "high";
  }

  if (warningEvents.length >= 2) {
    return "medium";
  }

  if (sensorHealthEvents.length > 0) {
    return "medium";
  }

  return "low";
}
