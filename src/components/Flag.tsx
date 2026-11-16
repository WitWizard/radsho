import { useId, type ReactNode } from "react";

interface FlagProps {
  code: string;
  size?: number;
  ring?: boolean;
  className?: string;
}

function hStripes(colors: string[]): ReactNode {
  const height = 24 / colors.length;
  return colors.map((color, index) => <rect key={index} x="0" y={index * height} width="24" height={height + 0.25} fill={color} />);
}

function vStripes(colors: string[]): ReactNode {
  const width = 24 / colors.length;
  return colors.map((color, index) => <rect key={index} x={index * width} y="0" width={width + 0.25} height="24" fill={color} />);
}

function nordicCross(bg: string, cross: string, inner?: string): ReactNode {
  return (
    <>
      <rect width="24" height="24" fill={bg} />
      <rect x="6.5" width="4.5" height="24" fill={cross} />
      <rect y="9.75" width="24" height="4.5" fill={cross} />
      {inner && (
        <>
          <rect x="7.6" width="2.3" height="24" fill={inner} />
          <rect y="10.85" width="24" height="2.3" fill={inner} />
        </>
      )}
    </>
  );
}

function usFlag(): ReactNode {
  const stripes = Array.from({ length: 7 }, (_, index) => (
    <rect key={index} x="0" y={index * 3.4285} width="24" height="1.7142" fill="#B22234" />
  ));
  const stars = [3, 6, 9].flatMap((x) => [2.5, 5.5, 8.5].map((y) => <circle key={`${x}-${y}`} cx={x} cy={y} r="0.65" fill="#fff" />));
  return (
    <>
      <rect width="24" height="24" fill="#fff" />
      {stripes}
      <rect width="11" height="12.5" fill="#3C3B6E" />
      {stars}
    </>
  );
}

function ukJack(): ReactNode {
  return (
    <>
      <rect width="24" height="24" fill="#012169" />
      <path d="M0 0 L24 24 M24 0 L0 24" stroke="#fff" strokeWidth="4" />
      <path d="M0 0 L24 24 M24 0 L0 24" stroke="#C8102E" strokeWidth="1.5" />
      <path d="M12 0v24M0 12h24" stroke="#fff" strokeWidth="6" />
      <path d="M12 0v24M0 12h24" stroke="#C8102E" strokeWidth="3.4" />
    </>
  );
}

