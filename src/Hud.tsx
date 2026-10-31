// Kompakt dikey kenar kapsülü: üstte yüzde, ortada dikey kaydırıcı, altta simge.
// Sol kenar: ses. Sağ kenar: parlaklık.
import { useEffect, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useSettings } from "./settings";
import { IconBrightness, IconVolume } from "./design/icons";
import "./hud.css";

type Side = "left" | "right";

export default function Hud() {
  useSettings();
  // Pencere etiketi tarafı belirler: hud-left = ses, hud-right = parlaklık.
  const side: Side = getCurrentWindow().label === "hud-right" ? "right" : "left";
  const [value, setValue] = useState(0.5);
  const [muted, setMuted] = useState(false);
  const [supported, setSupported] = useState(true);
  const [open, setOpen] = useState(false);
  const track = useRef<HTMLDivElement>(null);
  const dragging = useRef(false);
  const isVolume = side === "left";

  const refresh = async (sd: Side) => {
    if (dragging.current) return;
    if (sd === "left") {
      const v = await invoke<{ level: number; muted: boolean } | null>("get_volume").catch(() => null);
      setSupported(!!v);
      if (v && !dragging.current) {
        setValue(v.level);
        setMuted(v.muted);
      }
    } else {
      const b = await invoke<number | null>("get_brightness").catch(() => null);
      setSupported(b !== null);
      if (b !== null && !dragging.current) setValue(b / 100);
    }
  };

  useEffect(() => {
    const a = listen("hud-open", () => {
      setOpen(true);
      refresh(side);
    });
    const b = listen("hud-close", () => setOpen(false));
    return () => {
      a.then((f) => f());
      b.then((f) => f());
    };
  }, []);

  // Yalnızca görünürken, dış değişiklikleri (klavye tuşları vb.) yakalamak için hafif yoklama.
  useEffect(() => {
    if (!open) return;
    const id = setInterval(() => refresh(side), 600);
    return () => clearInterval(id);
  }, [open, side]);

  const apply = (v: number) => {
    // İyimser arayüz: önce göster, sonra yerel komutu gönder. Arka uç yalnızca son değeri uygular.
    setValue(v);
    if (isVolume) {
      if (v > 0) setMuted(false);
      invoke("set_volume", { level: v });
    } else {
      invoke("set_brightness", { percent: Math.round(v * 100) });
    }
  };

  const fromPointer = (clientY: number) => {
    const r = track.current!.getBoundingClientRect();
    apply(Math.min(1, Math.max(0, 1 - (clientY - r.top) / r.height)));
  };

  const shown = isVolume && muted ? 0 : value;

  return (
    <div className="hud-stage">
      <div className={`vpill ${side} ${open ? "open" : ""}`}>
        <span className="vpill-pct">{supported ? Math.round(shown * 100) : "—"}</span>
        <div
          className={`vpill-track ${supported ? "" : "off"}`}
          ref={track}
          onPointerDown={(e) => {
            if (!supported) return;
            dragging.current = true;
            e.currentTarget.setPointerCapture(e.pointerId);
            fromPointer(e.clientY);
          }}
          onPointerMove={(e) => dragging.current && fromPointer(e.clientY)}
          onPointerUp={() => (dragging.current = false)}
          onPointerCancel={() => (dragging.current = false)}
          onWheel={(e) => supported && apply(Math.min(1, Math.max(0, value + (e.deltaY < 0 ? 0.04 : -0.04))))}
        >
          <div className="vpill-fill" style={{ height: `${shown * 100}%` }} />
        </div>
        <button
          className="vpill-icon"
          disabled={!isVolume}
          onClick={() => {
            const m = !muted;
            setMuted(m);
            invoke("set_mute", { muted: m });
          }}
          aria-label={isVolume ? (muted ? "Sesi aç" : "Sessize al") : "Parlaklık"}
        >
          {isVolume ? <IconVolume size={19} level={shown} muted={muted} /> : <IconBrightness size={19} />}
        </button>
      </div>
    </div>
  );
}
