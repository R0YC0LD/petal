//! Windows Global System Media Transport Controls (GSMTC) üzerinden gerçek medya oturumu bilgisi
//! ve varsayılan ses cihazının anlık tepe seviyesi.

use base64::Engine;
use serde::Serialize;
use std::sync::Mutex;
use windows::{
    core::Result as WinResult,
    Media::Control::{
        GlobalSystemMediaTransportControlsSession as Session,
        GlobalSystemMediaTransportControlsSessionManager as Manager,
        GlobalSystemMediaTransportControlsSessionPlaybackStatus as Status,
    },
    Media::MediaPlaybackAutoRepeatMode as Repeat,
    Storage::Streams::DataReader,
    Win32::{
        Media::Audio::{
            eConsole, eRender, Endpoints::IAudioMeterInformation, IMMDeviceEnumerator,
            MMDeviceEnumerator,
        },
        System::Com::{CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_MULTITHREADED},
    },
};

#[derive(Serialize, Clone)]
pub struct MediaInfo {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub playing: bool,
    /// Saniye cinsinden; kaynak uygulama zaman çizelgesi vermiyorsa 0.
    pub position: f64,
    pub duration: f64,
    pub cover: Option<String>,
    pub can_prev: bool,
    pub can_next: bool,
    pub source: String,
    /// `None`: kaynak uygulama karıştırmayı desteklemiyor veya bildirmiyor.
    pub shuffle: Option<bool>,
    /// "none" | "track" | "list"; `None`: desteklenmiyor.
    pub repeat: Option<String>,
}

struct CoverCache {
    key: String,
    data: Option<String>,
}

static COVER: Mutex<CoverCache> = Mutex::new(CoverCache {
    key: String::new(),
    data: None,
});

fn init_com() {
    // Aynı iş parçacığında tekrar çağrılırsa hata döner, yok sayılır.
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
    }
}

fn current_session() -> WinResult<Session> {
    Manager::RequestAsync()?.join()?.GetCurrentSession()
}

fn read_cover(session: &Session) -> Option<String> {
    let props = session.TryGetMediaPropertiesAsync().ok()?.join().ok()?;
    let thumb = props.Thumbnail().ok()?;
    let stream = thumb.OpenReadAsync().ok()?.join().ok()?;
    let size = stream.Size().ok()? as u32;
    if size == 0 || size > 8 * 1024 * 1024 {
        return None;
    }
    let mime = stream.ContentType().ok()?.to_string();
    let reader = DataReader::CreateDataReader(&stream).ok()?;
    reader.LoadAsync(size).ok()?.join().ok()?;
    let mut buf = vec![0u8; size as usize];
    reader.ReadBytes(&mut buf).ok()?;
    let mime = if mime.is_empty() {
        "image/jpeg".to_string()
    } else {
        mime
    };
    Some(format!(
        "data:{mime};base64,{}",
        base64::engine::general_purpose::STANDARD.encode(buf)
    ))
}

fn ticks_to_secs(ticks: i64) -> f64 {
    ticks as f64 / 10_000_000.0
}