function renderFlag(code: string): ReactNode {
  switch (code.toUpperCase()) {
    case "US":
      return usFlag();
    case "GB":
      return ukJack();
    case "DE":
      return hStripes(["#000000", "#DD0000", "#FFCE00"]);
    case "NL":
      return hStripes(["#AE1C28", "#FFFFFF", "#21468B"]);
    case "IT":
      return vStripes(["#009246", "#FFFFFF", "#CE2B37"]);
    case "FR":
      return vStripes(["#0055A4", "#FFFFFF", "#EF4135"]);
    case "BE":
      return vStripes(["#000000", "#FDDA24", "#EF3340"]);
    case "IE":
      return vStripes(["#169B62", "#FFFFFF", "#FF883E"]);
    case "RO":
      return vStripes(["#002B7F", "#FCD116", "#CE1126"]);
    case "MD":
      return (
        <>
          {vStripes(["#003DA5", "#FFD100", "#C8102E"])}
          <rect x="10.2" y="9" width="3.6" height="6" rx="0.8" fill="#8C5A2B" />
        </>
      );
    case "ES":
      return (
        <>
          <rect width="24" height="24" fill="#AA151B" />
          <rect y="6" width="24" height="12" fill="#F1BF00" />
        </>
      );
    case "PT":
      return (
        <>
          <rect width="24" height="24" fill="#DA291C" />
          <rect width="9.5" height="24" fill="#046A38" />
          <circle cx="9.5" cy="12" r="3.2" fill="#FFE900" />
          <rect x="8.3" y="10.8" width="2.4" height="2.4" rx="0.5" fill="#DA291C" />
        </>
      );
    case "AT":
      return hStripes(["#ED2939", "#FFFFFF", "#ED2939"]);
    case "LV":
      return (
        <>
          <rect width="24" height="24" fill="#9E3039" />
          <rect y="9.5" width="24" height="5" fill="#FFFFFF" />
        </>
      );
    case "PL":
      return hStripes(["#FFFFFF", "#DC143C"]);
    case "UA":
      return hStripes(["#0057B7", "#FFD700"]);
    case "EE":
      return hStripes(["#0072CE", "#000000", "#FFFFFF"]);
    case "LT":
      return hStripes(["#FDB913", "#006A44", "#C1272D"]);
    case "BG":
      return hStripes(["#FFFFFF", "#00966E", "#D62612"]);
    case "HU":
      return hStripes(["#CD2A3E", "#FFFFFF", "#436F4D"]);
    case "HR":
      return (
        <>
          {hStripes(["#FF0000", "#FFFFFF", "#171796"])}
          <rect x="9.5" y="8.5" width="5" height="6" rx="1" fill="#FF0000" stroke="#fff" strokeWidth="0.7" />
        </>
      );
    case "SI":
      return (
        <>
          {hStripes(["#FFFFFF", "#005CE6", "#ED1C24"])}
          <rect x="5" y="6" width="4" height="5" rx="1" fill="#005CE6" stroke="#fff" strokeWidth="0.6" />
        </>
      );
    case "RS":
      return (
        <>
          {hStripes(["#C6363C", "#0C4076", "#FFFFFF"])}
          <rect x="5" y="7" width="5" height="9" rx="1.2" fill="#C6363C" />
          <rect x="6.3" y="8.8" width="2.4" height="3.6" fill="#fff" opacity="0.85" />
        </>
      );
    case "CZ":
      return (
        <>
          <rect width="24" height="12" fill="#fff" />
          <rect y="12" width="24" height="12" fill="#D7141A" />
          <path d="M0 0 L13 12 L0 24 Z" fill="#11457E" />
        </>
      );
    case "SK":
      return (
        <>
          {hStripes(["#FFFFFF", "#0B4EA2", "#EE1C25"])}
          <path d="M5 6.5h6v6.2c0 2.2-1.8 3.6-3 4.1-1.2-.5-3-1.9-3-4.1z" fill="#EE1C25" stroke="#fff" strokeWidth="0.8" />
          <path d="M8 8v7M6 10h4" stroke="#fff" strokeWidth="1" />
        </>
      );
    case "CH":
      return (
        <>
          <rect width="24" height="24" fill="#D52B1E" />
          <path d="M10 5h4v5h5v4h-5v5h-4v-5H5v-4h5z" fill="#fff" />
        </>
      );
    case "SE":
      return nordicCross("#006AA7", "#FECC00");
    case "DK":
      return nordicCross("#C8102E", "#FFFFFF");
    case "FI":
      return nordicCross("#FFFFFF", "#002F6C");
    case "NO":
      return nordicCross("#BA0C2F", "#FFFFFF", "#00205B");
    case "IS":
      return nordicCross("#02529C", "#FFFFFF", "#DC1E35");
    case "GR":
      return (
        <>
          {hStripes(["#0D5EAF", "#FFFFFF", "#0D5EAF", "#FFFFFF", "#0D5EAF", "#FFFFFF", "#0D5EAF", "#FFFFFF", "#0D5EAF"])}
          <rect width="11" height="13.4" fill="#0D5EAF" />
          <path d="M5.5 0v13.4M0 6.7h11" stroke="#FFFFFF" strokeWidth="2.4" />
        </>
      );
    case "TR":
      return (
        <>
          <rect width="24" height="24" fill="#E30A17" />
          <circle cx="10.2" cy="12" r="5" fill="#fff" />
          <circle cx="11.5" cy="12" r="4" fill="#E30A17" />
          <polygon points="15.2,12 18.2,10.9 16.5,13.5 16.5,10.5 18.2,13.1" fill="#fff" />
        </>
      );
    case "AE":
      return (
        <>
          {hStripes(["#00732F", "#FFFFFF", "#000000"])}
          <rect width="6.5" height="24" fill="#FF0000" />
        </>
      );
    case "CA":
      return (
        <>
          <rect width="24" height="24" fill="#FFFFFF" />
          <rect width="5.5" height="24" fill="#D80621" />
          <rect x="18.5" width="5.5" height="24" fill="#D80621" />
          <path d="M12 6l1.5 3.2 3-.6-1.8 2.8 2.3 1.4-3.8.5v3.2h-2.4v-3.2l-3.8-.5 2.3-1.4-1.8-2.8 3 .6z" fill="#D80621" />
        </>
      );
    case "BR":
      return (
        <>
          <rect width="24" height="24" fill="#009739" />
          <polygon points="12,3.5 21.5,12 12,20.5 2.5,12" fill="#FEDD00" />
          <circle cx="12" cy="12" r="4.2" fill="#012169" />
          <path d="M8.2 11.2c2.5-.8 5.2-.5 7.6.8" stroke="#fff" strokeWidth="1" fill="none" />
        </>
      );
    case "JM":
      return (
        <>
          <rect width="24" height="24" fill="#009B3A" />
          <path d="M0 0 L12 12 L0 24 Z M24 0 L12 12 L24 24 Z" fill="#000" />
          <path d="M0 0 L24 24 M24 0 L0 24" stroke="#FED100" strokeWidth="3.4" />
        </>
      );
    case "JP":
      return (
        <>
          <rect width="24" height="24" fill="#FFFFFF" />
          <circle cx="12" cy="12" r="5.5" fill="#BC002D" />
        </>
      );
    case "KR":
      return (
        <>
          <rect width="24" height="24" fill="#FFFFFF" />
          <circle cx="12" cy="12" r="5.2" fill="#CD2E3A" />
          <path d="M6.8 12a2.6 2.6 0 0 0 5.2 0 2.6 2.6 0 0 1 5.2 0 5.2 5.2 0 0 1-10.4 0z" fill="#0047A0" />
        </>
      );
    case "SG":
      return (
        <>
          {hStripes(["#EF3340", "#FFFFFF"])}
          <circle cx="7.2" cy="6.2" r="3.4" fill="#fff" />
          <circle cx="8.3" cy="6.2" r="3" fill="#EF3340" />
        </>
      );
    case "HK":
      return (
        <>
          <rect width="24" height="24" fill="#DE2910" />
          <circle cx="12" cy="12" r="4.8" fill="none" stroke="#fff" strokeWidth="2.2" strokeDasharray="3 1.8" />
        </>
      );
    case "TW":
      return (
        <>
          <rect width="24" height="24" fill="#FE0000" />
          <rect width="12" height="12" fill="#000095" />
          <circle cx="6" cy="6" r="3" fill="#FFFFFF" />
        </>
      );
    case "IN":
      return (
        <>
          {hStripes(["#FF671F", "#FFFFFF", "#046A38"])}
          <circle cx="12" cy="12" r="2.7" fill="none" stroke="#06038D" strokeWidth="1.2" />
          <circle cx="12" cy="12" r="0.8" fill="#06038D" />
        </>
      );
    case "AU":
      return (
        <>
          <rect width="24" height="24" fill="#012169" />
          <g transform="scale(0.5)">{ukJack()}</g>
          <circle cx="6" cy="18" r="1.8" fill="#fff" />
          <circle cx="18" cy="7" r="1.1" fill="#fff" />
          <circle cx="15" cy="12" r="1.1" fill="#fff" />
          <circle cx="20" cy="13" r="1.1" fill="#fff" />
          <circle cx="18" cy="18.5" r="1.2" fill="#fff" />
        </>
      );
    case "NZ":
      return (
        <>
          <rect width="24" height="24" fill="#00247D" />
          <g transform="scale(0.5)">{ukJack()}</g>
          <circle cx="18" cy="7" r="1.2" fill="#CC142B" stroke="#fff" strokeWidth="0.4" />
          <circle cx="15.5" cy="12" r="1.2" fill="#CC142B" stroke="#fff" strokeWidth="0.4" />
          <circle cx="20" cy="13" r="1.2" fill="#CC142B" stroke="#fff" strokeWidth="0.4" />
          <circle cx="18" cy="18.5" r="1.3" fill="#CC142B" stroke="#fff" strokeWidth="0.4" />
        </>
      );
    case "ZA":
      return (
        <>
          <rect width="24" height="12" fill="#E03C31" />
          <rect y="12" width="24" height="12" fill="#001489" />
          <path d="M0 2 L11 12 L0 22 M10 12 H24" stroke="#FFFFFF" strokeWidth="5.5" fill="none" />
          <path d="M0 2 L11 12 L0 22 M10 12 H24" stroke="#007749" strokeWidth="3.5" fill="none" />
          <polygon points="0,4.5 8,12 0,19.5" fill="#000000" stroke="#FFB81C" strokeWidth="1.2" />
        </>
      );
    default:
      return <rect width="24" height="24" fill="#64748B" />;
  }
}

export function Flag({ code, size = 40, ring = false, className = "" }: FlagProps) {
  const id = useId();
  return (
    <span
      className={`inline-flex shrink-0 rounded-full ${ring ? "p-[3px] ring-2 ring-mint" : ""} ${className}`}
      style={{ width: size, height: size }}
    >
      <svg viewBox="0 0 24 24" width="100%" height="100%" aria-hidden="true">
        <defs>
          <clipPath id={id}>
            <circle cx="12" cy="12" r="12" />
          </clipPath>
        </defs>
        <g clipPath={`url(#${id})`}>{renderFlag(code)}</g>
      </svg>
    </span>
  );
}