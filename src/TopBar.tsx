// Üst çubuk: ekranın üst kenarına bitişik tek yüzey.
// Genişlemiş durum: SOL (saat, tarih, sistem) | MERKEZ (medya) | SAĞ (hava).
// Küçülmüş durum: yalnızca saat. İkisi aynı yüzeydir; saat öğesi `layoutId` ile paylaşılır.
import { useEffect, useRef, useState } from "react";
import { motion } from "motion/react";
import { invoke } from "@tauri-apps/api/core";
import { Settings, springOf, useSettings } from "./settings";
import { fmtClock, MediaAction, MediaInfo, useAudioPeak, useMedia, useSysStats, useWeather, Weather } from "./widgets";
import { IconNext, IconNote, IconPause, IconPlay, IconPrev, IconRepeat, IconShuffle, weatherIcon } from "./design/icons";
import Logo from "./design/Logo";
import Search from "./Search";
import { listen } from "@tauri-apps/api/event";
import "./topbar.css";

type Mode = "boot" | "intro" | "expanded" | "compact" | "search";

const H_MEDIA = 96;
const H_PLAIN = 66;
const COMPACT = { w: 132, h: 38 };
const INTRO = { w: 380, h: 200 };
const BOX_H = 420;
const SEARCH_W = 640;

const mmss = (sec: number) => fmtClock(sec * 1000);

function barWidth() {
  return Math.min(1040, Math.max(560, window.screen.width - 24));
}

/** Saat yalnızca dakika değişince yeniden çizilir (saniye göstergesi bu çubukta yok). */
function useNow() {
  const [now, setNow] = useState(new Date());
  useEffect(() => {
    let id = 0;
    const tick = () => {
      const d = new Date();
      setNow(d);
      id = window.setTimeout(tick, (60 - d.getSeconds()) * 1000 - d.getMilliseconds() + 20);
    };
    id = window.setTimeout(tick, 0);
    return () => clearTimeout(id);
  }, []);
  return now;
}

function Bars({ peak, playing }: { peak: number; playing: boolean }) {
  const [t, setT] = useState(0);
  useEffect(() => {
    if (!playing) return;
    const id = setInterval(() => setT((v) => v + 1), 170);
    return () => clearInterval(id);
  }, [playing]);
  const level = playing ? Math.min(1, peak * 1.6) : 0;
  return (
    <div className="bars" aria-hidden>
      {Array.from({ length: 5 }, (_, i) => {
        const wobble = 0.45 + 0.55 * Math.abs(Math.sin(t * 0.9 + i * 1.7));
        return <i key={i} style={{ height: `${Math.max(14, level * wobble * 100)}%` }} />;
      })}
    </div>
  );
}

function Stat({ label, value }: { label: string; value: number | null }) {
  return (
    <span className="stat">
      {label} <b>{value === null ? "N/A" : `${Math.round(value)}%`}</b>
    </span>
  );
}

function Clock({ s, now, big }: { s: Settings; now: Date; big?: boolean }) {
  const time = now.toLocaleTimeString("tr-TR", { hour: "2-digit", minute: "2-digit", hour12: !s.clock24h });
  return <div className={`tb-clock ${big ? "big" : ""}`}>{time}</div>;
}

function Left({ s, now, stats }: { s: Settings; now: Date; stats: ReturnType<typeof useSysStats> }) {
  const date = now.toLocaleDateString("tr-TR", { weekday: "short", day: "numeric", month: "long" });
  return (
    <div className="zone left">
      <Clock s={s} now={now} />
      <div className="tb-meta">
        <div className="tb-date">{date}</div>
        {s.topBarStats && (
          <div className="tb-stats">
            {s.showCpu && <Stat label="CPU" value={stats.cpu} />}
            {s.showGpu && <Stat label="GPU" value={stats.gpu} />}
            {s.showRam && <Stat label="RAM" value={stats.ram} />}
          </div>
        )}
      </div>
    </div>
  );
}

