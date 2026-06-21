import { useEffect } from "react";
import {
  Line,
  LineChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
  CartesianGrid,
  Legend,
} from "recharts";
import { useAnalyticsStore } from "../../store/analyticsStore";

export default function GeofenceActivityPanel() {
  const {
    geofenceActivityTrends,
    loadingGeofenceActivityTrends,
    geofenceActivityTrendsError,
    fetchGeofenceActivityTrends,
    selectedDays,
  } = useAnalyticsStore();

  useEffect(() => {
    fetchGeofenceActivityTrends(selectedDays);
  }, [fetchGeofenceActivityTrends, selectedDays]);

  if (loadingGeofenceActivityTrends) {
    return <p>Loading geofence activity...</p>;
  }

  if (geofenceActivityTrendsError) {
    return <p>{geofenceActivityTrendsError}</p>;
  }

  const chartData =
    geofenceActivityTrends?.trend.map((item) => ({
      date: new Date(item.day).toLocaleDateString("en-GB", {
        day: "2-digit",
        month: "short",
      }),
      entries: item.entries,
      exits: item.exits,
    })) ?? [];

  return (
    <section className="fleet-card analytics-chart-card">
      <h2>Geofence Activity Trend</h2>

      <p className="analytics-chart-description">
        Zone entry and exit activity across the selected reporting period.
      </p>

      <div className="analytics-chart-container">
        <ResponsiveContainer width="100%" height={320}>
          <LineChart data={chartData}>
            <CartesianGrid
              stroke="rgba(148, 163, 184, 0.15)"
              strokeDasharray="4 4"
            />

            <XAxis
              dataKey="date"
              tick={{ fill: "#94a3b8" }}
              axisLine={false}
              tickLine={false}
            />

            <YAxis
              tick={{ fill: "#94a3b8" }}
              axisLine={false}
              tickLine={false}
              label={{
                value: "Activity Count",
                angle: -90,
                position: "insideLeft",
                fill: "#94a3b8",
              }}
            />

            <Tooltip />

            <Legend />

            <Line
              type="monotone"
              dataKey="entries"
              stroke="#22c55e"
              strokeWidth={3}
              dot={{ r: 4 }}
              activeDot={{ r: 7 }}
              name="Entries"
            />

            <Line
              type="monotone"
              dataKey="exits"
              stroke="#f59e0b"
              strokeWidth={3}
              dot={{ r: 4 }}
              activeDot={{ r: 7 }}
              name="Exits"
            />
          </LineChart>
        </ResponsiveContainer>
      </div>
    </section>
  );
}
