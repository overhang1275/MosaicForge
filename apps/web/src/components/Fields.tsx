import { useId, type ReactNode } from 'react';

export function Field({
  label,
  value,
  onChange,
  hint,
  min,
  max,
  step = 'any',
  decimal = false,
}: {
  label: string;
  value: string;
  onChange: (value: string) => void;
  hint?: string;
  min?: number;
  max?: number;
  step?: string;
  decimal?: boolean;
}) {
  const inputId = useId();
  return (
    <label className="field" htmlFor={inputId}>
      <span>{label}</span>
      <input
        id={inputId}
        name={label.toLowerCase().replaceAll(/\s+/g, '-')}
        autoComplete="off"
        spellCheck={false}
        type={decimal ? 'text' : 'number'}
        inputMode={decimal ? 'numeric' : 'decimal'}
        value={value}
        min={min}
        max={max}
        step={step}
        onChange={(event) => onChange(event.target.value)}
      />
      {hint && <small>{hint}</small>}
    </label>
  );
}

export function Section({
  title,
  children,
  open = false,
}: {
  title: string;
  children: ReactNode;
  open?: boolean;
}) {
  return (
    <details className="inspector-section" open={open}>
      <summary>
        {title}
        <span className="disclosure" aria-hidden="true" />
      </summary>
      <div className="section-content">{children}</div>
    </details>
  );
}

export function ImageIcon() {
  return (
    <svg
      className="image-icon"
      viewBox="0 0 48 48"
      fill="none"
      aria-hidden="true"
    >
      <rect x="6" y="6" width="36" height="36" rx="5" />
      <circle cx="32" cy="16" r="4" />
      <path d="m7 34 12-13 12 13m-4-5 6-6 9 10" />
    </svg>
  );
}
