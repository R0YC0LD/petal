// Hızlı arama ve başlatıcı (Alt+Space). Üst çubuk yüzeyinin içinde açılır; ayrı pencere yoktur.
// Sonuçlar: açık pencereler, Başlat menüsü uygulamaları ve basit hesaplamalar. Tam klavye kontrolü.
import { useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { IconSearch } from "./design/icons";

interface StartApp {
  name: string;
  id: string;
}
interface WinInfo {
  hwnd: number;
  title: string;
  exe: string;
}

type Result =
  | { kind: "calc"; label: string; sub: string }
  | { kind: "win"; label: string; sub: string; hwnd: number }
  | { kind: "app"; label: string; sub: string; id: string };

export const ROW_H = 46;
export const MAX_ROWS = 7;
export const SEARCH_BASE_H = 70;

let appCache: StartApp[] | null = null;
const iconCache = new Map<string, string | null>();

/** Yalnızca sayı, işlem işareti ve parantezden oluşan ifadeyi güvenle hesaplar (eval kullanmaz). */
export function calc(src: string): number | null {
  const s = src.replace(/,/g, ".").replace(/\s+/g, "");
  if (!/^[0-9+\-*/().%^]+$/.test(s) || !/[+\-*/%^]/.test(s.replace(/^-/, ""))) return null;
  let i = 0;
  const peek = () => s[i];
  function num(): number {
    if (peek() === "(") {
      i++;
      const v = expr();
      if (peek() !== ")") throw new Error("parantez");
      i++;
      return v;
    }
    if (peek() === "-") {
      i++;
      return -pow();
    }
    const m = /^[0-9]*\.?[0-9]+/.exec(s.slice(i));
    if (!m) throw new Error("sayı");
    i += m[0].length;
    return parseFloat(m[0]);
  }
  function pow(): number {
    const b = num();
    if (peek() === "^") {
      i++;
      return Math.pow(b, pow());
    }
    return b;
  }
  function term(): number {
    let v = pow();
    while (peek() === "*" || peek() === "/" || peek() === "%") {
      const op = s[i++];
      const r = pow();
      v = op === "*" ? v * r : op === "/" ? v / r : v % r;
    }
    return v;
  }
  function expr(): number {
    let v = term();
    while (peek() === "+" || peek() === "-") {
      const op = s[i++];
      const r = term();
      v = op === "+" ? v + r : v - r;
    }
    return v;
  }
  try {
    const v = expr();
    return i === s.length && Number.isFinite(v) ? v : null;
  } catch {
    return null;
  }
}

function Row({ r, active, onPick, onHover }: { r: Result; active: boolean; onPick: () => void; onHover: () => void }) {
  const [src, setSrc] = useState<string | null>(null);
  useEffect(() => {
    if (r.kind !== "app") return;
    if (iconCache.has(r.id)) {
      setSrc(iconCache.get(r.id) ?? null);
      return;
    }
    let alive = true;
    invoke<string | null>("get_icon", { kind: "appid", target: r.id })
      .then((v) => {
        iconCache.set(r.id, v);
        if (alive) setSrc(v);
      })
      .catch(() => {});
    return () => {
      alive = false;
    };
  }, [r.kind === "app" ? r.id : ""]);
  return (
    <div className={`sr-row ${active ? "on" : ""}`} onMouseMove={onHover} onClick={onPick}>
      <div className="sr-ico">
        {src ? <img src={src} alt="" draggable={false} /> : <span>{r.kind === "calc" ? "=" : r.label.slice(0, 1).toUpperCase()}</span>}
      </div>
      <div className="sr-text">
        <div className="sr-label">{r.label}</div>
        <div className="sr-sub">{r.sub}</div>
      </div>
      {active && <span className="sr-hint">↵</span>}
    </div>
  );
}

export default function Search({ onClose, onHeight }: { onClose: () => void; onHeight: (h: number) => void }) {
  const [q, setQ] = useState("");
  const [apps, setApps] = useState<StartApp[]>(appCache ?? []);
  const [wins, setWins] = useState<WinInfo[]>([]);
  const [sel, setSel] = useState(0);
  const input = useRef<HTMLInputElement>(null);

  useEffect(() => {
    input.current?.focus();
    if (!appCache) {
      invoke<StartApp[]>("list_start_apps")
        .then((a) => {
          appCache = a;
          setApps(a);
        })
        .catch(() => {});
    }
    invoke<WinInfo[]>("list_windows").then(setWins).catch(() => {});
  }, []);

  const results = useMemo<Result[]>(() => {
    const t = q.trim();
    const out: Result[] = [];
    if (!t) {
      // Boş arama: açık pencereler.
      return wins.slice(0, MAX_ROWS).map((w) => ({ kind: "win", label: w.title, sub: "Açık pencere", hwnd: w.hwnd }) as Result);
    }
    const c = calc(t);
    if (c !== null) out.push({ kind: "calc", label: String(+c.toFixed(10)), sub: `${t} =` });
    const low = t.toLocaleLowerCase("tr");
    for (const w of wins) {
      if (w.title.toLocaleLowerCase("tr").includes(low) || w.exe.includes(low)) out.push({ kind: "win", label: w.title, sub: "Açık pencere", hwnd: w.hwnd });
    }
    const scored = apps
      .map((a) => {
        const n = a.name.toLocaleLowerCase("tr");
        const idx = n.indexOf(low);
        return { a, score: idx === 0 ? 0 : idx > 0 ? 1 + idx / 100 : n.split(/\s+/).some((w) => w.startsWith(low)) ? 2 : 99 };
      })
      .filter((x) => x.score < 99)
      .sort((x, y) => x.score - y.score || x.a.name.localeCompare(y.a.name, "tr"));
    for (const { a } of scored) out.push({ kind: "app", label: a.name, sub: "Uygulama", id: a.id });
    return out.slice(0, MAX_ROWS);
  }, [q, apps, wins]);

  useEffect(() => setSel(0), [q]);
  useEffect(() => {
    onHeight(SEARCH_BASE_H + results.length * ROW_H + (results.length ? 12 : 0));
  }, [results.length]);

  const pick = (r: Result | undefined) => {
    if (!r) return;
    if (r.kind === "win") invoke("activate_window", { hwnd: r.hwnd });
    else if (r.kind === "app") invoke("launch_app", { kind: "appid", target: r.id });
    else navigator.clipboard?.writeText(r.label).catch(() => {});
    onClose();
  };

  return (
    <div
      className="search"
      onKeyDown={(e) => {
        if (e.key === "Escape") onClose();
        else if (e.key === "ArrowDown") {
          e.preventDefault();
          setSel((s) => Math.min(results.length - 1, s + 1));
        } else if (e.key === "ArrowUp") {
          e.preventDefault();
          setSel((s) => Math.max(0, s - 1));
        } else if (e.key === "Enter") pick(results[sel]);
      }}
    >
      <div className="sr-input">
        <IconSearch size={20} />
        <input ref={input} value={q} onChange={(e) => setQ(e.target.value)} placeholder="Uygulama, pencere veya işlem ara" spellCheck={false} />
        <kbd>Esc</kbd>
      </div>
      {results.length > 0 && (
        <div className="sr-list">
          {results.map((r, i) => (
            <Row key={`${r.kind}-${r.label}-${i}`} r={r} active={i === sel} onPick={() => pick(r)} onHover={() => setSel(i)} />
          ))}
        </div>
      )}
      {q.trim() && results.length === 0 && <div className="sr-empty">Sonuç yok</div>}
    </div>
  );
}
