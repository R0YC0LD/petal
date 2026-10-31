// Petal ikon seti: 24×24, currentColor, tutarlı 1.75 px çizgi. Emoji veya unicode glifi kullanılmaz.
import type { ReactNode } from "react";

interface P {
  size?: number;
  className?: string;
}

const base = (children: ReactNode, { size = 18, className }: P, fill = false) => (
  <svg
    width={size}
    height={size}
    viewBox="0 0 24 24"
    className={className}
    fill={fill ? "currentColor" : "none"}
    stroke={fill ? "none" : "currentColor"}
    strokeWidth="1.75"
    strokeLinecap="round"
    strokeLinejoin="round"
    aria-hidden
  >
    {children}
  </svg>
);

// --- Medya ---
export const IconPlay = (p: P) => base(<path d="M8 5.2v13.6a.6.6 0 0 0 .92.5l10.6-6.8a.6.6 0 0 0 0-1L8.92 4.7A.6.6 0 0 0 8 5.2z" />, p, true);
export const IconPause = (p: P) => base(<><rect x="6.5" y="5" width="3.8" height="14" rx="1.1" /><rect x="13.7" y="5" width="3.8" height="14" rx="1.1" /></>, p, true);
export const IconPrev = (p: P) => base(<><rect x="5" y="5" width="2.2" height="14" rx="1" /><path d="M19 5.9v12.2a.5.5 0 0 1-.78.42l-8.4-6.1a.5.5 0 0 1 0-.84l8.4-6.1a.5.5 0 0 1 .78.42z" /></>, p, true);
export const IconNext = (p: P) => base(<><rect x="16.8" y="5" width="2.2" height="14" rx="1" /><path d="M5 5.9v12.2a.5.5 0 0 0 .78.42l8.4-6.1a.5.5 0 0 0 0-.84l-8.4-6.1A.5.5 0 0 0 5 5.9z" /></>, p, true);
export const IconShuffle = (p: P) => base(<path d="M3 6h3.5c3 0 4 3 5.5 6s2.5 6 5.5 6H21M3 18h3.5c1.5 0 2.5-1.2 3.5-2.8M13.5 8.8C14.5 7.2 15.5 6 17.5 6H21M18 3l3 3-3 3M18 15l3 3-3 3" />, p);
export const IconRepeat = (p: P) => base(<path d="M17 2l3 3-3 3M4 11V9a4 4 0 0 1 4-4h12M7 22l-3-3 3-3M20 13v2a4 4 0 0 1-4 4H4" />, p);
export const IconNote = (p: P) => base(<path d="M9 18V6l10-2v12M9 18a2.5 2.5 0 1 1-5 0 2.5 2.5 0 0 1 5 0zm10-2a2.5 2.5 0 1 1-5 0 2.5 2.5 0 0 1 5 0z" />, p);

// --- Ses / parlaklık ---
export const IconVolume = ({ level = 1, muted = false, ...p }: P & { level?: number; muted?: boolean }) =>
  base(
    <>
      <path d="M4 9.5v5h3.6L12.5 19V5L7.6 9.5H4z" fill="currentColor" stroke="none" />
      {muted ? (
        <path d="M16.5 9.5l5 5M21.5 9.5l-5 5" />
      ) : (
        <>
          {level > 0 && <path d="M15.8 9.6a3.6 3.6 0 0 1 0 4.8" />}
          {level > 0.5 && <path d="M18.4 7.2a7 7 0 0 1 0 9.6" />}
        </>
      )}
    </>,
    p,
  );
export const IconBrightness = (p: P) => base(<><circle cx="12" cy="12" r="3.8" /><path d="M12 2.8v2M12 19.2v2M2.8 12h2M19.2 12h2M5.5 5.5l1.4 1.4M17.1 17.1l1.4 1.4M5.5 18.5l1.4-1.4M17.1 6.9l1.4-1.4" /></>, p);

// --- Hava durumu ---
export const IconSun = (p: P) => base(<><circle cx="12" cy="12" r="4.2" /><path d="M12 2.5v2.2M12 19.3v2.2M2.5 12h2.2M19.3 12h2.2M5.3 5.3l1.6 1.6M17.1 17.1l1.6 1.6M5.3 18.7l1.6-1.6M17.1 6.9l1.6-1.6" /></>, p);
export const IconCloud = (p: P) => base(<path d="M7 18.5a4.5 4.5 0 0 1-.4-8.98A5.5 5.5 0 0 1 17.2 8.6 4.95 4.95 0 0 1 17 18.5H7z" />, p);
export const IconPartly = (p: P) => base(<><path d="M9.5 5V3.2M4.2 7.2L3 6M3.2 12H1.5M15.6 6.3L16.8 5" /><circle cx="9.5" cy="10" r="3.2" /><path d="M8 20a3.7 3.7 0 0 1-.3-7.4 4.6 4.6 0 0 1 8.9 1A4 4 0 0 1 16.5 20H8z" /></>, p);
export const IconRain = (p: P) => base(<><path d="M7 15.5a4.5 4.5 0 0 1-.4-8.98A5.5 5.5 0 0 1 17.2 5.6 4.95 4.95 0 0 1 17 15.5H7z" /><path d="M8.5 18.5l-1 2.5M12.5 18.5l-1 2.5M16.5 18.5l-1 2.5" /></>, p);
export const IconSnow = (p: P) => base(<><path d="M7 15a4.5 4.5 0 0 1-.4-8.98A5.5 5.5 0 0 1 17.2 5.1 4.95 4.95 0 0 1 17 15H7z" /><path d="M8.5 18.5h.01M12 20.5h.01M15.5 18.5h.01" strokeWidth="2.6" /></>, p);
export const IconStorm = (p: P) => base(<><path d="M7 15a4.5 4.5 0 0 1-.4-8.98A5.5 5.5 0 0 1 17.2 5.1 4.95 4.95 0 0 1 17 15" /><path d="M12.5 12l-2.5 4.2h3.2L11.5 21" /></>, p);
export const IconFog = (p: P) => base(<path d="M5 9h14M3 13h18M6 17h12" />, p);

export function weatherIcon(code: number): (p: P) => ReactNode {
  if (code === 0 || code === 1) return IconSun;
  if (code === 2) return IconPartly;
  if (code === 3) return IconCloud;
  if (code === 45 || code === 48) return IconFog;
  if (code >= 51 && code <= 67) return IconRain;
  if (code >= 71 && code <= 77) return IconSnow;
  if (code >= 80 && code <= 82) return IconRain;
  if (code >= 85 && code <= 86) return IconSnow;
  if (code >= 95) return IconStorm;
  return IconCloud;
}

// --- Genel ---
export const IconSearch = (p: P) => base(<><circle cx="11" cy="11" r="6.5" /><path d="M20 20l-4.2-4.2" /></>, p);
export const IconClose = (p: P) => base(<path d="M6 6l12 12M18 6L6 18" />, p);

export const IconChevronUp = (p: P) => base(<path d="M6 14.5l6-6 6 6" />, p);
export const IconDesktop = (p: P) => base(<><rect x="3" y="4.5" width="18" height="12" rx="2.2" /><path d="M8.5 20h7M12 16.5V20" /></>, p);
