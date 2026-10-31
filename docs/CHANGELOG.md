# Değişiklik Günlüğü

## 0.9.11
- Dock: sabit uygulamalar sürüklenerek yeniden sıralanır (yaylı animasyon, tıklama ile karışmaz, sıra kalıcıdır).
- Dock: ayrı "Dock kapanma süresi" ayarı (Anında - 5 sn, varsayılan 1,2 sn); otomatik gizleme ve önizleme/menü panelleri bu süreyi kullanır.

## 0.9.1
- Üst çubuk yeniden yazıldı: ekranın üst kenarına bitişik tek yüzey, SOL | MERKEZ | SAĞ ızgarası, tek medya oynatıcı (ayrı ayrıntı çekmecesi ve ikinci kopya kaldırıldı).
- Boşta (varsayılan 7 sn) yüzey sıvı geçişle yalnızca saate küçülür; üzerine gelince veya tıklayınca genişler. Sabitleme ve süre ayarı eklendi.
- Medya: ilerleme çubuğundan konum değiştirme (seek), tek biçimli geçen/kalan süre, karıştır/tekrarla ikincil düğmeler.
- Çakışan eski CSS kuralları budandı. Kompakt durumda istatistik ve saniye güncellemeleri durduruldu.

## 0.9.0
- Yeni üst çubuk: sol saat/tarih/CPU/GPU/RAM, orta medya, sağ hava.
- Kompakt yatay kenar kapsülleri (ses sol, parlaklık sağ); dikey kart kaldırıldı.
- Ses: kalıcı COM iş parçacığı, son değer birleştirme. Parlaklık: IOCTL, süreç başlatma yok.
- CPU/RAM Win32 ile, GPU motor türleri arası en büyük; `sysinfo` kaldırıldı; olay tabanlı `stats`.
- Tasarım belirteçleri, SVG ikon seti, Petal logosu.
- Belgeler: RESEARCH, AUDIT, DESIGN/TECHNICAL_DECISIONS, ROADMAP, STATUS, QA_REPORT.

## 0.8.x
Dock pencere odaklama ve önizleme, görev çubuğu gizleme ve kurtarma, hover niyeti, HUD düzeltmeleri.

## 0.9.2
- Kenar kontrolleri dikey kapsüle çevrildi (56×176, üstte yüzde, dikey kaydırıcı, altta simge). Pencere uygulama açılışında önceden oluşturulup gizli tutulur; açılış yalnızca konumlandırma + gösterme.
- Açılış gecikmesi kaldırıldı (üst çubuk, kenar kapsülleri, dock). Yalnızca kapanma gecikmesi ayarlanır.
- Üst çubuk animasyonu pencere/düzen boyutunu değil, sabit kutunun clip-path kırpmasını anime eder; içerik yeniden yerleşmez.

## 0.9.3 - 0.9.5
- Ses ve parlaklık kapsülleri artık iki ayrı, önceden hazırlanmış pencere; ikisi aynı anda açık olabilir.
- Üst çubukta içerik bölgeleri sırayla (sol, orta, sağ) belirir; kapsül ve çubuk yüzeyi saydamlık ayarından bağımsız en az %92 opak (okunabilirlik).
- Kare süresi ölçümü eklendi (bkz. QA_REPORT #24-27).

## 0.9.6 - 0.9.7
- Kapsül ve çubuk yüzeylerinin arkadaki pencere içeriğini göstermesinin kök nedeni, ayarlardaki düşük yüzey opaklığıydı. Opaklık artık uygulamada en az %92'ye sabitleniyor; ayar kaydırıcısının alt sınırı %92.

## 0.9.8
- Dock: arka plan uygulamaları paneli (Petal tasarımı) ve masaüstünü göster/geri al düğmesi eklendi.
- Arka plan uygulamaları, Windows'un 'Diğer sistem tepsisi simgeleri' kayıt listesinden okunur ve yalnızca çalışanlar gösterilir; tıklayınca uygulama öne gelir veya açılır, sağ tık menüsünde Aç ve Kapat (pencereye kapatma iletisi) bulunur.

## 0.9.9
- Dock: sabit olmayan, açık penceresi olan uygulamalar da gösterilir (ayraçtan sonra, gerçek ikonlarıyla). Pencere değişimleri Windows olay kancasıyla anında yansır (yoklama yalnızca güvenlik ağı).
- Dock sağ tık menüsü: Yeni pencere / Aç, Dock'ta tut / Dock'tan kaldır, pencereyi kapat. Etkin uygulamada vurgulu gösterge; öndeki tek pencereye tekrar tıklayınca küçülür, çok pencerede sıradakine geçer.
- Hızlı arama (üst çubuğun içinde): uygulama, açık pencere ve hesaplama; ok tuşları, Enter, Esc. Kısayol Alt+Space; başkası tutuyorsa Ctrl+Alt+Space, Ctrl+Shift+Space, Alt+Shift+Space sırasıyla denenir, kullanılan Ayarlar'da görünür.
- Tam ekran algılama SHQueryUserNotificationState ile yapılır; tam ekran oyun/videoda üst çubuk ve dock gizlenir, kenar kapsülleri açılmaz. Önceden pencere boyutu ekranı kapladığında (görev çubuğu gizliyken büyütülmüş pencereler) kapsüller yanlışlıkla açılmıyordu; düzeltildi.
