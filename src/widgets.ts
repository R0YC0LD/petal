import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export type MediaAction = "toggle" | "next" | "prev" | "shuffle" | "repeat";

export interface MediaInfo {
  title: string;
  artist: string;
  album: string;
  playing: boolean;
  position: number;
  duration: number;
  cover: string | null;
  can_prev: boolean;
  can_next: boolean;
  source: string;
  shuffle: boolean | null;
  repeat: "none" | "track" | "list" | null;
}

/** Windows medya oturumunu yoklar. Oturum yoksa `null` döner, sahte veri üretilmez. */
export function useMedia(enabled: boolean, fastWhenOpen: boolean) {
  const [m, setM] = useState<MediaInfo | null>(null);
  const [pos, setPos] = useState(0);
  const stamp = useRef(0);

  useEffect(() => {
    if (!enabled) {
      setM(null);
      return;
    }
    let alive = true;
    const tick = () =>
      invoke<MediaInfo | null>("get_media")
        .then((v) => {
          if (!alive) return;
          setM(v);
          if (v) {
            setPos(v.position);
            stamp.current = performance.now();
          }
        })
        .catch(() => alive && setM(null));
    tick();
    const id = setInterval(tick, fastWhenOpen ? 1000 : 3000);
    return () => {
      alive = false;
      clearInterval(id);
    };
  }, [enabled, fastWhenOpen]);

  useEffect(() => {
    if (!m?.playing || !fastWhenOpen) return;
    const base = m.position;
    const t0 = stamp.current;
    const id = setInterval(() => setPos(Math.min(m.duration || Infinity, base + (performance.now() - t0) / 1000)), 250);
    return () => clearInterval(id);
  }, [m, fastWhenOpen]);

  const control = (action: MediaAction) => invoke("media_control", { action }).catch(() => {});
  const seek = (seconds: number) => {
    setPos(seconds);
    stamp.current = performance.now();
    return invoke("media_seek", { seconds }).catch(() => {});
  };
  return { media: m, position: pos, control, seek };
}

/** Çıkış cihazı tepe seviyesi (0-1). Yalnızca `active` iken yoklar. */
export function useAudioPeak(active: boolean) {
  const [peak, setPeak] = useState(0);
  useEffect(() => {
    if (!active) {
      setPeak(0);
      return;
    }
    let alive = true;
    const id = setInterval(() => {
      invoke<number | null>("get_audio_peak")
        .then((v) => alive && setPeak(v ?? 0))
        .catch(() => {});
    }, 140);
    return () => {
      alive = false;
      clearInterval(id);
    };
  }, [active]);
  return peak;
}

export function fmtClock(ms: number, hundredths = false) {
  const total = Math.max(0, ms);
  const m = Math.floor(total / 60000);
  const s = Math.floor((total % 60000) / 1000);
  const base = `${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
  return hundredths ? `${base}.${String(Math.floor((total % 1000) / 10)).padStart(2, "0")}` : base;
}

export interface Weather {
  temp: number;
  text: string;
  wind: number;
  city: string;
  code: number;
  /** Son başarılı güncelleme (ms). Eski veri güncel gibi gösterilmez. */
  updatedAt: number;
}

const WMO: Record<number, string> = {
  0: "Açık", 1: "Çoğunlukla açık", 2: "Parçalı bulutlu", 3: "Kapalı",
  45: "Sisli", 48: "Kırağılı sis", 51: "Hafif çiseleme", 53: "Çiseleme", 55: "Yoğun çiseleme",
  61: "Hafif yağmur", 63: "Yağmurlu", 65: "Şiddetli yağmur", 71: "Hafif kar", 73: "Karlı", 75: "Yoğun kar",
  80: "Sağanak", 81: "Sağanak", 82: "Şiddetli sağanak", 95: "Gök gürültülü fırtına", 96: "Dolu", 99: "Dolu",
};

const CACHE_KEY = "petal-weather";

export interface GeoHit {
  name: string;
  admin: string;
  country: string;
  lat: number;
  lon: number;
}

/** Şehir adından koordinat bulur (Open-Meteo geocoding). */
export async function searchCity(q: string): Promise<GeoHit[]> {
  const url = `https://geocoding-api.open-meteo.com/v1/search?name=${encodeURIComponent(q)}&count=5&language=tr&format=json`;
  const j = await (await fetch(url)).json();
  return (j.results ?? []).map((r: any) => ({
    name: r.name,
    admin: r.admin1 ?? "",
    country: r.country ?? "",
    lat: r.latitude,
    lon: r.longitude,
  }));
}

export function useWeather(enabled: boolean, celsius: boolean, city: string, lat: number | null, lon: number | null) {
  const [w, setW] = useState<Weather | null>(() => {
    try {
      return JSON.parse(localStorage.getItem(CACHE_KEY) ?? "null");
    } catch {
      return null;
    }
  });

  useEffect(() => {
    if (!enabled) return;
    let alive = true;
    const load = async () => {
      try {
        let geo: { success: boolean; latitude: number; longitude: number; city?: string };
        if (lat !== null && lon !== null) {
          geo = { success: true, latitude: lat, longitude: lon, city };
        } else {
          try {
            geo = await (await fetch("https://ipwho.is/")).json();
          } catch {
            geo = { success: false, latitude: 0, longitude: 0 };
          }
          if (!geo.success) {
            // Yedek servis
            const g2 = await (await fetch("https://ipapi.co/json/")).json();
            geo = { success: !!g2.latitude, latitude: g2.latitude, longitude: g2.longitude, city: g2.city };
          }
        }
        if (!geo.success) return;
        const unit = celsius ? "celsius" : "fahrenheit";
        const url = `https://api.open-meteo.com/v1/forecast?latitude=${geo.latitude}&longitude=${geo.longitude}&current=temperature_2m,weather_code,wind_speed_10m&temperature_unit=${unit}&wind_speed_unit=kmh`;
        const j = await (await fetch(url)).json();
        const next: Weather = {
          temp: Math.round(j.current.temperature_2m),
          text: WMO[j.current.weather_code] ?? "—",
          wind: Math.round(j.current.wind_speed_10m),
          city: geo.city ?? "",
          code: j.current.weather_code,
          updatedAt: Date.now(),
        };
        if (!alive) return;
        setW(next);
        try {
          localStorage.setItem(CACHE_KEY, JSON.stringify(next));
        } catch {
          /* önbellek yazılamazsa sorun değil */
        }
      } catch {
        /* ağ yoksa son değer kalır */
      }
    };
    load();
    const id = setInterval(load, 20 * 60000);
    return () => {
      alive = false;
      clearInterval(id);
    };
  }, [enabled, celsius, city, lat, lon]);

  return w;
}

export interface SysStats {
  cpu: number;
  ram: number;
  /** null: GPU sayacı yok veya henüz ölçülemedi (N/A). */
  gpu: number | null;
}

/** Rust örnekleyicisinin `stats` olaylarını dinler; yoklama yapmaz. */
export function useSysStats(active = true): SysStats {
  const [s, setS] = useState<SysStats>({ cpu: 0, ram: 0, gpu: null });
  useEffect(() => {
    if (!active) return;
    let alive = true;
    invoke<SysStats>("get_stats").then((v) => alive && setS(v)).catch(() => {});
    const un = listen<SysStats>("stats", (e) => setS(e.payload));
    return () => {
      alive = false;
      un.then((f) => f());
    };
  }, [active]);
  return s;
}
