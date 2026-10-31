//! Gerçek sistem ölçümleri: CPU (GetSystemTimes), RAM (GlobalMemoryStatusEx), GPU (PDH).
//! Tek bir örnekleme iş parçacığı değerleri hesaplar ve `stats` olayı olarak arayüze iter.

use serde::Serialize;
use std::{
    sync::{
        atomic::{AtomicU32, Ordering},
        Mutex,
    },
    thread,
    time::Duration,
};
use tauri::{AppHandle, Emitter};
use windows::Win32::{
    Foundation::FILETIME,
    System::{
        SystemInformation::{GlobalMemoryStatusEx, MEMORYSTATUSEX},
        Threading::GetSystemTimes,
    },
};

#[derive(Serialize, Clone, Copy, Default, PartialEq, Debug)]
pub struct Stats {
    pub cpu: f32,
    pub ram: f32,
    /// `None`: bu sistemde GPU sayacı yok veya henüz ölçülemedi (arayüzde N/A).
    pub gpu: Option<f32>,
}

/// Örnekleme aralığı (ms). Ayarlardan değiştirilebilir.
pub static INTERVAL_MS: AtomicU32 = AtomicU32::new(1000);
static LATEST: Mutex<Stats> = Mutex::new(Stats {
    cpu: 0.0,
    ram: 0.0,
    gpu: None,
});

fn ticks(f: FILETIME) -> u64 {
    ((f.dwHighDateTime as u64) << 32) | f.dwLowDateTime as u64
}

/// (boşta, çekirdek, kullanıcı). Çekirdek süresi boşta süreyi de içerir.
fn times() -> Option<(u64, u64, u64)> {
    let (mut i, mut k, mut u) = (
        FILETIME::default(),
        FILETIME::default(),
        FILETIME::default(),
    );
    unsafe { GetSystemTimes(Some(&mut i), Some(&mut k), Some(&mut u)).ok()? };
    Some((ticks(i), ticks(k), ticks(u)))
}

/// İki örnek arasındaki CPU yüzdesi.
pub fn cpu_percent(prev: (u64, u64, u64), now: (u64, u64, u64)) -> f32 {
    let idle = now.0.saturating_sub(prev.0) as f64;
    let total = (now.1.saturating_sub(prev.1) + now.2.saturating_sub(prev.2)) as f64;
    if total <= 0.0 {
        return 0.0;
    }
    (((total - idle) / total) * 100.0).clamp(0.0, 100.0) as f32
}

fn ram_percent() -> Option<f32> {
    let mut m = MEMORYSTATUSEX {
        dwLength: std::mem::size_of::<MEMORYSTATUSEX>() as u32,
        ..Default::default()
    };
    unsafe { GlobalMemoryStatusEx(&mut m).ok()? };
    Some(m.dwMemoryLoad as f32)
}

pub fn latest() -> Stats {
    *LATEST.lock().unwrap()
}

pub fn start(app: AppHandle) {
    thread::spawn(move || {
        let mut prev = times();
        let mut last = Stats::default();
        loop {
            thread::sleep(Duration::from_millis(
                INTERVAL_MS.load(Ordering::Relaxed).clamp(250, 10_000) as u64,
            ));
            let now = times();
            let cpu = match (prev, now) {
                (Some(p), Some(n)) => cpu_percent(p, n),
                _ => last.cpu,
            };
            prev = now;
            let s = Stats {
                cpu,
                ram: ram_percent().unwrap_or(last.ram),
                gpu: crate::gpu::usage(),
            };
            if s != last {
                *LATEST.lock().unwrap() = s;
                let _ = app.emit("stats", s);
                last = s;
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idle_system_is_zero() {
        // Tüm geçen süre boşta: kullanım 0.
        assert_eq!(cpu_percent((0, 0, 0), (100, 100, 0)), 0.0);
    }

    #[test]
    fn half_busy() {
        // Çekirdek 100 (50'si boşta) + kullanıcı 100 → toplam 200, meşgul 150 → %75.
        let v = cpu_percent((0, 0, 0), (50, 100, 100));
        assert!((v - 75.0).abs() < 0.01, "{v}");
    }

    #[test]
    fn no_elapsed_time_is_safe() {
        assert_eq!(cpu_percent((5, 5, 5), (5, 5, 5)), 0.0);
    }

    #[test]
    fn live_values_are_sane() {
        let a = times().unwrap();
        thread::sleep(Duration::from_millis(250));
        let b = times().unwrap();
        let c = cpu_percent(a, b);
        assert!((0.0..=100.0).contains(&c));
        let r = ram_percent().unwrap();
        assert!(r > 0.0 && r <= 100.0);
    }
}