function Center({
  s,
  media,
  position,
  control,
  seek,
  peak,
}: {
  s: Settings;
  media: MediaInfo | null;
  position: number;
  control: (a: MediaAction) => void;
  seek: (sec: number) => void;
  peak: number;
}) {
  const track = useRef<HTMLDivElement>(null);
  const [drag, setDrag] = useState<number | null>(null);
  if (!s.musicEnabled || !media) {
    return (
      <div className="zone center nomedia">
        <IconNote size={16} />
        <span>Çalan medya yok</span>
      </div>
    );
  }
  const shown = drag ?? position;
  const pct = media.duration > 0 ? Math.min(100, (shown / media.duration) * 100) : 0;
  const remaining = media.duration > 0 ? Math.max(0, media.duration - shown) : 0;
  const fromX = (x: number) => {
    const r = track.current!.getBoundingClientRect();
    return Math.min(1, Math.max(0, (x - r.left) / r.width)) * media.duration;
  };
  const repeatOn = media.repeat !== null && media.repeat !== "none";
  return (
    <div className="zone center">
      <div className="m-row">
        <div className="cover" key={media.cover ?? media.title}>
          {media.cover ? <img src={media.cover} alt="" draggable={false} /> : <IconNote size={20} />}
        </div>
        <div className="song">
          <div className="song-title">{media.title}</div>
          <div className="song-artist">{media.artist || media.album}</div>
        </div>
        {s.visualizer && <Bars peak={peak} playing={media.playing} />}
        <div className="ctl-row">
          <button className={`ctl small ${media.shuffle ? "on" : ""}`} disabled={media.shuffle === null} onClick={() => control("shuffle")} aria-label="Karışık çal">
            <IconShuffle size={14} />
          </button>
          <button className="ctl" disabled={!media.can_prev} onClick={() => control("prev")} aria-label="Önceki">
            <IconPrev size={16} />
          </button>
          <button className="ctl play" onClick={() => control("toggle")} aria-label={media.playing ? "Duraklat" : "Oynat"}>
            {media.playing ? <IconPause size={17} /> : <IconPlay size={17} />}
          </button>
          <button className="ctl" disabled={!media.can_next} onClick={() => control("next")} aria-label="Sonraki">
            <IconNext size={16} />
          </button>
          <button className={`ctl small ${repeatOn ? "on" : ""}`} disabled={media.repeat === null} onClick={() => control("repeat")} aria-label="Tekrarla">
            <IconRepeat size={14} />
          </button>
        </div>
      </div>
      <div className="prog">
        <span className="t-l">{mmss(shown)}</span>
        <div
          className={`track ${media.duration > 0 ? "" : "off"}`}
          ref={track}
          onPointerDown={(e) => {
            if (media.duration <= 0) return;
            e.currentTarget.setPointerCapture(e.pointerId);
            setDrag(fromX(e.clientX));
          }}
          onPointerMove={(e) => drag !== null && setDrag(fromX(e.clientX))}
          onPointerUp={(e) => {
            if (drag !== null) seek(fromX(e.clientX));
            setDrag(null);
          }}
          onPointerCancel={() => setDrag(null)}
        >
          <div className="fill" style={{ width: `${pct}%` }} />
        </div>
        <span className="t-r">{media.duration > 0 ? `-${mmss(remaining)}` : "--:--"}</span>
      </div>
    </div>
  );
}

function Right({ s, w, enable }: { s: Settings; w: Weather | null; enable: () => void }) {
  if (!s.weatherEnabled) {
    return (
      <div className="zone right">
        <button className="wx-btn" onClick={enable}>
          Hava durumunu aç
        </button>
      </div>
    );
  }
  if (!w) {
    return (
      <div className="zone right">
        {s.weatherLat === null ? (
          <button className="wx-btn" onClick={() => invoke("open_settings")}>
            Şehir seç
          </button>
        ) : (
          <span className="wx-sub">Hava yükleniyor</span>
        )}
      </div>
    );
  }
  const Icon = weatherIcon(w.code);
  const ageMin = Math.round((Date.now() - w.updatedAt) / 60000);
  const stale = ageMin > 40;
  return (
    <div className={`zone right ${stale ? "stale" : ""}`}>
      <div className="wx-icon">{Icon({ size: 26 })}</div>
      <div className="wx-temp">{w.temp}°</div>
      <div className="wx-col">
        <div className="wx-text">{w.text}</div>
        <div className="wx-sub">
          {w.city}
          {stale && ` · ${ageMin >= 60 ? `${Math.floor(ageMin / 60)} sa` : `${ageMin} dk`} önce`}
        </div>
      </div>
    </div>
  );
}

