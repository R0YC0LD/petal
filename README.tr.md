<div align="center">

# 🌸 Petal

**Windows 10/11 için sakin, hızlı ve yerel hissettiren bir üst çubuk ve dock.**
Canlı sistem bilgisi, medya kontrolü, akıllı dock ve kenar ses/parlaklık kapsülleri, tek küçük uygulamada.

[![Sürüm](https://img.shields.io/github/v/release/R0YC0LD/petal?style=flat-square&label=s%C3%BCr%C3%BCm)](https://github.com/R0YC0LD/petal/releases/latest)
[![Lisans: MIT](https://img.shields.io/badge/lisans-MIT-blue?style=flat-square)](LICENSE)
![Platform](https://img.shields.io/badge/platform-Windows%2010%2F11-0078D4?style=flat-square)
![Teknoloji](https://img.shields.io/badge/Tauri%202%20%2B%20Rust%20%2B%20React-orange?style=flat-square)

[English](README.md) · **Türkçe**

![Üst çubuk](docs/img/notch.png)

![Dock](docs/img/dock.png)

</div>

## Neden Petal

Ekranda gerçekten baktığın şeyleri iki küçük yüzeyde toplar: üst çubuk ve dock. Windows'tan gerçek veri okur (sahte widget yok), ekranının yenileme hızında akar ve kullanmadığında köşeye çekilir.

- **Ücretsiz ve açık kaynak**, MIT lisanslı. Hesap yok, telemetri yok.
- **Hafif**: boştayken yaklaşık %0,3 CPU, Rust arka uç, yüzey başına tek hafif WebView.
- **Akıcı**: yaylı animasyonlar, açılışta gecikme yok, kapanma gecikmesi sende.

## Özellikler

### Üst çubuk
- Saat, tarih ve canlı **CPU / GPU / RAM** yüzdeleri
- Çalan medya: albüm ve çalma listesi bilgisi, oynat/duraklat/atla/ileri sar (Spotify, tarayıcılar ve Windows'a medya bildiren her uygulama)
- Şehrin için hava durumu (otomatik veya elle seçim)
- Boştayken küçük saate iner, üzerine gelince açılır
- **Hızlı arama** (`Ctrl+Shift+Space`, olmazsa `Alt+Space`): uygulama, açık pencere ve basit hesap; tamamen klavyeyle

### Dock
- İstediğin uygulamayı sabitle: `.exe` veya kısayolu dock'a sürükle ya da Başlat menüsü listesinden seç
- Sabit uygulamaları **sürükleyerek sırala**
- **Açık uygulamaları** ve pencerelerini gösterir; üzerine gelince canlı **pencere önizlemesi**
- Sağ tık menüsü: yeni pencere, sabitle/kaldır, tüm pencereleri kapat
- Arka plan uygulamaları paneli (bildirim alanı taşması) ve **Masaüstünü göster** düğmesi
- Büyütme efekti, açılışta zıplama, otomatik gizleme ve ayarlanabilir **kapanma süresi**

### Kenar kontrolleri
- Fareyi **sol kenara** götürünce dikey ses, **sağ kenara** götürünce parlaklık kapsülü açılır
- İkisi aynı anda açık olabilir

### Diğer
- Tam ekran oyun, video ve sunumlarda kendiliğinden gizlenir
- İstersen Windows görev çubuğunu gizler (çıkışta her zaman geri gelir; kurtarma komutu aşağıda)
- Koyu, açık veya sistem teması, vurgu rengi, ölçek, yüzey saydamlığı
- Ayarlar penceresi; ayarlar düz JSON olarak, otomatik yedekle saklanır

## Kurulum

1. [Son sürümden](https://github.com/R0YC0LD/petal/releases/latest) `Petal_x.y.z_x64-setup.exe` dosyasını indir.
2. Çalıştır. Yönetici izni gerekmez; `%LOCALAPPDATA%\Petal` klasörüne kurulur.
3. Petal hemen başlar. **Ayarlar** için üst çubuğa veya tepsi simgesine sağ tıkla.

> Kurulum dosyası henüz dijital imzalı değil, Windows SmartScreen uyarı verebilir. **Ek bilgi → Yine de çalıştır** de.

Gereksinim: Windows 10 veya 11 (x64) ve WebView2 çalışma zamanı (Windows 11'de hazır gelir).

**Kaldırma:** Windows Ayarlar → Uygulamalar → Petal.

## Kullanım

| İşlem | Nasıl |
|---|---|
| Üst çubuğu aç | Üzerine gel |
| Ayarları aç | Üst çubuğa veya tepsi simgesine sağ tık |
| Hızlı arama | `Ctrl+Shift+Space` (veya `Alt+Space`) |
| Dock'a uygulama ekle | `.exe` / kısayolu dock'a sürükle, ya da Ayarlar → Dock |
| Dock sırasını değiştir | Simgeyi basılı tutup yana sürükle |
| Pencere önizlemesi | Açık uygulamanın simgesine gel |
| Dock menüsü | Simgeye sağ tık |
| Ses / parlaklık | Fareyi sol / sağ ekran kenarına götür |

## Ayarlar

| Bölüm | Seçenekler |
|---|---|
| Genel | Tema, vurgu rengi, arayüz ölçeği, yüzey saydamlığı, Windows ile başlat, görev çubuğunu gizle |
| Üst çubuk | Açık sabitle, otomatik küçülme süresi, panel ve metrik sırası, şehir, tam ekranda gizle, arama kısayolu |
| Dock | Simge boyutu, büyütme, etiketler, açık uygulamaları göster, otomatik gizle, **dock kapanma süresi**, sabit uygulamalar |
| Kenar kapsülleri | Aç / kapat, kapanma gecikmesi |

Ayar dosyası: `%APPDATA%\com.by599.petal\settings.json`.

## Sorun giderme

- **Çökme sonrası görev çubuğu yok:** `Win+R` ile `%LOCALAPPDATA%\Petal\petal.exe --restore-taskbar` çalıştır.
- **Arama kısayolu çalışmıyor:** başka bir uygulama tutuyor olabilir; Petal sıradaki alternatifi dener.
- **Hava durumu yok:** Ayarlar → Üst çubuk bölümünden şehir seç.

## Kaynaktan derleme

Gereksinimler: Windows 10/11, [Node.js](https://nodejs.org) 20+, [Rust](https://rustup.rs) (stable, MSVC), WebView2.

```bash
git clone https://github.com/R0YC0LD/petal.git
cd petal
npm install
npm run tauri dev      # geliştirme modunda çalıştır
npm run tauri build    # src-tauri/target/release/bundle/nsis altında kurulum dosyası üretir
```

Teknoloji: **Tauri 2**, **Rust** (`windows` crate: Core Audio, GSMTC medya, WinEvent kancaları), **React 19**, **TypeScript**, **Motion**, Vite.
Ek notlar [`docs/`](docs) klasöründe.

## Bilinen sınırlar

- Şimdilik tek monitör (çoklu monitör planlı)
- Arka plan paneli, uygulamaların kendi tepsi menülerini gösteremez
- Kurulum dosyası imzasız

Ayrıntı: [`docs/KNOWN_ISSUES.md`](docs/KNOWN_ISSUES.md) ve [değişiklik günlüğü](docs/CHANGELOG.md).

## Lisans

[MIT](LICENSE) © Onur Teryakioğlu ([R0YC0LD](https://github.com/R0YC0LD))
