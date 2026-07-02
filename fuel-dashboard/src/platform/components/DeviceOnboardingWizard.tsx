import { useState } from "react";
import { useOrganizationStore } from "../store/organizationStore";

interface WizardOrganization {
  organization_id: string;
  organization_name: string;
  industry?: string | null;
  asset_count: number;
  device_count: number;
  online_device_count: number;
}

interface DeviceOnboardingWizardProps {
  open: boolean;
  onClose: () => void;
  organizations: WizardOrganization[];
}
const steps = ["Organization", "Asset", "Device Model", "Review", "Complete"];

export default function DeviceOnboardingWizard({
  open,
  onClose,
  organizations,
}: DeviceOnboardingWizardProps) {
  const [selectedOrganizationId, setSelectedOrganizationId] = useState<
    string | null
  >(null);

  const [showCreateOrganization, setShowCreateOrganization] = useState(false);
  const [organizationName, setOrganizationName] = useState("");
  const [industry, setIndustry] = useState("");

  const createOrganization = useOrganizationStore(
    (state) => state.createOrganization,
  );

  if (!open) {
    return null;
  }

  return (
    <div className="platform-wizard-overlay">
      <section className="platform-wizard">
        <header className="platform-wizard__header">
          <div>
            <p className="platform-eyebrow">Device Onboarding</p>
            <h2>Provision a New Device</h2>
          </div>

          <button type="button" onClick={onClose}>
            Close
          </button>
        </header>

        <div className="platform-wizard__steps">
          {steps.map((step, index) => (
            <div
              key={step}
              className={`platform-wizard__step ${
                index === 0 ? "platform-wizard__step--active" : ""
              }`}
            >
              <span>{index + 1}</span>
              <p>{step}</p>
            </div>
          ))}
        </div>

        <div className="platform-wizard__body">
          <p className="platform-eyebrow">Step 1</p>

          <h3>Select Organization</h3>

          <p>Choose the customer account that will own this device.</p>

          <div
            style={{
              display: "flex",
              justifyContent: "flex-end",
              marginTop: "18px",
              marginBottom: "18px",
            }}
          >
            <button
              type="button"
              className="platform-primary-button"
              onClick={() => setShowCreateOrganization(!showCreateOrganization)}
            >
              + Create Organization
            </button>
          </div>

          {showCreateOrganization && (
            <div className="platform-form">
              <label>
                Organization Name
                <input
                  value={organizationName}
                  onChange={(event) => setOrganizationName(event.target.value)}
                />
              </label>

              <label>
                Business Type / Sector
                <input
                  value={industry}
                  onChange={(event) => setIndustry(event.target.value)}
                  placeholder="Example: Transport, Mining, Logistics"
                />
              </label>

              <div
                style={{
                  display: "flex",
                  justifyContent: "flex-end",
                  gap: "12px",
                }}
              >
                <button
                  type="button"
                  className="platform-secondary-button"
                  onClick={() => setShowCreateOrganization(false)}
                >
                  Cancel
                </button>

                <button
                  type="button"
                  className="platform-primary-button"
                  onClick={async () => {
                    const id = await createOrganization({
                      name: organizationName,
                      industry,
                    });

                    if (id) {
                      setSelectedOrganizationId(id);
                      setOrganizationName("");
                      setIndustry("");
                      setShowCreateOrganization(false);
                    }
                  }}
                >
                  Create Organization
                </button>
              </div>
            </div>
          )}

          <div className="platform-list" style={{ marginTop: "24px" }}>
            {organizations.map((organization) => (
              <button
                key={organization.organization_id}
                type="button"
                onClick={() =>
                  setSelectedOrganizationId(organization.organization_id)
                }
                className={`platform-list-card ${
                  selectedOrganizationId === organization.organization_id
                    ? "platform-list-card--selected"
                    : ""
                }`}
              >
                <div>
                  <p>{organization.industry ?? "Organization"}</p>

                  <h3>{organization.organization_name}</h3>

                  <span>
                    {organization.asset_count} Assets •{" "}
                    {organization.device_count} Devices
                  </span>
                </div>

                <strong>{organization.online_device_count} Online</strong>
              </button>
            ))}
          </div>
        </div>

        <footer className="platform-wizard__footer">
          <button type="button" onClick={onClose}>
            Cancel
          </button>

          <div>
            <button type="button" disabled>
              Back
            </button>

            <button type="button" className="platform-primary-button">
              Next
            </button>
          </div>
        </footer>
      </section>
    </div>
  );
}
