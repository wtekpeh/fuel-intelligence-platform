import type { ReactNode } from "react";
import "./BottomSheet.css";

interface BottomSheetProps {
  open: boolean;
  title: string;
  children: ReactNode;
  onClose: () => void;
  size?: "small" | "medium" | "large";
}

export default function BottomSheet({
  open,
  title,
  children,
  onClose,
  size = "medium",
}: BottomSheetProps) {
  if (!open) {
    return null;
  }

  return (
    <div className="bottom-sheet" role="dialog" aria-modal="true">
      <button
        type="button"
        className="bottom-sheet__backdrop"
        aria-label="Close bottom sheet"
        onClick={onClose}
      />

      <section className={`bottom-sheet__panel bottom-sheet__panel--${size}`}>
        <div className="bottom-sheet__handle" />

        <header className="bottom-sheet__header">
          <h2>{title}</h2>

          <button type="button" onClick={onClose}>
            Close
          </button>
        </header>

        <div className="bottom-sheet__body">{children}</div>
      </section>
    </div>
  );
}
