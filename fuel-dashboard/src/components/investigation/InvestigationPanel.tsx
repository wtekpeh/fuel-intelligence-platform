import { useState } from "react";
import { useFleetStore } from "../../store/fleetStore";
import { useInvestigationStore } from "../../store/investigationStore";
import { buildInvestigationTimeline } from "../../utils/investigationTimeline";
import { InvestigationDetailPanel } from "./InvestigationDetailPanel";
import { buildInvestigationClusters } from "../../utils/investigationClusters";
import { calculateClusterRisk } from "../../utils/investigationRisk";
import { explainInvestigationCluster } from "../../utils/investigationExplanation";
import { useGeofenceStore } from "../../store/geofenceStore";

export function InvestigationPanel() {
  const selectedDevice = useFleetStore((state) => state.selectedDevice);

  const { fuelEvents, deviceStateEvents, sensorHealthEvents } =
    useInvestigationStore();

  const selectTimelineItem = useInvestigationStore(
    (state) => state.selectTimelineItem,
  );

  const selectedTimelineItem = useInvestigationStore(
    (state) => state.selectedTimelineItem,
  );

  const focusedFuelEventId = useInvestigationStore(
    (state) => state.focusedFuelEventId,
  );

  const geofenceTransitionEvents = useGeofenceStore(
    (state) => state.transitionEvents,
  );

  const [activeTimelineFilter, setActiveTimelineFilter] = useState<
    | "all"
    | "fuel_event"
    | "device_state"
    | "sensor_health"
    | "geofence_transition"
  >("all");

  const timelineItems = buildInvestigationTimeline({
    fuelEvents,
    deviceStateEvents,
    sensorHealthEvents,
    geofenceTransitionEvents,
  });

  const visibleTimelineItems =
    activeTimelineFilter === "all"
      ? timelineItems
      : timelineItems.filter((item) => item.type === activeTimelineFilter);

  const investigationClusters =
    buildInvestigationClusters(visibleTimelineItems);

  return (
    <section className="investigation-panel">
      <div className="investigation-panel__header">
        <div>
          <h2>Investigation</h2>

          <p>
            Device-scoped operational reconstruction using fuel events, movement
            states, and sensor health events.
          </p>
        </div>
      </div>

      <div className="investigation-summary">
        <div>
          <label>Device</label>
          <strong>{selectedDevice?.device_code ?? "No device selected"}</strong>
        </div>

        <div>
          <label>Fuel Events</label>
          <strong>{fuelEvents.length}</strong>
        </div>

        <div>
          <label>State Events</label>
          <strong>{deviceStateEvents.length}</strong>
        </div>

        <div>
          <label>Sensor Health</label>
          <strong>{sensorHealthEvents.length}</strong>
        </div>

        <div>
          <label>Geofence Events</label>
          <strong>{geofenceTransitionEvents.length}</strong>
        </div>
      </div>

      <div className="investigation-filter">
        <button
          type="button"
          className={
            activeTimelineFilter === "all"
              ? "investigation-filter__button investigation-filter__button--active"
              : "investigation-filter__button"
          }
          onClick={() => setActiveTimelineFilter("all")}
        >
          All
        </button>

        <button
          type="button"
          className={
            activeTimelineFilter === "fuel_event"
              ? "investigation-filter__button investigation-filter__button--active"
              : "investigation-filter__button"
          }
          onClick={() => setActiveTimelineFilter("fuel_event")}
        >
          Fuel Events
        </button>

        <button
          type="button"
          className={
            activeTimelineFilter === "device_state"
              ? "investigation-filter__button investigation-filter__button--active"
              : "investigation-filter__button"
          }
          onClick={() => setActiveTimelineFilter("device_state")}
        >
          Device State
        </button>

        <button
          type="button"
          className={
            activeTimelineFilter === "sensor_health"
              ? "investigation-filter__button investigation-filter__button--active"
              : "investigation-filter__button"
          }
          onClick={() => setActiveTimelineFilter("sensor_health")}
        >
          Sensor Health
        </button>

        <button
          type="button"
          className={
            activeTimelineFilter === "geofence_transition"
              ? "investigation-filter__button investigation-filter__button--active"
              : "investigation-filter__button"
          }
          onClick={() => setActiveTimelineFilter("geofence_transition")}
        >
          Geofence
        </button>
      </div>

      <div className="investigation-workspace">
        <div className="investigation-workspace__timeline">
          <div className="investigation-timeline">
            {[
              ...investigationClusters.filter((cluster) =>
                cluster.items.some(
                  (item) => item.id === `fuel-${focusedFuelEventId}`,
                ),
              ),

              ...investigationClusters.filter(
                (cluster) =>
                  !cluster.items.some(
                    (item) => item.id === `fuel-${focusedFuelEventId}`,
                  ),
              ),
            ]
              .slice(0, 15)
              .map((cluster) => {
                const clusterRisk = calculateClusterRisk(cluster);

                return (
                  <section
                    key={cluster.id}
                    className={`investigation-cluster investigation-cluster--${clusterRisk}`}
                  >
                    <div className="investigation-cluster__header">
                      <div>
                        <h3>Incident Cluster</h3>

                        <span>
                          {new Date(cluster.startTime).toLocaleString()}
                        </span>
                      </div>

                      <div
                        className={`investigation-cluster__risk investigation-cluster__risk--${clusterRisk}`}
                      >
                        {clusterRisk.toUpperCase()}
                      </div>

                      <p className="investigation-cluster__explanation">
                        {explainInvestigationCluster(cluster)}
                      </p>
                    </div>

                    <div className="investigation-cluster__items">
                      {cluster.items.map((item) => (
                        <article
                          key={item.id}
                          className={`investigation-timeline__item investigation-timeline__item--${item.severity} ${
                            selectedTimelineItem?.id === item.id ||
                            item.id === `fuel-${focusedFuelEventId}`
                              ? "investigation-timeline__item--selected"
                              : ""
                          }`}
                          onClick={() => selectTimelineItem(item)}
                        >
                          <div>
                            <span className="investigation-timeline__type">
                              {item.type.replace("_", " ")}
                            </span>

                            <h3>{item.title}</h3>

                            <p>{item.subtitle}</p>
                          </div>

                          <time>
                            Event time:{" "}
                            {new Date(item.timestamp).toLocaleString()}
                          </time>
                        </article>
                      ))}
                    </div>
                  </section>
                );
              })}
          </div>
        </div>

        <div className="investigation-workspace__detail">
          <InvestigationDetailPanel />
        </div>
      </div>
    </section>
  );
}
