//! Tam ekran algılama ve pencere görünürlüğü.
//!
//! Tam ekran, Windows'un resmî `SHQueryUserNotificationState` bilgisinden anlaşılır
//! (oyun, tam ekran video, sunum). Pencere boyutunu ekranla karşılaştırmak yetmez: Petal görev
//! çubuğunu gizlediğinde normal bir "büyütülmüş" pencere de ekranı kaplar.

use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
};
use tauri::{AppHandle, Manager};
use windows::Win32::UI::Shell::{
    SHQueryUserNotificationState, QUNS_BUSY, QUNS_PRESENTATION_MODE, QUNS_RUNNING_D3D_FULL_SCREEN,
};

/// Tam ekranda gizlenen yüzeyler.
const HIDE_IN_FULLSCREEN: [&str; 2] = ["notch", "dock"];

static FULLSCREEN: AtomicBool = AtomicBool::new(false);
pub static HIDE_ENABLED: AtomicBool = AtomicBool::new(true);
static WANT: Mutex<Option<HashMap<String, bool>>> = Mutex::new(None);

/// Şu an tam ekran bir uygulama (oyun, video, sunum) ön planda mı?
pub fn fullscreen_app_active() -> bool {
    match unsafe { SHQueryUserNotificationState() } {
        Ok(s) => s == QUNS_BUSY || s == QUNS_RUNNING_D3D_FULL_SCREEN || s == QUNS_PRESENTATION_MODE,
        Err(_) => false,
    }
}

fn apply(app: &AppHandle, label: &str, want: bool) {
    let hidden_by_fs = HIDE_ENABLED.load(Ordering::Relaxed)
        && FULLSCREEN.load(Ordering::Relaxed)
        && HIDE_IN_FULLSCREEN.contains(&label);
    if let Some(w) = app.get_webview_window(label) {
        let show = want && !hidden_by_fs;
        if w.is_visible().unwrap_or(false) != show {
            let _ = if show { w.show() } else { w.hide() };
        }
    }
}

/// Arayüzün istediği görünürlüğü kaydeder ve tam ekran durumuna göre uygular.
pub fn set_visible(app: &AppHandle, label: &str, visible: bool) {
    WANT.lock()
        .unwrap()
        .get_or_insert_with(HashMap::new)
        .insert(label.to_string(), visible);
    apply(app, label, visible);
}

/// Pencere olaylarından sonra çağrılır; tam ekran durumu değiştiyse yüzeyleri gizler/gösterir.
pub fn check(app: &AppHandle) {
    let now = fullscreen_app_active();
    if FULLSCREEN.swap(now, Ordering::Relaxed) == now {
        return;
    }
    let wants: Vec<(String, bool)> = WANT
        .lock()
        .unwrap()
        .as_ref()
        .map(|m| m.iter().map(|(k, v)| (k.clone(), *v)).collect())
        .unwrap_or_default();
    for (label, want) in wants {
        apply(app, &label, want);
    }
}
