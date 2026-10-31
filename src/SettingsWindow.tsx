import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { searchCity, GeoHit } from "./widgets";
import { fixOrder, ACCENTS, DEFAULTS, DockApp, Settings, useSettings } from "./settings";

type Tab = "genel" | "notch" | "dock" | "gorunum" | "animasyon" | "hakkinda";

const TABS: [Tab, string][] = [
  ["genel", "Genel"],
  ["notch", "Notch"],
  ["dock", "Dock"],
  ["gorunum", "Görünüm"],
  ["animasyon", "Animasyon"],
  ["hakkinda", "Hakkında"],
];

const ACCENT_NAMES: Record<string, string> = { blue: "Mavi", green: "Yeşil", orange: "Turuncu", red: "Kırmızı", graphite: "Grafit" };

function Toggle({ on, set }: { on: boolean; set: (v: boolean) => void }) {
  return (
    <button className={`switch ${on ? "on" : ""}`} onClick={() => set(!on)} aria-pressed={on}>
      <i />
    </button>
  );
}

function Row({ title, desc, children }: { title: string; desc?: string; children: React.ReactNode }) {
  return (
    <div className="row">
      <div>
        <div className="t">{title}</div>
        {desc && <div className="d">{desc}</div>}
      </div>
      {children}
    </div>
  );
}

function Slide(p: {
  value: number;
  min: number;
  max: number;
  step: number;
  set: (v: number) => void;
  fmt?: (v: number) => string;
}) {
  return (
    <div className="slider">
      <input
        type="range"
        min={p.min}
        max={p.max}
        step={p.step}
        value={p.value}
        onChange={(e) => p.set(Number(e.target.value))}
      />
      <b>{p.fmt ? p.fmt(p.value) : p.value}</b>
    </div>
  );
}

function AppIcon({ app }: { app: DockApp }) {
  const [src, setSrc] = useState<string | null>(null);
  useEffect(() => {
    invoke<string | null>("get_icon", { kind: app.kind, target: app.target }).then(setSrc).catch(() => {});
  }, [app.target]);
  return src ? <img className="ico" src={src} alt="" /> : <div className="ico fallback" style={{ fontSize: 14 }}>{app.name[0]}</div>;
}

const SECTION_NAMES: Record<string, string> = { top: "Saat ve hava durumu", metrics: "CPU, GPU, RAM", music: "Müzik" };
const METRIC_NAMES: Record<string, string> = { cpu: "CPU", gpu: "GPU", ram: "RAM" };

function OrderList({ items, names, set }: { items: string[]; names: Record<string, string>; set: (v: string[]) => void }) {
  const move = (i: number, d: number) => {
    const a = [...items];
    const j = i + d;
    if (j < 0 || j >= a.length) return;
    [a[i], a[j]] = [a[j], a[i]];
    set(a);
  };
  return (
    <div className="order">
      {items.map((k, i) => (
        <div className="order-row" key={k}>
          <span>{names[k]}</span>
          <button className="btn" disabled={i === 0} onClick={() => move(i, -1)}>↑</button>
          <button className="btn" disabled={i === items.length - 1} onClick={() => move(i, 1)}>↓</button>
        </div>
      ))}
    </div>
  );
}

function HotkeyPill() {
  const [k, setK] = useState<string | null | undefined>(undefined);
  useEffect(() => {
    invoke<string | null>("search_hotkey").then(setK).catch(() => setK(null));
  }, []);
  return <span className="kbd-pill">{k === undefined ? "…" : k ? k.replace(/\+/g, " + ") : "Kısayol atanamadı"}</span>;
}

