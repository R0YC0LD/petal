# Teknik Kararlar

| # | Karar | Gerekçe |
|---|---|---|
| T1 | Tauri 2 + Rust + React/TS kalır. | Mevcut proje çalışıyor; RAM ve CPU Electron'dan düşük (ölçüldü: boşta ~%0,7 CPU, ~300 MB). |
| T2 | CPU: `GetSystemTimes` farkı. RAM: `GlobalMemoryStatusEx`. İkisi de `sysinfo` yerine doğrudan Win32; tek örnekleme iş parçacığı, sonuç önbelleği, arayüze `stats` olayı olarak itilir. | Şartname 6; gereksiz yoklama ve kilit çekişmesi azalır. |
| T3 | GPU: PDH `GPU Engine(*)\Utilization Percentage`; (LUID, motor türü) başına işlemleri topla, adaptör için motor türleri arasında en büyüğünü, adaptörler arasında en büyüğünü raporla. Sayaç yok veya ilk örnek: `null` → arayüzde `N/A`. | RESEARCH §4 |
| T4 | Ses: kalıcı `IAudioEndpointVolume` (özel iş parçacığında COM, MTA); `IAudioEndpointVolumeCallback` ile dış değişim olayı. | Şartname 7.3 |
| T5 | Parlaklık: dahili panel için sürücü IOCTL (`\\.\LCD`) veya kalıcı WMI COM bağlantısı; harici için Monitor Configuration API. PowerShell süreci yasak. En son değer kuyruğu (coalescing). | Şartname 7.4–7.6 |
| T6 | Medya: GSMTC olayları (`CurrentSessionChanged`, `MediaPropertiesChanged`, `PlaybackInfoChanged`, `TimelinePropertiesChanged`) ile olay tabanlı; yoklama yalnızca ilerleme çubuğu için yerel zamanlayıcı. | RESEARCH §4 |
| T7 | Görselleştirici: WASAPI loopback yalnızca kullanıcı açarsa; kapalıyken yalnızca çalma durumu animasyonu (gerçek ses analizi iddiası yok). | Şartname 10 |
| T8 | Kenar tetikleyici: dar (3–4 px) hotspot; fare konumu ve düğme durumu `GetCursorPos`/`GetAsyncKeyState` ile; ileride düşük seviyeli kanca. Tetikleyici iş parçacığı fiziksel piksel kullanır (DPI farkındalığı). | Şartname 7.7; hata geçmişi (DPI) |
| T9 | Ayarlar: JSON, `schemaVersion`, atomik yazım + `.bak`; bozuk/BOM'lu dosya kurtarma. Layout ve tema içe aktarımı şema doğrulamalı. | Mevcut `store.rs` + şartname 22 |
| T10 | Tauri capabilities: pencere başına ayrı capability dosyası, yalnızca gereken komutlar. CSP sıkı. | https://v2.tauri.app/security/ |
| T11 | `winlist` DWM thumbnail yerine `PrintWindow` (şimdilik); simge durumundaki pencereler için DWM küçük resim araştırılacak. | AUDIT P2-8 |