/// Etkin medya oturumunu döner; oturum yoksa `None`.
pub fn current() -> Option<MediaInfo> {
    // Yalnızca görsel test için: PETAL_FAKE_MEDIA ortam değişkeni ayarlıysa sahte oturum döner.
    // Normal kullanımda bu değişken yoktur ve gerçek GSMTC oturumu okunur.
    if let Some(v) = std::env::var_os("PETAL_FAKE_MEDIA") {
        let long = v.to_string_lossy() == "long";
        return Some(MediaInfo {
            title: if long {
                "Katliam 3 (Remastered) — Extended Mix ve çok uzun bir şarkı adı".into()
            } else {
                "Rocketship".into()
            },
            artist: if long {
                "Massaka feat. Çok Uzun Bir Sanatçı Adı".into()
            } else {
                "Llunr".into()
            },
            album: "Test Albümü".into(),
            playing: true,
            position: 94.0,
            duration: 156.0,
            cover: None,
            can_prev: true,
            can_next: true,
            source: "test".into(),
            shuffle: Some(true),
            repeat: Some("list".into()),
        });
    }
    init_com();
    let session = current_session().ok()?;
    let props = session.TryGetMediaPropertiesAsync().ok()?.join().ok()?;
    let title = props.Title().ok()?.to_string();
    let artist = props.Artist().map(|s| s.to_string()).unwrap_or_default();
    let album = props
        .AlbumTitle()
        .map(|s| s.to_string())
        .unwrap_or_default();
    if title.is_empty() && artist.is_empty() {
        return None;
    }

    let info = session.GetPlaybackInfo().ok()?;
    let playing = info.PlaybackStatus().ok()? == Status::Playing;
    let (can_prev, can_next) = info
        .Controls()
        .map(|c| {
            (
                c.IsPreviousEnabled().unwrap_or(false),
                c.IsNextEnabled().unwrap_or(false),
            )
        })
        .unwrap_or((false, false));

    let can_shuffle = info
        .Controls()
        .map(|c| c.IsShuffleEnabled().unwrap_or(false))
        .unwrap_or(false);
    let can_repeat = info
        .Controls()
        .map(|c| c.IsRepeatEnabled().unwrap_or(false))
        .unwrap_or(false);
    let shuffle = if can_shuffle {
        info.IsShuffleActive().ok().and_then(|r| r.Value().ok())
    } else {
        None
    };
    let repeat = if can_repeat {
        info.AutoRepeatMode()
            .ok()
            .and_then(|r| r.Value().ok())
            .map(|m| {
                if m == Repeat::Track {
                    "track"
                } else if m == Repeat::List {
                    "list"
                } else {
                    "none"
                }
                .to_string()
            })
    } else {
        None
    };

    let (position, duration) = session
        .GetTimelineProperties()
        .map(|t| {
            let start = t.StartTime().map(|d| d.Duration).unwrap_or(0);
            let end = t.EndTime().map(|d| d.Duration).unwrap_or(0);
            let pos = t.Position().map(|d| d.Duration).unwrap_or(0);
            (
                ticks_to_secs((pos - start).max(0)),
                ticks_to_secs((end - start).max(0)),
            )
        })
        .unwrap_or((0.0, 0.0));

    let key = format!("{title}\u{1}{artist}\u{1}{album}");
    let cover = {
        let mut cache = COVER.lock().ok()?;
        if cache.key != key || cache.data.is_none() {
            cache.data = read_cover(&session);
            cache.key = key;
        }
        cache.data.clone()
    };

    let source = session
        .SourceAppUserModelId()
        .map(|s| s.to_string())
        .unwrap_or_default();
    Some(MediaInfo {
        title,
        artist,
        album,
        playing,
        position,
        duration,
        cover,
        can_prev,
        can_next,
        source,
        shuffle,
        repeat,
    })
}

pub fn control(action: &str) -> Result<(), String> {
    init_com();
    let s = current_session().map_err(|_| "etkin medya oturumu yok".to_string())?;
    let ok = match action {
        "toggle" => s.TryTogglePlayPauseAsync(),
        "next" => s.TrySkipNextAsync(),
        "prev" => s.TrySkipPreviousAsync(),
        "shuffle" => {
            let now = s
                .GetPlaybackInfo()
                .ok()
                .and_then(|i| i.IsShuffleActive().ok())
                .and_then(|r| r.Value().ok())
                .unwrap_or(false);
            s.TryChangeShuffleActiveAsync(!now)
        }
        "repeat" => {
            let now = s
                .GetPlaybackInfo()
                .ok()
                .and_then(|i| i.AutoRepeatMode().ok())
                .and_then(|r| r.Value().ok());
            let next = match now {
                Some(m) if m == Repeat::List => Repeat::Track,
                Some(m) if m == Repeat::Track => Repeat::None,
                _ => Repeat::List,
            };
            s.TryChangeAutoRepeatModeAsync(next)
        }
        _ => return Err("bilinmeyen komut".into()),
    }
    .map_err(|e| e.to_string())?
    .join()
    .map_err(|e| e.to_string())?;
    if ok {
        Ok(())
    } else {
        Err("uygulama bu komutu kabul etmedi".into())
    }
}

/// Varsayılan çıkış cihazının anlık tepe seviyesi, 0.0 - 1.0. Cihaz yoksa `None`.
pub fn audio_peak() -> Option<f32> {
    init_com();
    unsafe {
        let en: IMMDeviceEnumerator =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).ok()?;
        let dev = en.GetDefaultAudioEndpoint(eRender, eConsole).ok()?;
        let meter: IAudioMeterInformation = dev.Activate(CLSCTX_ALL, None).ok()?;
        meter.GetPeakValue().ok()
    }
}

/// Çalma konumunu saniye cinsinden değiştirir (kaynak uygulama desteklemiyorsa hata döner).
pub fn seek(seconds: f64) -> Result<(), String> {
    init_com();
    let s = current_session().map_err(|_| "etkin medya oturumu yok".to_string())?;
    let ticks = (seconds.max(0.0) * 10_000_000.0) as i64;
    let ok = s
        .TryChangePlaybackPositionAsync(ticks)
        .map_err(|e| e.to_string())?
        .join()
        .map_err(|e| e.to_string())?;
    if ok {
        Ok(())
    } else {
        Err("uygulama konum değiştirmeyi kabul etmedi".into())
    }
}
