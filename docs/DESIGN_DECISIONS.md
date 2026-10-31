# Tasarım Kararları

| # | Karar | Dayanak |
|---|---|---|
| D1 | Renkler verilen açık/koyu paletten, tek yerde CSS değişkenleri olarak tanımlanır. Vurgu: Soft Blue `#5b8def`, Muted Green `#6fae8b`, Warm Orange `#e39a5a`, Soft Red `#e0696a`, Graphite `#8a8a93`, özel. | Kullanıcı şartnamesi |
| D2 | Gradient, glow, neon yok. Vurgu rengi yalnızca etkin durum, ilerleme ve odak için. | Kullanıcı kuralı "AI estetiği yasak" |
| D3 | Yüzeyler şeffaf pencere üzerinde yarı saydam düz renk + 1 px kenarlık + yumuşak gölge. Sistem Mica/Acrylic uygulanmaz. | Microsoft materyal belgesi: Acrylic geçici yüzeyler için, Mica opak kalıcı zemin; yuvarlak şeffaf pencerede sistem efekti dikdörtgen çıkar. |
| D4 | Spacing ölçeği 4, 8, 12, 16, 20, 24, 32 px (`--sp-*`). Köşe: küçük 10, kart 16, panel 24, kapsül yarım yükseklik. | Şartname 3.3–3.4 |
| D5 | Hareket: yay tabanlı, kesilebilir, girdiyle aynı karede başlar. Roller: `hover` (≈120 ms eşdeğeri), `small`, `panel`, `large`. Aşırı sıçrama yok (sönümleme ≈ kritik). Reduced Motion: 80–160 ms tween. | Apple hareket ilkeleri (genel bilgi, bkz. RESEARCH §1) |
| D6 | Üst çubuk: CSS Grid `1fr auto 1fr`. Orta kolon medya, ekranın geometrik merkezinde; yan kolonlar eşit esner, içerik orta kolona taşamaz. | Şartname 5 |
| D7 | Kenar kontrolleri yatay kapsül: `[ikon] [yatay kaydırıcı] [%]`, yaklaşık 188×48, tam yuvarlak. Dikey kart yok. | Şartname 7 |
| D8 | Hover niyeti: açılma 80–120 ms (kullanıcı ayarlı), kapanma 250–350 ms, histerezis, fare düğmesi basılıyken ve tam ekranda tetiklenmez. | Şartname 7.7–7.8; forum şikâyetleri (RESEARCH §3) |
| D9 | Tipografi: Segoe UI Variable (Windows'ta yerleşik, lisans sorunu yok); sayılar `tabular-nums`; roller: saat, başlık, gövde, ikincil, mikro. | Şartname 3.2 |
| D10 | İkonlar kendi SVG setimiz (24×24, `currentColor`, 1.75 px çizgi). Emoji ve unicode glifi yok. | Şartname 4 |
| D11 | Petal logosu: yaprak/taç yaprağı biçiminde katmanlı SVG (3 yaprak), tek renk + vurgu; intro 1–1,5 sn, atlanabilir. | Şartname 15 |