function CityPicker({ s, up }: { s: Settings; up: (p: Partial<Settings>) => void }) {
  const [q, setQ] = useState("");
  const [hits, setHits] = useState<GeoHit[] | null>(null);
  const [err, setErr] = useState("");
  const find = () => {
    setErr("");
    searchCity(q.trim()).then(setHits).catch(() => setErr("Şehir aranamadı, internet bağlantısını kontrol et"));
  };
  return (
    <>
      <Row title="Şehir" desc={s.weatherLat !== null ? `Seçili: ${s.weatherCity}` : "Seçilmedi, konum IP adresinden bulunur"}>
        <div className="city">
          <input value={q} placeholder="Şehir ara" onChange={(e) => setQ(e.target.value)} onKeyDown={(e) => e.key === "Enter" && q.trim() && find()} />
          <button className="btn" onClick={find} disabled={!q.trim()}>Ara</button>
          {s.weatherLat !== null && (
            <button className="btn" onClick={() => up({ weatherCity: "", weatherLat: null, weatherLon: null })}>Temizle</button>
          )}
        </div>
      </Row>
      {err && <div className="sub" style={{ color: "#e0696a" }}>{err}</div>}
      {hits && (
        <div className="picker" style={{ marginBottom: 8 }}>
          {hits.length === 0 && <div className="sub" style={{ margin: 8 }}>Sonuç yok.</div>}
          {hits.map((h, i) => (
            <button key={i} onClick={() => { up({ weatherCity: h.name, weatherLat: h.lat, weatherLon: h.lon }); setHits(null); setQ(""); }}>
              {h.name}{h.admin && `, ${h.admin}`}{h.country && ` · ${h.country}`}
            </button>
          ))}
        </div>
      )}
    </>
  );
}

function DockTab({ s, up }: { s: Settings; up: (p: Partial<Settings>) => void }) {
  const [list, setList] = useState<{ name: string; id: string }[] | null>(null);
  const [filter, setFilter] = useState("");

  const move = (i: number, d: number) => {
    const a = [...s.dockApps];
    const j = i + d;
    if (j < 0 || j >= a.length) return;
    [a[i], a[j]] = [a[j], a[i]];
    up({ dockApps: a });
  };
  const remove = (i: number) => up({ dockApps: s.dockApps.filter((_, k) => k !== i) });
  const add = (name: string, id: string) => {
    if (s.dockApps.some((a) => a.target === id)) return;
    up({ dockApps: [...s.dockApps, { id: `${name}-${Date.now()}`, name, kind: "appid", target: id }] });
  };

  return (
    <>
      <h2>Dock</h2>
      <div className="sub">Uygulamaları dock'a ekle, sırala. Bir .exe veya kısayolu dock'un üstüne sürükleyip de bırakabilirsin.</div>
      <Row title="Dock'u göster">
        <Toggle on={s.dockEnabled} set={(v) => up({ dockEnabled: v })} />
      </Row>
      <Row title="Simge boyutu">
        <Slide value={s.dockIconSize} min={36} max={80} step={2} set={(v) => up({ dockIconSize: v })} fmt={(v) => `${v}px`} />
      </Row>
      <Row title="Büyütme gücü" desc="Fare üstüne gelince simgenin ne kadar büyüyeceği">
        <Slide value={s.dockMagnify} min={1} max={2.2} step={0.05} set={(v) => up({ dockMagnify: v })} fmt={(v) => `${v.toFixed(2)}x`} />
      </Row>
      <Row title="Uygulama adlarını göster">
        <Toggle on={s.dockLabels} set={(v) => up({ dockLabels: v })} />
      </Row>
      <Row title="Açık uygulamaları göster" desc="Dock'ta sabit olmayan ama açık penceresi olan uygulamalar da görünür">
        <Toggle on={s.dockShowRunning} set={(v) => up({ dockShowRunning: v })} />
      </Row>
      <Row title="Otomatik gizle" desc="Kullanmadığında alt kenara iner, imleci aşağı götürünce geri gelir">
        <Toggle on={s.dockAutoHide} set={(v) => up({ dockAutoHide: v })} />
      </Row>
      <Row title="Dock kapanma süresi" desc="Fare dock'tan uzaklaşınca dock ve önizleme panelleri bu süre sonra kapanır">
        <Slide value={s.dockCloseDelay} min={0} max={5000} step={100} set={(v) => up({ dockCloseDelay: v })} fmt={(v) => (v === 0 ? "Anında" : `${(v / 1000).toFixed(1)} sn`)} />
      </Row>
      <div style={{ height: 12 }} />
      <div className="apps">
        {s.dockApps.map((a, i) => (
          <div className="app-row" key={a.id}>
            <AppIcon app={a} />
            <div className="name">{a.name}</div>
            <button className="btn" onClick={() => move(i, -1)}>↑</button>
            <button className="btn" onClick={() => move(i, 1)}>↓</button>
            <button className="btn danger" onClick={() => remove(i)}>Kaldır</button>
          </div>
        ))}
        {s.dockApps.length === 0 && <div className="sub">Dock boş.</div>}
      </div>
      {list === null ? (
        <button className="btn primary" onClick={() => invoke<typeof list>("list_start_apps").then(setList)}>
          Başlat menüsünden uygulama ekle
        </button>
      ) : (
        <>
          <input
            className="search"
            placeholder="Uygulama ara…"
            value={filter}
            onChange={(e) => setFilter(e.target.value)}
            style={{ width: "100%", padding: "9px 12px", borderRadius: 10, border: 0, marginBottom: 8, background: "rgba(255,255,255,0.07)", color: "#fff", font: "inherit" }}
          />
          <div className="picker">
            {list
              .filter((a) => a.name.toLowerCase().includes(filter.toLowerCase()))
              .map((a) => (
                <button key={a.id} onClick={() => add(a.name, a.id)}>
                  {a.name}
                </button>
              ))}
          </div>
        </>
      )}
    </>
  );
}

