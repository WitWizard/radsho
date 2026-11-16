interface LogoProps {
  size?: number;
  className?: string;
}

export function Logo({ size = 36, className = "" }: LogoProps) {
  return (
    <svg
      viewBox="0 0 32 40"
      width={size * 0.8}
      height={size}
      className={className}
      role="img"
      aria-label="Radsho"
    >
      <rect x="9" y="1" width="14" height="8" rx="1.5" fill="#00E599" />
      <rect x="3" y="11" width="26" height="6" rx="1.5" fill="#00E599" />
      <rect x="3" y="19" width="26" height="6" rx="1.5" fill="#00E599" />
      <rect x="11" y="12.5" width="10" height="3" fill="#0E1514" />
      <path d="M9 27h14v5.5l-7 6-7-6z" fill="#00E599" />
    </svg>
  );
}
