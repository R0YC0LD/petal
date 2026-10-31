# Bilinen Sorunlar (0.9.0)

| Öncelik | Sorun | Tekrar üretme / not |
|---|---|---|
| P2 | Hava: IP tabanlı konum bu ağda başarısız olabiliyor | Ayarlar > Notch > Şehir ile elle seçilmeli; çubukta "Şehir seç" görünür |
| P2 | Müzik görselleştirici toplam ses seviyesini kullanır, frekans analizi değil | WASAPI loopback henüz yok |
| P2 | Medya ve pencere listesi yoklama ile çalışır | GSMTC olay mimarisi planlandı |
| P2 | Üst çubuk 560–980 px genişliğinde ekranın üstünü kaplar, altındaki sekmelere tıklamayı engelleyebilir | Çubuk boyutu ayarı henüz yok |
| P2 | Kenar kapsülleri yalnızca birincil monitörde, yalnızca orta yükseklikte | Konum ve ekran seçimi planlandı |
| P3 | Pencere önizlemesi simge durumundaki pencerelerde görüntü vermez | DWM küçük resmi araştırılacak |
| P3 | Kullanılmayan eski CSS kuralları `styles.css` içinde duruyor | Temizlik planlandı |
| Bilgi | Kurulum dosyası dijital imzalı değil | SmartScreen uyarı verebilir |

| P2 | Arka plan uygulamaları panelinde uygulamaların kendi tepsi menüleri (ör. Çıkış, Kullanım) gösterilemez | Windows 11'de gizli simgeler XAML ile çizilir ve görev çubuğu gizliyken başka süreçlerce okunamaz (UI Automation ile denendi). Panel kayıt defteri listesini kullanır; simge görüntüsü uygulamanın exe simgesidir, tepsi simgesinin kendisi değildir |
