import type { InvestigationCluster } from "./investigationClusters";

export function explainInvestigationCluster(
  cluster: InvestigationCluster,
): string {
  const fuelEvents = cluster.items.filter((item) => item.type === "fuel_event");

  const theftEvents = fuelEvents.filter((item) => item.title.includes("THEFT"));

  const refillEvents = fuelEvents.filter((item) =>
    item.title.includes("REFILL"),
  );

  const sensorHealthEvents = cluster.items.filter(
    (item) => item.type === "sensor_health",
  );

  const stateEvents = cluster.items.filter(
    (item) => item.type === "device_state",
  );

  if (theftEvents.length > 0 && refillEvents.length > 0) {
    return (
      "Potential conflicting fuel activity detected. " +
      "Fuel refill and theft patterns occurred within " +
      "the same operational window."
    );
  }

  if (theftEvents.length > 0 && sensorHealthEvents.length > 0) {
    return (
      "Fuel theft behavior was detected alongside " +
      "sensor integrity anomalies during this " +
      "operational period."
    );
  }

  if (theftEvents.length > 0 && stateEvents.length > 0) {
    return (
      "Fuel theft behavior appears operationally " +
      "consistent with surrounding movement and " +
      "device state telemetry."
    );
  }

  if (refillEvents.length > 0 && stateEvents.length > 0) {
    return (
      "Fuel refill activity was correlated with " +
      "device movement and operational telemetry."
    );
  }

  if (sensorHealthEvents.length > 0) {
    return (
      "Sensor health anomalies were detected " +
      "during this investigation period."
    );
  }

  return (
    "Multiple operational telemetry events were " +
    "grouped into a correlated investigation cluster."
  );
}
