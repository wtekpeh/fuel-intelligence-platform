import { useAnalyticsStore } from "../../store/analyticsStore";

const dayOptions = [7, 30, 90];

export default function AnalyticsFilterBar() {
  const { selectedDays, setSelectedDays } = useAnalyticsStore();

  return (
    <div className="analytics-filter-bar">
      <div>
        <p>Analytics Period</p>
        <span>Apply one reporting window across all analytics panels.</span>
      </div>

      <div className="analytics-filter-options">
        {dayOptions.map((days) => (
          <button
            key={days}
            type="button"
            className={
              selectedDays === days
                ? "analytics-filter-button analytics-filter-button--active"
                : "analytics-filter-button"
            }
            onClick={() => setSelectedDays(days)}
          >
            Last {days} Days
          </button>
        ))}
      </div>
    </div>
  );
}
