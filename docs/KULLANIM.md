# Petal Kurulum ve Kullanım

**Kurulum:** `Petal-Setup.exe` dosyasını çalıştır. Yönetici izni gerekmez, kullanıcı klasörüne kurulur (`%LOCALAPPDATA%\Petal`).
**Taşınabilir:** `Petal-Portable.exe` doğrudan çalışır. Windows 10/11 üzerinde WebView2 gerekir.
**Kaldırma:** Windows Ayarlar › Uygulamalar › Petal, ya da `%LOCALAPPDATA%\Petal\uninstall.exe`. Windows görev çubuğuna dokunulmaz.

**Kullanım**
- Notch: üstüne gel, sayfalar arasında fare tekerleği veya alttaki sekmelerle gez. Sağ tık: Ayarlar.
- Dock: alt kenara imleci götürünce açılır. EXE veya kısayol sürükleyip bırakarak uygulama ekle.
- Tray ikonu: sol tık Ayarlar, sağ tık menü (Ayarlar, Çıkış).
- Ayarlar dosyası: `%APPDATA%\com.by599.petal\settings.json` (bozulursa `.bak` yedeğinden kurtarılır).

Geliştirici: R0YC0LD (Onur Teryakioğlu)

**Görev çubuğunu gizleme:** Ayarlar > Genel > Windows görev çubuğunu gizle. Petal kapanınca çubuk geri gelir. Sorun olursa tray menüsünden 'Windows görev çubuğunu geri getir', ya da Win+R ile `%LOCALAPPDATA%\Petal\petal.exe --restore-taskbar` çalıştır.
