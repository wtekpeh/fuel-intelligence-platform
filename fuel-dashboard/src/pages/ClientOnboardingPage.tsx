import { useState, type FormEvent } from "react";
import axios from "axios";

import { createClientOnboarding } from "../api/onboardingApi";
import { logout } from "../auth/logout";

import "../styles/client-onboarding.css";

interface ClientOnboardingPageProps {
  onComplete: () => void;
}

export function ClientOnboardingPage({
  onComplete,
}: ClientOnboardingPageProps) {
  const [organizationName, setOrganizationName] = useState("");
  const [industry, setIndustry] = useState("");

  const [submitting, setSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();

    const name = organizationName.trim();
    const selectedIndustry = industry.trim();

    if (!name || !selectedIndustry) {
      setError("Organization name and industry are required.");
      return;
    }

    if (name.length > 200 || selectedIndustry.length > 100) {
      setError("Organization information exceeds the allowed length.");
      return;
    }

    setSubmitting(true);
    setError(null);

    try {
      await createClientOnboarding({
        organization_name: name,
        industry: selectedIndustry,
      });

      onComplete();
    } catch (err: unknown) {
      if (axios.isAxiosError(err) && err.response?.status === 409) {
        setError(
          "This account is already registered. Please refresh the page.",
        );
      } else {
        setError("Unable to complete registration. Please try again.");
      }
    } finally {
      setSubmitting(false);
    }
  }

  return (
    <main className="client-onboarding">
      <section className="client-onboarding__card">
        <div className="client-onboarding__header">
          <p className="client-onboarding__eyebrow">ORBI Sensor Intelligence</p>

          <h1>Welcome to ORBI</h1>

          <p>
            Set up your organization to start managing your fleet, connected
            devices, and operational intelligence.
          </p>
        </div>

        <form onSubmit={(event) => void handleSubmit(event)}>
          <div className="client-onboarding__field">
            <label htmlFor="organization-name">Organization Name</label>

            <input
              id="organization-name"
              type="text"
              value={organizationName}
              onChange={(event) => setOrganizationName(event.target.value)}
              placeholder="Enter your organization name"
              maxLength={200}
              required
              disabled={submitting}
            />
          </div>

          <div className="client-onboarding__field">
            <label htmlFor="organization-industry">Industry</label>

            <input
              id="organization-industry"
              type="text"
              value={industry}
              onChange={(event) => setIndustry(event.target.value)}
              placeholder="e.g. Transport and Logistics"
              maxLength={100}
              required
              disabled={submitting}
            />
          </div>

          {error && (
            <p className="client-onboarding__error" role="alert">
              {error}
            </p>
          )}

          <button
            type="submit"
            className="client-onboarding__submit"
            disabled={submitting}
          >
            {submitting ? "Creating Organization..." : "Create Organization"}
          </button>
        </form>

        <button
          type="button"
          className="client-onboarding__signout"
          onClick={() => void logout()}
          disabled={submitting}
        >
          Sign Out
        </button>
      </section>
    </main>
  );
}
