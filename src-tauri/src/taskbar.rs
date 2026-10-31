//! Windows görev çubuğunu isteğe bağlı gizleme ve geri yükleme.
//! Explorer'a dokunulmaz: yalnızca belgelenmiş AppBar durumu ve pencere görünürlüğü değiştirilir.
//! Önceki durum diske yazılır; uygulama çökse bile bir sonraki açılışta veya
//! `--restore-taskbar` argümanıyla geri yüklenebilir.

use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::Duration,
};
use windows::{
    core::w,
    Win32::{
        Foundation::HWND,
        UI::{
            Shell::{SHAppBarMessage, ABM_GETSTATE, ABM_SETSTATE, APPBARDATA},
            WindowsAndMessaging::{
                FindWindowExW, FindWindowW, IsWindowVisible, ShowWindow, SW_HIDE, SW_SHOWNA,
            },
        },
    },
};

const ABS_AUTOHIDE: usize = 1;
const ABS_ALWAYSONTOP: usize = 2;

static ACTIVE: AtomicBool = AtomicBool::new(false);
static WATCHDOG: AtomicBool = AtomicBool::new(false);

fn hide_visible() {
    for h in tray_windows() {
        unsafe {
            if IsWindowVisible(h).as_bool() {
                let _ = ShowWindow(h, SW_HIDE);
            }
        }
    }
}

/// Explorer çubuğu kendiliğinden geri gösterebilir veya yeniden başlatılabilir;
/// gizleme etkinken iki saniyede bir kontrol edilir.
fn start_watchdog() {
    if WATCHDOG.swap(true, Ordering::SeqCst) {
        return;
    }
    thread::spawn(|| loop {
        thread::sleep(Duration::from_secs(2));
        if ACTIVE.load(Ordering::SeqCst) {
            hide_visible();
        }
    });
}

fn tray_windows() -> Vec<HWND> {
    let mut out = Vec::new();
    unsafe {
        if let Ok(h) = FindWindowW(w!("Shell_TrayWnd"), None) {
            out.push(h);
        }
        let mut prev = HWND::default();
        while let Ok(h) = FindWindowExW(None, Some(prev), w!("Shell_SecondaryTrayWnd"), None) {
            out.push(h);
            prev = h;
        }
    }
    out
}

fn bar_data() -> APPBARDATA {
    let hwnd = unsafe { FindWindowW(w!("Shell_TrayWnd"), None).unwrap_or_default() };
    APPBARDATA {
        cbSize: std::mem::size_of::<APPBARDATA>() as u32,
        hWnd: hwnd,
        ..Default::default()
    }
}

fn get_state() -> usize {
    let mut d = bar_data();
    unsafe { SHAppBarMessage(ABM_GETSTATE, &mut d) as usize }
}

fn set_state(state: usize) {
    let mut d = bar_data();
    d.lParam = windows::Win32::Foundation::LPARAM(state as isize);
    unsafe {
        SHAppBarMessage(ABM_SETSTATE, &mut d);
    }
}

fn flag_path(dir: &PathBuf) -> PathBuf {
    dir.join("taskbar-state.txt")
}

/// Görev çubuğunu gizler. Önceki durum `dir` altına kaydedilir.
pub fn hide(dir: &PathBuf) {
    let flag = flag_path(dir);
    if !flag.exists() {
        let _ = fs::create_dir_all(dir);
        let _ = fs::write(&flag, get_state().to_string());
    }
    ACTIVE.store(true, Ordering::SeqCst);
    set_state(ABS_AUTOHIDE | ABS_ALWAYSONTOP);
    // Explorer durum değişikliğini eşzamansız işler; hemen gizlersek çubuğu geri gösterebilir.
    thread::spawn(|| {
        thread::sleep(Duration::from_millis(800));
        if ACTIVE.load(Ordering::SeqCst) {
            hide_visible();
        }
    });
    start_watchdog();
}

/// Görev çubuğunu kayıtlı önceki durumuna döndürür.
pub fn restore(dir: &PathBuf) {
    ACTIVE.store(false, Ordering::SeqCst);
    let flag = flag_path(dir);
    let prev = fs::read_to_string(&flag)
        .ok()
        .and_then(|s| s.trim().parse::<usize>().ok())
        .unwrap_or(ABS_ALWAYSONTOP);
    // Önce otomatik gizlemeyi eski değerine al, sonra pencereyi göster.
    set_state(prev);
    for h in tray_windows() {
        unsafe {
            let _ = ShowWindow(h, SW_SHOWNA);
        }
    }
    let _ = fs::remove_file(flag);
}

pub fn is_hidden_by_us(dir: &PathBuf) -> bool {
    flag_path(dir).exists()
}
