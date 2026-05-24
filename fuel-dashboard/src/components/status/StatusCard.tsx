interface StatusCardProps {
  label: string;
  value: string | number;
  hint: string;
  tone?: "neutral" | "good" | "warning" | "danger";
}

export function StatusCard({
  label,
  value,
  hint,
  tone = "neutral",
}: StatusCardProps) {
  return (
    <article className={`status-card status-card--${tone}`}>
      <p className="status-card__label">{label}</p>
      <strong className="status-card__value">{value}</strong>
      <span className="status-card__hint">{hint}</span>
    </article>
  );
}
