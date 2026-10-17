import "./ComingSoonPage.css";

interface ComingSoonPageProps {
  title: string;
  description: string;
}

export function ComingSoonPage({ title, description }: ComingSoonPageProps) {
  return (
    <section className="coming-soon-page">
      <h2>{title}</h2>
      <p>{description}</p>

      <div className="coming-soon-card">
        <h3>Under Development</h3>
        <p>
          This management section is being prepared. Its operational features
          are not yet available.
        </p>
      </div>
    </section>
  );
}
