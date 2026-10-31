import { useEffect, useRef, useState } from "react";
import { animate, motion, useMotionValue, useSpring, useTransform, type MotionValue } from "motion/react";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { listen } from "@tauri-apps/api/event";
import { DockApp, Settings, springOf, useSettings } from "./settings";
import { IconChevronUp, IconDesktop } from "./design/icons";

const iconCache = new Map<string, string>();
const HIDDEN_STRIP = 8;
const CARD_W = 176;
const PREVIEW_H = 190;
const TRAY_COLS = 5;
const TILE_W = 72;
const TRAY_W = TRAY_COLS * TILE_W + 28;
const CTX_H = 168;
// Dock'ta gösterilmeyen sistem/yardımcı süreçler.
const HIDDEN_EXES = new Set(["textinputhost", "searchhost", "startmenuexperiencehost", "shellexperiencehost", "systemsettings", "lockapp"]);
const matchesHidden = (exe: string) => HIDDEN_EXES.has(exe);

interface TrayApp {
  name: string;
  path: string;
  exe: string;
}

function TrayTile({ app, menuExe, setMenuExe, done }: { app: TrayApp; menuExe: string | null; setMenuExe: (e: string | null) => void; done: () => void }) {
  const src = useIcon({ id: app.exe, name: app.name, kind: "path", target: app.path });
  const open = () => {
    invoke("tray_open", { path: app.path, exe: app.exe });
    done();
  };
  return (
    <div
      className="ttile"
      onClick={() => (menuExe === app.exe ? setMenuExe(null) : open())}
      onContextMenu={(e) => {
        e.preventDefault();
        setMenuExe(menuExe === app.exe ? null : app.exe);
      }}
      title={`${app.name} — sağ tık: seçenekler`}
    >
      <div className="ticon">{src ? <img src={src} alt="" draggable={false} /> : <span>{app.name.slice(0, 1).toUpperCase()}</span>}</div>
      {menuExe === app.exe ? (
        <div className="tmenu" onClick={(e) => e.stopPropagation()}>
          <button onClick={open}>Aç</button>
          <button
            className="warn"
            onClick={() => {
              invoke("tray_close", { exe: app.exe });
              setMenuExe(null);
            }}
          >
            Kapat
          </button>
        </div>
      ) : (
        <div className="tname">{app.name}</div>
      )}
    </div>
  );
}

interface WinInfo {
  hwnd: number;
  title: string;
  exe: string;
  path: string;
  minimized: boolean;
  active: boolean;
}

interface Preview {
  hwnd: number;
  title: string;
  img: string | null;
  minimized: boolean;
}

function useIcon(app: DockApp) {
  const key = `${app.kind}:${app.target}`;
  const [src, setSrc] = useState<string | undefined>(iconCache.get(key));
  useEffect(() => {
    if (iconCache.get(key)) {
      setSrc(iconCache.get(key));
      return;
    }
    let alive = true;
    const attempt = (n: number) =>
      invoke<string | null>("get_icon", { kind: app.kind, target: app.target })
        .catch(() => null)
        .then((v) => {
          if (!alive) return;
          if (v) {
            iconCache.set(key, v);
            setSrc(v);
          } else if (n < 4) {
            setTimeout(() => attempt(n + 1), 1500);
          }
        });
    attempt(0);
    return () => {
      alive = false;
    };
  }, [key]);
  return src;
}

function stemOf(app: DockApp) {
  if (app.kind === "path") {
    const f = app.target.split("\\").pop() ?? "";
    return f.replace(/\.[^.]+$/, "").toLowerCase();
  }
  return app.name.toLowerCase();
}

interface IconProps {
  app: DockApp;
  mouseX: MotionValue<number>;
  settings: Settings;
  windows: WinInfo[];
  running: boolean;
  onHover: (app: DockApp, el: HTMLElement | null) => void;
  onMenu: (app: DockApp, el: HTMLElement | null) => void;
  drag?: DragApi;
}

interface DragState {
  id: string;
  dx: number;
  from: number;
  to: number;
  slotW: number;
}
interface DragApi {
  index: number;
  state: DragState | null;
  settle: { id: string; x: number; n: number } | null;
  start: (id: string, index: number) => void;
  move: (dx: number) => void;
  end: () => void;
}