export default function TopBar() {
  const [s, up] = useSettings();
  const [mode, setMode] = useState<Mode>("boot");
  const started = useRef(false);
  const hovering = useRef(false);
  const dragging = useRef(false);
  const idleTimer = useRef(0);
  const shrinkTimer = useRef(0);
  const now = useNow();
  const spring = springOf(s);
  const stats = useSysStats(mode === "expanded");
  const weather = useWeather(s.weatherEnabled, s.celsius, s.weatherCity, s.weatherLat, s.weatherLon);
  const { media, position, control, seek } = useMedia(s.musicEnabled, mode === "expanded");
  const playing = !!media?.playing;
  const peak = useAudioPeak(s.visualizer && playing && mode === "expanded");

  const hasMedia = s.musicEnabled && !!media;
  const expandedH = hasMedia ? H_MEDIA : H_PLAIN;
  const boxW = barWidth();

  // Pencere boyutu duruma göre değişir; yüzey ise sabit boyutlu bir kutudur ve kırpma
  // (clip-path) ile büyür/küçülür. Böylece animasyon sırasında yeniden yerleşim hesaplanmaz.
  const [searchH, setSearchH] = useState(70);
  const winDims = (m: Mode) => {
    if (m === "search") return { w: SEARCH_W, h: searchH };
    if (m === "intro") return INTRO;
    if (m === "compact") return COMPACT;
    return { w: boxW, h: expandedH };
  };
  const clipFor = (m: Mode) => {
    const d = m === "boot" ? COMPACT : winDims(m);
    const r = m === "intro" ? 34 : m === "expanded" || m === "search" ? 28 : 19;
    const side = Math.max(0, (boxW - d.w) / 2);
    const bottom = Math.max(0, BOX_H - d.h);
    return `inset(0px ${side}px ${bottom}px ${side}px round 0px 0px ${r}px ${r}px)`;
  };

  const resize = (m: Mode) => {
    const d = winDims(m);
    return invoke("resize_notch", { width: d.w * s.scale, height: d.h * s.scale });
  };
  const latest = useRef({ resize, mode });
  latest.current = { resize, mode };

  const armIdle = () => {
    clearTimeout(idleTimer.current);
    if (s.topBarPinned || s.topBarIdle <= 0) return;
    idleTimer.current = window.setTimeout(() => {
      if (hovering.current || dragging.current) return armIdle();
      collapse();
    }, s.topBarIdle * 1000);
  };

  const collapse = () => {
    if (latest.current.mode !== "expanded") return;
    setMode("compact");
    // Yüzey küçülürken pencere büyük kalır; animasyon bitince pencere de küçülür.
    clearTimeout(shrinkTimer.current);
    shrinkTimer.current = window.setTimeout(() => {
      if (latest.current.mode === "compact") latest.current.resize("compact");
    }, 480);
  };

  // Gecikmesiz açılış: pencere büyütme komutu ve yüzey animasyonu aynı anda başlar.
  const expand = () => {
    if (latest.current.mode !== "compact") return;
    clearTimeout(shrinkTimer.current);
    latest.current.resize("expanded");
    setMode("expanded");
  };

  useEffect(() => {
    if (started.current) return;
    started.current = true;
    (async () => {
      await new Promise((r) => setTimeout(r, 200));
      if (s.introEnabled) {
        await latest.current.resize("intro");
        await invoke("set_visible", { label: "notch", visible: true });
        setMode("intro");
        await new Promise((r) => setTimeout(r, 1900));
      }
      await latest.current.resize("expanded");
      await invoke("set_visible", { label: "notch", visible: true });
      setMode("expanded");
    })();
  }, []);

  useEffect(() => {
    if (mode === "expanded") armIdle();
    return () => clearTimeout(idleTimer.current);
  }, [mode, s.topBarIdle, s.topBarPinned]);

  useEffect(() => {
    if (mode === "boot") return;
    invoke("set_visible", { label: "notch", visible: s.notchEnabled || mode === "intro" });
  }, [s.notchEnabled, mode]);

  useEffect(() => {
    if (mode === "expanded" || mode === "compact") resize(mode);
  }, [s.scale, hasMedia]);

  // Hızlı arama: Alt+Space açar/kapatır; önceki duruma (geniş/küçük) geri döner.
  const prevMode = useRef<Mode>("expanded");
  const closeSearch = () => {
    if (latest.current.mode !== "search") return;
    const back = prevMode.current === "compact" ? "compact" : "expanded";
    setMode(back);
    clearTimeout(shrinkTimer.current);
    shrinkTimer.current = window.setTimeout(() => latest.current.resize(back), 420);
  };
  const toggleSearch = () => {
    const m = latest.current.mode;
    if (m === "search") return closeSearch();
    if (m !== "expanded" && m !== "compact") return;
    prevMode.current = m;
    clearTimeout(shrinkTimer.current);
    latest.current.resize("search");
    invoke("focus_notch");
    setMode("search");
  };
  const toggleRef = useRef(toggleSearch);
  toggleRef.current = toggleSearch;
  useEffect(() => {
    const un = listen("search-toggle", () => toggleRef.current());
    return () => {
      un.then((f) => f());
    };
  }, []);
  useEffect(() => {
    if (mode === "search") resize("search");
  }, [searchH]);
  // Pencere odağı başka yere giderse arama kapanır.
  useEffect(() => {
    if (mode !== "search") return;
    const onBlur = () => closeSearch();
    const t = setTimeout(() => window.addEventListener("blur", onBlur), 400);
    return () => {
      clearTimeout(t);
      window.removeEventListener("blur", onBlur);
    };
  }, [mode]);

  const onEnter = (e: React.MouseEvent) => {
    hovering.current = true;
    clearTimeout(idleTimer.current);
    if (mode === "compact" && e.buttons === 0) expand();
  };
  const onLeave = () => {
    hovering.current = false;
    if (mode === "expanded") armIdle();
  };

  const showGrid = mode === "expanded";

  return (
    <div className="stage" onPointerDown={() => (dragging.current = true)} onPointerUp={() => (dragging.current = false)}>
      <motion.div
        className="tb"
        style={{ width: boxW, height: BOX_H }}
        onMouseEnter={onEnter}
        onMouseLeave={onLeave}
        onMouseMove={() => mode === "expanded" && armIdle()}
        onClick={() => mode === "compact" && expand()}
        onMouseDown={(e) => e.button === 1 && e.preventDefault()}
        onContextMenu={(e) => {
          e.preventDefault();
          invoke("open_settings");
        }}
        initial={false}
        animate={{ clipPath: clipFor(mode), opacity: mode === "boot" ? 0 : 1 }}
        transition={{ clipPath: spring, opacity: { duration: 0.15 } }}
      >
        {mode === "intro" && (
          <div className="intro" style={{ width: INTRO.w, height: INTRO.h }}>
            <Logo size={92} animate />
            <motion.div className="intro-name" initial={{ opacity: 0, y: 6 }} animate={{ opacity: 1, y: 0 }} transition={{ delay: 0.6, duration: 0.4 }}>
              Petal
            </motion.div>
            <motion.div className="intro-by" initial={{ opacity: 0 }} animate={{ opacity: 1 }} transition={{ delay: 0.95, duration: 0.4 }}>
              R0YC0LD · Onur Teryakioğlu
            </motion.div>
          </div>
        )}
        {mode === "search" && <Search onClose={closeSearch} onHeight={setSearchH} />}
        <div className={`compact ${mode === "compact" ? "on" : ""}`}>
          <Clock s={s} now={now} big />
        </div>
        <div className={`grid ${showGrid ? "on" : ""}`} style={{ width: boxW, height: expandedH }}>
          <Left s={s} now={now} stats={stats} />
          <Center s={s} media={media} position={position} control={control} seek={seek} peak={peak} />
          <Right s={s} w={weather} enable={() => up({ weatherEnabled: true })} />
        </div>
      </motion.div>
    </div>
  );
}
