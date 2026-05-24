import { useAlertStore } from "../../store/alertStore";
import type { AlertResponse } from "../../types";

interface MobileAlertCardProps {
  alert: AlertResponse;
}

export function MobileAlertCard({ alert }: MobileAlertCardProps) {
  const setSelectedAlert = useAlertStore((state) => state.setSelectedAlert);

  return (
    <article
      className={[
        "mobile-alert-card",
        alert.status === "OPEN" ? "mobile-alert-card--attention" : "",
      ].join(" ")}
      onClick={() => setSelectedAlert(alert)}
    >
      <div className="mobile-alert-card__top">
        <div>
          <p className="mobile-alert-card__type">{alert.alert_type}</p>

          <span className="mobile-alert-card__time">
            {new Date(alert.created_at).toLocaleString()}
          </span>
        </div>

        <span
          className={`alert-pill ${
            alert.status === "RESOLVED"
              ? "alert-pill--resolved"
              : alert.status === "ACKNOWLEDGED"
                ? "alert-pill--acknowledged"
                : "alert-pill--open"
          }`}
        >
          {alert.status}
        </span>
      </div>

      <div className="mobile-alert-card__middle">
        <span
          className={
            alert.severity === "Critical"
              ? "alert-pill alert-pill--danger"
              : "alert-pill alert-pill--warning"
          }
        >
          {alert.severity}
        </span>
      </div>

      <p className="mobile-alert-card__reason">{alert.reason}</p>
    </article>
  );
}