function DockIcon({ app, mouseX, settings, windows, running, onHover, onMenu, drag }: IconProps) {
  const ref = useRef<HTMLDivElement>(null);
  const src = useIcon(app);
  const [bounce, setBounce] = useState(0);
  const size = settings.dockIconSize;

  const distance = useTransform(mouseX, (x) => {
    const b = ref.current?.getBoundingClientRect();
    return b ? x - (b.left + b.width / 2) : 9999;
  });
  const target = useTransform(distance, [-size * 2.4, 0, size * 2.4], [1, settings.dockMagnify, 1]);
  const scale = useSpring(target, { stiffness: 420, damping: 32, mass: 0.6 });

  // Sürükleyerek sıralama: imleç 6 px'den fazla oynarsa tıklama yerine sürükleme başlar.
  const x = useMotionValue(0);
  const press = useRef<{ x0: number; on: boolean } | null>(null);
  const swallow = useRef(false);
  const st = drag?.state ?? null;
  const dragging = !!st && st.id === app.id;
  const myIndex = drag?.index ?? -1;
  const offset = !st || dragging ? 0 : st.from < st.to ? (myIndex > st.from && myIndex <= st.to ? -st.slotW : 0) : st.from > st.to ? (myIndex < st.from && myIndex >= st.to ? st.slotW : 0) : 0;
  const settle = drag?.settle;
  useEffect(() => {
    if (dragging) x.set(st!.dx);
    else if (st) animate(x, offset, { type: "spring", stiffness: 520, damping: 38 });
  }, [dragging, st?.dx, offset, !!st]);
  useEffect(() => {
    if (!settle) return;
    x.set(settle.id === app.id ? settle.x : 0);
    animate(x, 0, { type: "spring", stiffness: 520, damping: 38 });
  }, [settle?.n]);

  const click = () => {
    if (swallow.current) {
      swallow.current = false;
      return;
    }
    if (windows.length > 0) {
      const first = windows[0];
      // Zaten öndeki tek pencereye tekrar tıklamak onu küçültür; birden çok pencere varsa sıradakine geçer.
      if (windows.length === 1 && first.active && !first.minimized) {
        invoke("minimize_window", { hwnd: first.hwnd });
      } else {
        const next = windows.length > 1 ? (windows.find((w) => !w.active) ?? first) : first;
        invoke("activate_window", { hwnd: next.hwnd });
      }
      return;
    }
    setBounce((n) => n + 1);
    invoke("launch_app", { kind: app.kind, target: app.target });
  };

  return (
    <motion.div
      ref={ref}
      className={`dock-slot ${dragging ? "dragging" : ""}`}
      style={{ width: size, height: size, x, zIndex: dragging ? 5 : undefined }}
      onPointerDown={(e) => {
        if (!drag || e.button !== 0) return;
        press.current = { x0: e.clientX, on: false };
        (e.currentTarget as HTMLElement).setPointerCapture(e.pointerId);
      }}
      onPointerMove={(e) => {
        const p = press.current;
        if (!p || !drag) return;
        const dx = e.clientX - p.x0;
        if (!p.on) {
          if (Math.abs(dx) < 6) return;
          p.on = true;
          swallow.current = true;
          drag.start(app.id, drag.index);
        }
        drag.move(dx);
      }}
      onPointerUp={() => {
        const p = press.current;
        press.current = null;
        if (p?.on) {
          drag?.end();
          setTimeout(() => (swallow.current = false), 0);
        }
      }}
      onPointerCancel={() => {
        if (press.current?.on) drag?.end();
        press.current = null;
      }}
      onMouseEnter={() => onHover(app, ref.current)}
      onContextMenu={(e) => {
        e.preventDefault();
        onMenu(app, ref.current);
      }}
    >
      <motion.div
        key={bounce}
        className="dock-icon"
        style={{ scale, width: size, height: size, originY: 1 }}
        animate={bounce ? { y: [0, -size * 0.45, 0, -size * 0.16, 0] } : undefined}
        transition={{ duration: 0.7, ease: "easeOut" }}
        onClick={click}
        whileTap={{ filter: "brightness(0.8)" }}
      >
        {src ? <img src={src} alt="" draggable={false} /> : <div className="fallback">{app.name.slice(0, 1).toUpperCase()}</div>}
        {settings.dockLabels && windows.length === 0 && <span className="tip">{app.name}</span>}
      </motion.div>
      <span className={`run-dot ${running ? "on" : ""} ${windows.some((w) => w.active) ? "active" : ""}`} />
      {windows.length > 1 && <span className="win-count">{windows.length}</span>}
    </motion.div>
  );
}

