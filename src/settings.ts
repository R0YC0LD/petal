import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export interface DockApp {
  id: string;
  name: string;
  kind: "path" | "appid";
  target: string;
}

export interface Settings {
  introEnabled: boolean;
  notchEnabled: boolean;
  dockEnabled: boolean;
  clock24h: boolean;
  showSeconds: boolean;
  showCpu: boolean;
  showRam: boolean;
  theme: "system" | "light" | "dark";
  accent: string;
  accentCustom: string;
  surfaceOpacity: number;
  stiffness: number;
  damping: number;
  dockIconSize: number;
  dockMagnify: number;
  dockLabels: boolean;
  dockAutoHide: boolean;
  dockCloseDelay: number;
  dockShowRunning: boolean;
  notchMode: "fixed" | "peek";
  weatherEnabled: boolean;
  celsius: boolean;
  weatherCity: string;
  weatherLat: number | null;
  weatherLon: number | null;
  showGpu: boolean;
  hudEnabled: boolean;
  hideInFullscreen: boolean;
  hoverDelay: number;
  closeDelay: number;
  statsInterval: number;
  topBarStats: boolean;
  topBarIdle: number;
  topBarPinned: boolean;
  layout: string[];
  metricsOrder: string[];
  hideTaskbar: boolean;
  reducedMotion: boolean;
  musicEnabled: boolean;
  visualizer: boolean;
  scale: number;
  dockApps: DockApp[];
  launchAtLogin: boolean;
}

export const DEFAULTS: Settings = {
  introEnabled: true,
  notchEnabled: true,
  dockEnabled: true,
  clock24h: true,
  showSeconds: true,
  showCpu: true,
  showRam: true,
  theme: "system",
  accent: "blue",
  accentCustom: "#5b8def",
  surfaceOpacity: 0.97,
  stiffness: 340,
  damping: 34,
  dockIconSize: 52,
  dockMagnify: 1.55,
  dockLabels: true,
  dockAutoHide: true,
  dockCloseDelay: 1200,
  dockShowRunning: true,
  notchMode: "fixed",
  weatherEnabled: false,
  celsius: true,
  weatherCity: "",
  weatherLat: null,
  weatherLon: null,
  showGpu: true,
  hudEnabled: true,
  hideInFullscreen: true,
  hoverDelay: 0,
  closeDelay: 300,
  statsInterval: 1000,
  topBarStats: true,
  topBarIdle: 7,
  topBarPinned: false,
  layout: ["top", "metrics", "music"],
  metricsOrder: ["cpu", "gpu", "ram"],
  hideTaskbar: false,
  reducedMotion: false,
  musicEnabled: true,
  visualizer: true,
  scale: 1,
  launchAtLogin: false,
  dockApps: [
    {
      id: "brave",
      name: "Brave",
      kind: "path",
      target: String.raw`C:\Program Files\BraveSoftware\Brave-Browser\Application\brave.exe`,
    },
    { id: "explorer", name: "Dosya Gezgini", kind: "path", target: String.raw`C:\Windows\explorer.exe` },
    {
      id: "spotify",
      name: "Spotify",
      kind: "appid",
      target: "SpotifyAB.SpotifyMusic_zpdnekdrzrea0!Spotify",
    },
  ],
};

export const ACCENTS: Record<string, string> = {
  blue: "#5b8def",
  green: "#6fae8b",
  orange: "#e39a5a",
  red: "#e0696a",
  graphite: "#8a8a93",
};

const darkQuery = typeof matchMedia === "function" ? matchMedia("(prefers-color-scheme: dark)") : null;

export function resolveTheme(s: Settings): "light" | "dark" {
  if (s.theme === "system") return darkQuery?.matches === false ? "light" : "dark";
  return s.theme;
}

/** Kayıtlı sıralamada eksik veya fazla öğe varsa varsayılanla tamamlar. */
export function fixOrder(saved: string[] | undefined, all: string[]): string[] {
  const kept = (saved ?? []).filter((x) => all.includes(x));
  return [...kept, ...all.filter((x) => !kept.includes(x))];
}

export function applyTheme(s: Settings) {
  const root = document.documentElement;
  root.dataset.theme = resolveTheme(s);
  const r = root.style;
  r.setProperty("--accent", s.accent === "custom" ? s.accentCustom : (ACCENTS[s.accent] ?? ACCENTS.blue));
  // Okunabilirlik için yüzeyler en az %92 opak tutulur (arkadaki pencere içeriği görünmez).
  r.setProperty("--surface-a", String(Math.max(0.92, s.surfaceOpacity)));
  if (document.body.dataset.view !== "settings") r.setProperty("zoom", String(s.scale));
}

export function useSettings(): [Settings, (patch: Partial<Settings>) => void] {
  const [s, setS] = useState<Settings>(DEFAULTS);

  useEffect(() => {
    let alive = true;
    invoke<Partial<Settings>>("load_settings").then((v) => {
      if (!alive) return;
      const merged = { ...DEFAULTS, ...v };
      setS(merged);
      applyTheme(merged);
    });
    const un = listen<Partial<Settings>>("settings-changed", (e) => {
      const merged = { ...DEFAULTS, ...e.payload };
      setS(merged);
      applyTheme(merged);
    });
    const onSystem = () => setS((cur) => {
      applyTheme(cur);
      return { ...cur };
    });
    darkQuery?.addEventListener("change", onSystem);
    return () => {
      alive = false;
      darkQuery?.removeEventListener("change", onSystem);
      un.then((f) => f());
    };
  }, []);

  const update = (patch: Partial<Settings>) => {
    const next = { ...s, ...patch };
    setS(next);
    applyTheme(next);
    invoke("save_settings", { value: next });
  };
  return [s, update];
}

/** Merkezi hareket motoru: tüm bileşenler yay ayarlarını buradan alır. */
export interface MotionSet {
  hover: Transition;
  small: Transition;
  panel: Transition;
  large: Transition;
}

type Transition = { type: "spring"; stiffness: number; damping: number; mass: number } | { type: "tween"; duration: number; ease: "easeOut" };

const spring = (stiffness: number, ratio: number): Transition => ({
  type: "spring",
  stiffness,
  damping: ratio * 2 * Math.sqrt(stiffness * 0.9),
  mass: 0.9,
});

export function motionOf(s: Settings): MotionSet {
  if (s.reducedMotion) {
    const t = (d: number): Transition => ({ type: "tween", duration: d, ease: "easeOut" });
    return { hover: t(0.08), small: t(0.1), panel: t(0.12), large: t(0.16) };
  }
  const k = s.stiffness / 340;
  const ratio = s.damping / 34;
  return {
    hover: spring(520 * k, 1 * ratio),
    small: spring(440 * k, 1 * ratio),
    panel: spring(340 * k, 1 * ratio),
    large: spring(260 * k, 1 * ratio),
  };
}

export const springOf = (s: Settings) => motionOf(s).panel;
