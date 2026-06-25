import { useOrganizationOverview } from "../services/useOrganizationOverview";

import { useOrganizationStore } from "../store/organizationStore";

import { useAppViewStore } from "../store/appViewStore";

import "../styles/landing.css";

export function LandingPage() {
  useOrganizationOverview();

  const organizations = useOrganizationStore((state) => state.organizations);

  const selectOrganization = useOrganizationStore(
    (state) => state.selectOrganization,
  );

  const showFleet = useAppViewStore((state) => state.showFleet);
  const showPlatform = useAppViewStore((state) => state.showPlatform);

  return (
    <main className="landing-page">
      <section className="landing-hero">
        <p className="landing-eyebrow">Fuel Intelligence Platform</p>

        <h1 className="landing-title">Operational Fleet Intelligence</h1>

        <p className="landing-subtitle">
          Monitor organizations, assets, sensors, fuel events, and operational
          intelligence from a centralized platform.
        </p>

        <div className="landing-actions">
          <button
            type="button"
            className="landing-platform-button"
            onClick={showPlatform}
          >
            Platform Management
          </button>
        </div>
      </section>

      <section className="organization-grid">
        {organizations.map((organization) => (
          <article
            key={organization.organization_id}
            className="organization-card"
          >
            <div className="organization-card__top">
              <div>
                <p className="organization-card__industry">
                  {organization.industry ?? "Unknown Industry"}
                </p>

                <h2>{organization.organization_name}</h2>
              </div>

              <span className="organization-card__alerts">
                {organization.open_alert_count} alerts
              </span>
            </div>

            <div className="organization-card__stats">
              <div>
                <label>Assets</label>

                <strong>{organization.asset_count}</strong>
              </div>

              <div>
                <label>Devices</label>

                <strong>{organization.device_count}</strong>
              </div>

              <div>
                <label>Online</label>

                <strong className="organization-card__online">
                  {organization.online_device_count}
                </strong>
              </div>

              <div>
                <label>Offline</label>

                <strong className="organization-card__offline">
                  {organization.offline_device_count}
                </strong>
              </div>
            </div>

            <button
              type="button"
              className="organization-card__button"
              onClick={() => {
                selectOrganization(organization);

                showFleet();
              }}
            >
              View Fleet Overview
            </button>
          </article>
        ))}

        {organizations.length === 0 && (
          <div className="organization-grid__empty">
            No organizations available.
          </div>
        )}
      </section>
    </main>
  );
}