export default function Dock() {
  const [dragState, setDragState] = useState<DragState | null>(null);
  const [settle, setSettle] = useState<{ id: string; x: number; n: number } | null>(null);
  const dragRef = useRef<DragState | null>(null);
  const [settings, up] = useSettings();
  const mouseX = useMotionValue(-9999);
  const [hidden, setHidden] = useState(false);
  const [running, setRunning] = useState<string[]>([]);
  const [wins, setWins] = useState<WinInfo[]>([]);
  const [previews, setPreviews] = useState<Preview[]>([]);
  const [previewLeft, setPreviewLeft] = useState(0);
  const [trayOpen, setTrayOpen] = useState(false);
  const [trayApps, setTrayApps] = useState<TrayApp[]>([]);
  const [menuExe, setMenuExe] = useState<string | null>(null);
  const [ctx, setCtx] = useState<{ app: DockApp; pinned: boolean; left: number } | null>(null);
  const hideTimer = useRef(0);
  const shrinkTimer = useRef(0);
  const hoverTimer = useRef(0);
  const closeTimer = useRef(0);
  const stageRef = useRef<HTMLDivElement>(null);
  const apps = settings.dockApps;
  const size = settings.dockIconSize;
  const spring = springOf(settings);

  // Sabit olmayan ama açık penceresi olan uygulamalar da dock'ta gösterilir.
  const pinnedStems = new Set(apps.map(stemOf));
  const extras: DockApp[] = [];
  if (settings.dockShowRunning) {
    for (const w of wins) {
      if (pinnedStems.has(w.exe) || extras.some((e) => stemOf(e) === w.exe)) continue;
      if (matchesHidden(w.exe)) continue;
      const pretty = w.exe === "applicationframehost" ? w.title : w.exe.charAt(0).toUpperCase() + w.exe.slice(1);
      extras.push({ id: `run-${w.exe}`, name: pretty, kind: "path", target: w.path });
    }
  }
  const allCount = apps.length + extras.length;

  // Uygulamalar + ayraç + iki yardımcı düğme (arka plan uygulamaları, masaüstünü göster).
  const barWidth = Math.max(120, Math.round(allCount * (size + 10) + (extras.length ? 14 : 0) + 28 + 2 * 46 + 14 + size * (settings.dockMagnify - 1) * 1.6));
  const barHeight = Math.round(size * settings.dockMagnify + 52);
  const cardsWidth = previews.length * (CARD_W + 8) + 16;
  const trayRows = Math.min(3, Math.max(1, Math.ceil(trayApps.length / TRAY_COLS)));
  const trayH = 56 + trayRows * 84 + 14;
  const width = Math.max(barWidth, previews.length ? cardsWidth : 0, trayOpen ? TRAY_W + 16 : 0);
  const height = barHeight + (previews.length ? PREVIEW_H : trayOpen ? trayH : ctx ? CTX_H : 0);
  const autoHide = settings.dockAutoHide;
  const shown = !autoHide || !hidden;

  useEffect(() => {
    invoke("set_visible", { label: "dock", visible: settings.dockEnabled && apps.length > 0 });
  }, [settings.dockEnabled, apps.length]);

  useEffect(() => {
    if (shown) invoke("resize_dock", { width: width * settings.scale, height: height * settings.scale });
  }, [width, height, shown, settings.scale]);

  useEffect(() => {
    const t = setTimeout(
      () => invoke("resize_dock", { width: width * settings.scale, height: (hidden && autoHide ? HIDDEN_STRIP : height) * settings.scale }),
      700,
    );
    return () => clearTimeout(t);
  }, [settings.hideTaskbar]);

  useEffect(() => {
    if (!autoHide) {
      setHidden(false);
      return;
    }
    const t = setTimeout(() => setHidden(true), 2500);
    return () => clearTimeout(t);
  }, [autoHide]);

  useEffect(() => {
    if (!settings.dockEnabled) return;
    let alive = true;
    // Art arda gelen olaylar tek sorguya indirilir (aynı anda en fazla bir istek).
    let busy = false;
    let again = false;
    const winTick = () => {
      if (busy) {
        again = true;
        return;
      }
      busy = true;
      invoke<WinInfo[]>("list_windows")
        .then((r) => alive && setWins((cur) => (JSON.stringify(cur) === JSON.stringify(r) ? cur : r)))
        .catch(() => {})
        .finally(() => {
          busy = false;
          if (again) {
            again = false;
            setTimeout(winTick, 250);
          }
        });
    };
    const procTick = () =>
      invoke<string[]>("get_running")
        .then((r) => alive && setRunning(r))
        .catch(() => {});
    winTick();
    procTick();
    // Pencere değişimleri Windows olay kancasından gelir (anında); yoklama yalnızca güvenlik ağıdır.
    const un = listen("windows-changed", winTick);
    const quiet = hidden && autoHide;
    const w = setInterval(winTick, quiet ? 15000 : 8000);
    const p = setInterval(procTick, quiet ? 30000 : 15000);
    return () => {
      alive = false;
      un.then((f) => f());
      clearInterval(w);
      clearInterval(p);
    };
  }, [settings.dockEnabled, hidden, autoHide]);

  useEffect(() => {
    const un = getCurrentWebview().onDragDropEvent((e) => {
      if (e.payload.type !== "drop") return;
      for (const p of e.payload.paths) invoke("add_dock_path", { path: p });
    });
    return () => {
      un.then((f) => f());
    };
  }, []);

  const winsOf = (app: DockApp) => {
    const stem = stemOf(app);
    return wins.filter((w) => w.exe === stem);
  };

  const closePreview = () => {
    clearTimeout(hoverTimer.current);
    clearTimeout(closeTimer.current);
    closeTimer.current = window.setTimeout(() => {
      setPreviews([]);
      setTrayOpen(false);
      setMenuExe(null);
      setCtx(null);
    }, Math.max(250, settings.dockCloseDelay));
  };

  const onHover = (app: DockApp, el: HTMLElement | null) => {
    clearTimeout(closeTimer.current);
    clearTimeout(hoverTimer.current);
    const list = winsOf(app).slice(0, 4);
    if (list.length > 0) setTrayOpen(false);
    if (list.length === 0 || !el) {
      setPreviews([]);
      return;
    }
    hoverTimer.current = window.setTimeout(async () => {
      const stage = stageRef.current?.getBoundingClientRect();
      const r = el.getBoundingClientRect();
      const base: Preview[] = list.map((w) => ({ hwnd: w.hwnd, title: w.title, img: null, minimized: w.minimized }));
      const rowW = base.length * (CARD_W + 8) + 16;
      const stageW = Math.max(barWidth, rowW);
      const dx = r.left + r.width / 2 - ((stage?.left ?? 0) + (stage?.width ?? stageW) / 2);
      const center = stageW / 2 + dx;
      setPreviewLeft(Math.min(Math.max(center - rowW / 2, 4), Math.max(4, stageW - rowW - 4)));
      setPreviews(base);
      for (const w of list) {
        if (w.minimized) continue;
        const img = await invoke<string | null>("window_preview", { hwnd: w.hwnd }).catch(() => null);
        setPreviews((cur) => cur.map((p) => (p.hwnd === w.hwnd ? { ...p, img } : p)));
      }
    }, 260);
  };

  const toggleTray = () => {
    if (trayOpen) {
      setTrayOpen(false);
      setMenuExe(null);
      return;
    }
    setPreviews([]);
    setTrayOpen(true);
    invoke<TrayApp[]>("list_tray_apps").then(setTrayApps).catch(() => setTrayApps([]));
  };

  const openMenu = (app: DockApp, el: HTMLElement | null) => {
    clearTimeout(closeTimer.current);
    setPreviews([]);
    setTrayOpen(false);
    const stage = stageRef.current?.getBoundingClientRect();
    const r = el?.getBoundingClientRect();
    const stageW = Math.max(barWidth, 200);
    const center = r && stage ? r.left + r.width / 2 - (stage.left + stage.width / 2) + stageW / 2 : stageW / 2;
    const pinned = apps.some((a) => stemOf(a) === stemOf(app));
    setCtx({ app, pinned, left: Math.min(Math.max(center - 90, 4), Math.max(4, stageW - 184)) });
  };

  const dragApi = (index: number): DragApi => ({
    index,
    state: dragState,
    settle,
    start: (id, from) => {
      const slots = stageRef.current?.querySelectorAll<HTMLElement>(".dock-bar > .dock-slot");
      let slotW = size + 10;
      if (slots && slots.length > 1) slotW = slots[1].getBoundingClientRect().left - slots[0].getBoundingClientRect().left || slotW;
      clearTimeout(closeTimer.current);
      setPreviews([]);
      setCtx(null);
      mouseX.set(-9999);
      const d = { id, dx: 0, from, to: from, slotW };
      dragRef.current = d;
      setDragState(d);
    },
    move: (dx) => {
      const d = dragRef.current;
      if (!d) return;
      const to = Math.min(apps.length - 1, Math.max(0, d.from + Math.round(dx / d.slotW)));
      const n = { ...d, dx, to };
      dragRef.current = n;
      setDragState(n);
    },
    end: () => {
      const d = dragRef.current;
      dragRef.current = null;
      if (!d) return;
      if (d.to !== d.from) {
        const next = apps.slice();
        const [m] = next.splice(d.from, 1);
        next.splice(d.to, 0, m);
        up({ dockApps: next });
      }
      setSettle({ id: d.id, x: d.dx - (d.to - d.from) * d.slotW, n: Date.now() });
      setDragState(null);
    },
  });

  const togglePin = (app: DockApp, pinned: boolean) => {
    if (pinned) up({ dockApps: apps.filter((a) => stemOf(a) !== stemOf(app)) });
    else up({ dockApps: [...apps, { ...app, id: `${stemOf(app)}-${Date.now()}` }] });
    setCtx(null);
  };

  const revealTimer = useRef(0);
  const enter = (e: React.MouseEvent) => {
    clearTimeout(hideTimer.current);
    clearTimeout(shrinkTimer.current);
    clearTimeout(closeTimer.current);
    clearTimeout(revealTimer.current);
    if (!hidden) return;
    // Fare düğmesi basılıyken (kaydırma çubuğu vb.) veya hızlıca geçerken açılma.
    if (e.buttons !== 0) return;
    setHidden(false);
  };
  const leave = () => {
    clearTimeout(revealTimer.current);
    mouseX.set(-9999);
    closePreview();
    if (!autoHide) return;
    clearTimeout(hideTimer.current);
    hideTimer.current = window.setTimeout(() => {
      setHidden(true);
      setPreviews([]);
      shrinkTimer.current = window.setTimeout(() => invoke("resize_dock", { width: barWidth * settings.scale, height: HIDDEN_STRIP * settings.scale }), 450);
    }, settings.dockCloseDelay);
  };

  return (
    <div className="dock-stage" ref={stageRef} onMouseEnter={enter} onMouseLeave={leave}>
      {ctx && (
        <motion.div
          className="dock-menu"
          style={{ left: ctx.left, bottom: barHeight - 6 }}
          initial={{ opacity: 0, y: 8, scale: 0.98 }}
          animate={{ opacity: 1, y: 0, scale: 1 }}
          transition={{ duration: 0.14 }}
          onMouseEnter={() => clearTimeout(closeTimer.current)}
        >
          <div className="dm-title">{ctx.app.name}</div>
          <button
            onClick={() => {
              invoke("launch_app", { kind: ctx.app.kind, target: ctx.app.target });
              setCtx(null);
            }}
          >
            {winsOf(ctx.app).length ? "Yeni pencere" : "Aç"}
          </button>
          <button onClick={() => togglePin(ctx.app, ctx.pinned)}>{ctx.pinned ? "Dock'tan kaldır" : "Dock'ta tut"}</button>
          {winsOf(ctx.app).length > 0 && (
            <button
              className="danger"
              onClick={() => {
                winsOf(ctx.app).forEach((w) => invoke("tray_close", { exe: w.exe }));
                setCtx(null);
              }}
            >
              {winsOf(ctx.app).length > 1 ? "Tüm pencereleri kapat" : "Pencereyi kapat"}
            </button>
          )}
        </motion.div>
      )}
      {trayOpen && (
        <motion.div
          className="tray-panel"
          style={{ bottom: barHeight - 6, width: TRAY_W }}
          initial={{ opacity: 0, y: 10, scale: 0.98 }}
          animate={{ opacity: 1, y: 0, scale: 1 }}
          transition={{ duration: 0.16 }}
          onMouseEnter={() => clearTimeout(closeTimer.current)}
        >
          <div className="tray-head">
            <span>Arka plan uygulamaları</span>
            <button className="tray-refresh" onClick={() => invoke<TrayApp[]>("list_tray_apps").then(setTrayApps)} aria-label="Yenile">
              Yenile
            </button>
          </div>
          {trayApps.length === 0 ? (
            <div className="tray-empty">Bildirim alanında simgesi olan, çalışan uygulama yok.</div>
          ) : (
            <div className="tray-grid">
              {trayApps.map((a) => (
                <TrayTile key={a.path} app={a} menuExe={menuExe} setMenuExe={setMenuExe} done={() => setTrayOpen(false)} />
              ))}
            </div>
          )}
        </motion.div>
      )}
      {previews.length > 0 && (
        <motion.div
          className="previews"
          style={{ left: previewLeft, bottom: barHeight - 6 }}
          initial={{ opacity: 0, y: 8 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.14 }}
          onMouseEnter={() => clearTimeout(closeTimer.current)}
        >
          {previews.map((p) => (
            <button key={p.hwnd} className="pcard" onClick={() => invoke("activate_window", { hwnd: p.hwnd })}>
              <div className="pimg">
                {p.img ? <img src={p.img} alt="" draggable={false} /> : <span>{p.minimized ? "Simge durumunda" : "…"}</span>}
              </div>
              <div className="ptitle">{p.title}</div>
            </button>
          ))}
        </motion.div>
      )}
      <motion.div
        className="dock-bar"
        initial={{ y: 90, opacity: 0 }}
        animate={shown ? { y: 0, opacity: 1 } : { y: size + 40, opacity: 0 }}
        transition={spring}
        onMouseMove={(e) => !dragRef.current && mouseX.set(e.clientX)}
        onMouseLeave={() => mouseX.set(-9999)}
        style={{ height: size + 18 }}
      >
        {apps.map((a, i) => {
          const own = winsOf(a);
          return (
            <DockIcon
              key={a.id}
              app={a}
              mouseX={mouseX}
              settings={settings}
              windows={own}
              running={own.length > 0 || running.includes(stemOf(a))}
              onHover={onHover}
              onMenu={openMenu}
              drag={dragApi(i)}
            />
          );
        })}
        {extras.length > 0 && <div className="dock-sep" />}
        {extras.map((a) => {
          const own = winsOf(a);
          return <DockIcon key={a.id} app={a} mouseX={mouseX} settings={settings} windows={own} running onHover={onHover} onMenu={openMenu} />;
        })}
        <div className="dock-sep" />
        <button className={`dock-util ${trayOpen ? "on" : ""}`} onClick={toggleTray} aria-label="Arka plan uygulamaları" title="Arka plan uygulamaları">
          <IconChevronUp size={20} />
        </button>
        <button className="dock-util" onClick={() => invoke("show_desktop")} aria-label="Masaüstünü göster" title="Masaüstünü göster / geri al">
          <IconDesktop size={20} />
        </button>
      </motion.div>
      {autoHide && !shown && <div className="dock-hint" />}
    </div>
  );
}