export default function SettingsWindow() {
  const [s, up] = useSettings();
  const [tab, setTab] = useState<Tab>("genel");

  return (
    <div className="settings">
      <nav className="side">
        <div className="brand">Petal</div>
        {TABS.map(([k, label]) => (
          <button key={k} className={`nav ${tab === k ? "on" : ""}`} onClick={() => setTab(k)}>
            {label}
          </button>
        ))}
      </nav>
      <main className="main">
        {tab === "genel" && (
          <>
            <h2>Genel</h2>
            <div className="sub">Petal'in temel davranışı.</div>
            <Row title="Windows ile başlat" desc="Bilgisayar açılınca Petal otomatik çalışsın">
              <Toggle on={s.launchAtLogin} set={(v) => up({ launchAtLogin: v })} />
            </Row>
            <Row title="Windows görev çubuğunu gizle" desc="Petal açıkken görev çubuğu gizlenir. Petal kapanınca geri gelir. Tray menüsünden de geri getirilebilir">
              <Toggle on={s.hideTaskbar} set={(v) => up({ hideTaskbar: v })} />
            </Row>
            <Row title="Açılış animasyonu" desc="Başlarken geliştirici imzasını göster">
              <Toggle on={s.introEnabled} set={(v) => up({ introEnabled: v })} />
            </Row>
            <Row title="Notch'u göster">
              <Toggle on={s.notchEnabled} set={(v) => up({ notchEnabled: v })} />
            </Row>
            <Row title="Dock'u göster">
              <Toggle on={s.dockEnabled} set={(v) => up({ dockEnabled: v })} />
            </Row>
          </>
        )}
        {tab === "notch" && (
          <>
            <h2>Notch</h2>
            <div className="sub">Üstteki ada. Üzerine gelince açılır, sağ tık ile ayarlar açılır.</div>
            <Row title="Üst çubuğu sabitle" desc="Açıkken kendiliğinden küçülüp yalnızca saate dönmez">
              <Toggle on={s.topBarPinned} set={(v) => up({ topBarPinned: v })} />
            </Row>
            <Row title="Otomatik küçülme süresi" desc="Etkileşim olmayınca üst çubuk yalnızca saate dönüşür. 0: hiç küçülmez">
              <Slide value={s.topBarIdle} min={0} max={30} step={1} set={(v) => up({ topBarIdle: v })} fmt={(v) => (v === 0 ? "Kapalı" : `${v} sn`)} />
            </Row>
            <Row title="Kapanma gecikmesi" desc="Fare uzaklaşınca kenar kapsülleri ve dock bu süre sonra kapanır. Açılışta gecikme yoktur">
              <Slide value={s.closeDelay} min={0} max={1500} step={50} set={(v) => up({ closeDelay: v })} fmt={(v) => `${v} ms`} />
            </Row>
            <Row title="Tam ekranda gizle" desc="Oyun, tam ekran video veya sunum sırasında üst çubuk ve dock gizlenir, kenar kapsülleri açılmaz">
              <Toggle on={s.hideInFullscreen} set={(v) => up({ hideInFullscreen: v })} />
            </Row>
            <Row title="Hızlı arama" desc="Uygulama, açık pencere ve basit hesaplama. Başka bir uygulama aynı kısayolu tutuyorsa sıradaki alternatif kullanılır">
              <HotkeyPill />
            </Row>
            <Row title="Kenar göstergeleri" desc="İmleci ekranın soluna götürünce ses, sağına götürünce parlaklık çubuğu açılır">
              <Toggle on={s.hudEnabled} set={(v) => up({ hudEnabled: v })} />
            </Row>
            <Row title="Hava durumu" desc="Konumu IP adresinden ipwho.is ile bulur, veriyi open-meteo.com'dan alır">
              <Toggle on={s.weatherEnabled} set={(v) => up({ weatherEnabled: v })} />
            </Row>
            {s.weatherEnabled && <CityPicker s={s} up={up} />}
            <Row title="Sıcaklık birimi">
              <div className="seg">
                {[true, false].map((c) => (
                  <button key={String(c)} className={s.celsius === c ? "on" : ""} onClick={() => up({ celsius: c })}>
                    {c ? "°C" : "°F"}
                  </button>
                ))}
              </div>
            </Row>
            <Row title="Müzik modülü" desc="Windows medya oturumundan (Spotify, tarayıcı vb.) çalan parçayı gösterir">
              <Toggle on={s.musicEnabled} set={(v) => up({ musicEnabled: v })} />
            </Row>
            <Row title="Müzik görselleştirici" desc="Çıkış cihazının anlık ses seviyesine göre hareket eder">
              <Toggle on={s.visualizer} set={(v) => up({ visualizer: v })} />
            </Row>
            <Row title="Panel düzeni" desc="Açılan panelde bölümlerin yukarıdan aşağıya sırası">
              <OrderList items={fixOrder(s.layout, ["top", "metrics", "music"])} names={SECTION_NAMES} set={(v) => up({ layout: v })} />
            </Row>
            <Row title="Sistem göstergeleri sırası" desc="CPU, GPU ve RAM'in soldan sağa sırası">
              <OrderList items={fixOrder(s.metricsOrder, ["cpu", "gpu", "ram"])} names={METRIC_NAMES} set={(v) => up({ metricsOrder: v })} />
            </Row>
            <Row title="24 saat biçimi">
              <Toggle on={s.clock24h} set={(v) => up({ clock24h: v })} />
            </Row>
            <Row title="Saniyeyi göster">
              <Toggle on={s.showSeconds} set={(v) => up({ showSeconds: v })} />
            </Row>
            <Row title="GPU göstergesi" desc="Windows performans sayacından, en yüklü ekran kartının 3D kullanımı">
              <Toggle on={s.showGpu} set={(v) => up({ showGpu: v })} />
            </Row>
            <Row title="CPU göstergesi">
              <Toggle on={s.showCpu} set={(v) => up({ showCpu: v })} />
            </Row>
            <Row title="RAM göstergesi">
              <Toggle on={s.showRam} set={(v) => up({ showRam: v })} />
            </Row>
          </>
        )}
        {tab === "dock" && <DockTab s={s} up={up} />}
        {tab === "gorunum" && (
          <>
            <h2>Görünüm</h2>
            <div className="sub">Renkler anında uygulanır.</div>
            <Row title="Tema" desc="Sistem seçeneği Windows'un açık/koyu ayarını izler">
              <div className="seg">
                {(["system", "light", "dark"] as const).map((t) => (
                  <button key={t} className={s.theme === t ? "on" : ""} onClick={() => up({ theme: t })}>
                    {t === "system" ? "Sistem" : t === "light" ? "Açık" : "Koyu"}
                  </button>
                ))}
              </div>
            </Row>
            <Row title="Vurgu rengi">
              <div className="swatches">
                {Object.entries(ACCENTS).map(([k, c]) => (
                  <button
                    key={k}
                    title={ACCENT_NAMES[k]}
                    className={`swatch ${s.accent === k ? "on" : ""}`}
                    style={{ background: c }}
                    onClick={() => up({ accent: k })}
                  />
                ))}
                <label className={`swatch custom ${s.accent === "custom" ? "on" : ""}`} title="Özel renk" style={{ background: s.accentCustom }}>
                  <input type="color" value={s.accentCustom} onChange={(e) => up({ accent: "custom", accentCustom: e.target.value })} />
                </label>
              </div>
            </Row>
            <Row title="Boyut" desc="Notch ve dock'u birlikte büyütür veya küçültür">
              <Slide value={s.scale} min={0.8} max={1.4} step={0.05} set={(v) => up({ scale: v })} fmt={(v) => `${Math.round(v * 100)}%`} />
            </Row>
            <Row title="Yüzey opaklığı" desc="En az %92: arkadaki pencere içeriği okunabilirliği bozmaz">
              <Slide value={s.surfaceOpacity} min={0.92} max={1} step={0.01} set={(v) => up({ surfaceOpacity: v })} fmt={(v) => `${Math.round(v * 100)}%`} />
            </Row>
          </>
        )}
        {tab === "animasyon" && (
          <>
            <h2>Animasyon</h2>
            <div className="sub">Yay animasyonlarının hissini ayarla.</div>
            <Row title="Sertlik" desc="Yüksek değer daha hızlı ve keskin hareket">
              <Slide value={s.stiffness} min={150} max={700} step={10} set={(v) => up({ stiffness: v })} />
            </Row>
            <Row title="Sönümleme" desc="Düşük değer daha esnek, zıplamalı hareket">
              <Slide value={s.damping} min={10} max={50} step={1} set={(v) => up({ damping: v })} />
            </Row>
            <Row title="Hareketi azalt" desc="Yay animasyonları yerine kısa, sade geçişler kullanır">
              <Toggle on={s.reducedMotion} set={(v) => up({ reducedMotion: v })} />
            </Row>
            <button className="btn" onClick={() => up({ stiffness: DEFAULTS.stiffness, damping: DEFAULTS.damping })}>
              Varsayılana dön
            </button>
          </>
        )}
        {tab === "hakkinda" && (
          <>
            <h2>Hakkında</h2>
            <div className="sub">Windows için notch ve dock.</div>
            <div className="about-card">
              <div className="big">Petal</div>
              <div className="sub" style={{ margin: "4px 0 0" }}>Sürüm 0.9.10</div>
              <div className="dev">R0YC0LD</div>
              <div className="real">Onur Teryakioğlu</div>
              <div className="sub" style={{ margin: "16px 0 0" }}>
                Notch ve dock fikri Nectar ve Bloom projelerinden esinlenmiştir. Petal'in kodu bağımsız yazılmıştır.
              </div>
            </div>
            <div style={{ height: 14 }} />
            <button className="btn danger" onClick={() => invoke("save_settings", { value: DEFAULTS })}>
              Tüm ayarları sıfırla
            </button>
          </>
        )}
      </main>
    </div>
  );
}
