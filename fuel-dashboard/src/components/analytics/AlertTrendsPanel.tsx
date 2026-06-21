import { useEffect } from "react";
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
} from "recharts";
import { useAnalyticsStore } from "../../store/analyticsStore";
import DeviceHealthTrendsPanel from "./DeviceHealthTrendsPanel";
import GeofenceUtilizationPanel from "./GeofenceUtilizationPanel";
import { useFleetStore } from "../../store/fleetStore";
import GeofenceActivityPanel from "./GeofenceActivityPanel";
import AnalyticsFilterBar from "./AnalyticsFilterBar";

type AlertTrendTooltipPayload = {
  name: string;
  value: number;
  color: string;
};

type AlertTrendTooltipProps = {
  active?: boolean;
  payload?: AlertTrendTooltipPayload[];
  label?: string;
};

const AlertTrendTooltip = ({
  active,
  payload,
  label,
}: AlertTrendTooltipProps) => {
  if (!active || !payload || payload.length === 0) {
    return null;
  }

  return (
    <div className="analytics-chart-tooltip">
      <strong>{label}</strong>

      {payload.map((entry) => (
        <div key={entry.name} className="analytics-chart-tooltip__row">
          <span style={{ color: entry.color }}>{entry.name}</span>
          <b>{entry.value}</b>
        </div>
      ))}
    </div>
  );
};

export default function AlertTrendsPanel() {
  const {
    alertTrends,
    loadingAlertTrends,
    alertTrendsError,
    fetchAlertTrends,
    selectedDays,
  } = useAnalyticsStore();

  const { selectedDevice } = useFleetStore();

  const sensorTypes = selectedDevice?.sensor_types ?? [];

  const normalizedSensorTypes = sensorTypes.map((sensorType) => {
    return sensorType.toUpperCase();
  });

  const hasFuelSensor = normalizedSensorTypes.some((sensorType) => {
    return sensorType.includes("FUEL");
  });

  const hasGpsSensor = normalizedSensorTypes.some((sensorType) => {
    return sensorType.includes("GPS");
  });

  useEffect(() => {
    fetchAlertTrends(selectedDays);
  }, [fetchAlertTrends, selectedDays]);

  const dailyTrendRows = Object.values(
    (alertTrends?.trend ?? []).reduce<
      Record<
        string,
        {
          day: string;
          theft: number;
          refill: number;
          leak: number;
          total: number;
        }
      >
    >((dailyTotals, item) => {
      if (!dailyTotals[item.day]) {
        dailyTotals[item.day] = {
          day: item.day,
          theft: 0,
          refill: 0,
          leak: 0,
          total: 0,
        };
      }

      if (item.alert_type === "THEFT") {
        dailyTotals[item.day].theft += item.count;
      }

      if (item.alert_type === "REFILL") {
        dailyTotals[item.day].refill += item.count;
      }

      if (item.alert_type === "LEAK") {
        dailyTotals[item.day].leak += item.count;
      }

      dailyTotals[item.day].total += item.count;

      return dailyTotals;
    }, {}),
  );

  const alertTrendChartData = dailyTrendRows.map((row) => {
    return {
      day: new Date(row.day).toLocaleDateString("en-GB", {
        day: "2-digit",
        month: "short",
      }),
      theft: row.theft,
      refill: row.refill,
      leak: row.leak,
    };
  });

  if (loadingAlertTrends) {
    return <p>Loading alert trends...</p>;
  }

  if (alertTrendsError) {
    return <p>{alertTrendsError}</p>;
  }

  return (
    <section>
      {hasFuelSensor && (
        <>
          <h2>Alert Trends</h2>

          <AnalyticsFilterBar />

          <div className="fleet-attention-center">
            <div className="fleet-attention-card">
              <label>Total Alerts</label>
              <strong>{alertTrends?.summary.total_alerts ?? 0}</strong>
            </div>

            <div className="fleet-attention-card">
              <label>Theft Alerts</label>
              <strong>{alertTrends?.summary.theft_alerts ?? 0}</strong>
            </div>

            <div className="fleet-attention-card">
              <label>Refill Alerts</label>
              <strong>{alertTrends?.summary.refill_alerts ?? 0}</strong>
            </div>

            <div className="fleet-attention-card">
              <label>Leak Alerts</label>
              <strong>{alertTrends?.summary.leak_alerts ?? 0}</strong>
            </div>
          </div>

          <div className="fleet-card analytics-chart-card">
            <div className="analytics-chart-header">
              <h2>Alert Type Trend</h2>

              <p>
                Theft, refill, and leak activity across the selected reporting
                period.
              </p>
            </div>

            <div className="analytics-chart">
              <ResponsiveContainer width="100%" height={320}>
                <LineChart data={alertTrendChartData}>
                  <CartesianGrid
                    stroke="rgba(148, 163, 184, 0.16)"
                    strokeDasharray="4 8"
                    vertical={false}
                  />

                  <XAxis
                    dataKey="day"
                    tick={{ fill: "#94a3b8", fontSize: 12 }}
                    axisLine={{ stroke: "rgba(148, 163, 184, 0.25)" }}
                    tickLine={false}
                  />

                  <YAxis
                    allowDecimals={false}
                    tick={{ fill: "#94a3b8", fontSize: 12 }}
                    axisLine={false}
                    tickLine={false}
                    label={{
                      value: "Alert count",
                      angle: -90,
                      position: "insideLeft",
                      fill: "#94a3b8",
                    }}
                  />
                  <Tooltip content={<AlertTrendTooltip />} />
                  <Legend />

                  <Line
                    type="monotone"
                    dataKey="theft"
                    name="Theft"
                    stroke="#ef4444"
                    strokeWidth={3}
                  />

                  <Line
                    type="monotone"
                    dataKey="refill"
                    name="Refill"
                    stroke="#22c55e"
                    strokeWidth={3}
                  />

                  <Line
                    type="monotone"
                    dataKey="leak"
                    name="Leak"
                    stroke="#f59e0b"
                    strokeWidth={3}
                  />
                </LineChart>
              </ResponsiveContainer>
            </div>
          </div>

          <div className="fleet-card">
            <h2>Daily Alert Trend</h2>

            <div className="table-scroll">
              <table className="analytics-table">
                <thead>
                  <tr>
                    <th>Date</th>
                    <th>Theft</th>
                    <th>Refill</th>
                    <th>Leak</th>
                    <th>Total</th>
                  </tr>
                </thead>

                <tbody>
                  {dailyTrendRows.map((row) => (
                    <tr key={row.day}>
                      <td>{row.day}</td>
                      <td>{row.theft}</td>
                      <td>{row.refill}</td>
                      <td>{row.leak}</td>
                      <td>{row.total}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        </>
      )}

      {hasGpsSensor && <GeofenceActivityPanel />}

      <h2 className="analytics-section-title">Infrastructure & Operations</h2>

      <div
        className={
          hasGpsSensor
            ? "analytics-insights-grid"
            : "analytics-insights-grid analytics-insights-grid--single"
        }
      >
        <DeviceHealthTrendsPanel />

        {hasGpsSensor && <GeofenceUtilizationPanel />}
      </div>
    </section>
  );
}
