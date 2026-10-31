//! Windows ana ses seviyesi. Kalıcı bir iş parçacığı `IAudioEndpointVolume` nesnesini tutar;
//! komutlar kuyruğa girer ve art arda gelen ayarlardan yalnızca en sonuncusu uygulanır.

use serde::Serialize;
use std::sync::{mpsc, Mutex, OnceLock};
use windows::Win32::{
    Media::Audio::{
        eConsole, eRender, Endpoints::IAudioEndpointVolume, IMMDeviceEnumerator, MMDeviceEnumerator,
    },
    System::Com::{CoCreateInstance, CoInitializeEx, CLSCTX_ALL, COINIT_MULTITHREADED},
};

#[derive(Serialize, Clone, Copy, PartialEq, Debug)]
pub struct Volume {
    pub level: f32,
    pub muted: bool,
}

enum Cmd {
    Set(f32),
    Mute(bool),
    Get(mpsc::Sender<Option<Volume>>),
}

static TX: OnceLock<Mutex<mpsc::Sender<Cmd>>> = OnceLock::new();

fn endpoint() -> Option<IAudioEndpointVolume> {
    unsafe {
        let en: IMMDeviceEnumerator =
            CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL).ok()?;
        let dev = en.GetDefaultAudioEndpoint(eRender, eConsole).ok()?;
        dev.Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None).ok()
    }
}

fn read(ep: &IAudioEndpointVolume) -> Option<Volume> {
    unsafe {
        Some(Volume {
            level: ep.GetMasterVolumeLevelScalar().ok()?,
            muted: ep.GetMute().ok()?.as_bool(),
        })
    }
}

fn worker(rx: mpsc::Receiver<Cmd>) {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
    }
    let mut ep = endpoint();
    while let Ok(first) = rx.recv() {
        // Kuyruktakileri topla: son ses seviyesi ve son sessiz durumu geçerli.
        let mut level = None;
        let mut mute = None;
        let mut gets = Vec::new();
        for cmd in std::iter::once(first).chain(rx.try_iter()) {
            match cmd {
                Cmd::Set(v) => level = Some(v),
                Cmd::Mute(m) => mute = Some(m),
                Cmd::Get(r) => gets.push(r),
            }
        }
        // Varsayılan cihaz değişmiş olabilir; hata olursa yeniden edin.
        if ep.is_none() {
            ep = endpoint();
        }
        let mut failed = false;
        if let Some(e) = ep.as_ref() {
            unsafe {
                if let Some(v) = level {
                    failed |= e
                        .SetMasterVolumeLevelScalar(v.clamp(0.0, 1.0), std::ptr::null())
                        .is_err();
                    if v > 0.0 && mute.is_none() {
                        let _ = e.SetMute(false, std::ptr::null());
                    }
                }
                if let Some(m) = mute {
                    failed |= e.SetMute(m, std::ptr::null()).is_err();
                }
            }
        }
        let vol = ep.as_ref().and_then(read);
        if failed || vol.is_none() {
            ep = endpoint();
        }
        let vol = vol.or_else(|| ep.as_ref().and_then(read));
        for g in gets {
            let _ = g.send(vol);
        }
    }
}

fn tx() -> mpsc::Sender<Cmd> {
    TX.get_or_init(|| {
        let (tx, rx) = mpsc::channel();
        std::thread::Builder::new()
            .name("petal-audio".into())
            .spawn(move || worker(rx))
            .ok();
        Mutex::new(tx)
    })
    .lock()
    .unwrap()
    .clone()
}

pub fn get() -> Option<Volume> {
    let (r, rx) = mpsc::channel();
    tx().send(Cmd::Get(r)).ok()?;
    rx.recv_timeout(std::time::Duration::from_millis(500))
        .ok()
        .flatten()
}

pub fn set(level: f32) {
    let _ = tx().send(Cmd::Set(level));
}

pub fn mute(m: bool) {
    let _ = tx().send(Cmd::Mute(m));
}
