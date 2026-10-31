//! Kompakt dikey kenar kapsülleri: ses solda, parlaklık sağda. İki kapsül ayrı pencerelerdir ve
//! birbirinden bağımsızdır; ikisi aynı anda açık olabilir.
//!
//! Pencereler uygulama açılışında önceden oluşturulup gizli tutulur; tetiklemede yalnızca
//! konumlanır ve gösterilir. Tetikleme: imleç ekranın en dış 3 pikselinde ve fare düğmesi basılı
//! değilken. Açılışta gecikme yoktur; yalnızca kapanma gecikmesi ayarlanır.

use std::{
    sync::{
        atomic::{AtomicBool, AtomicU32, Ordering},
        Mutex,
    },
    thread,
    time::{Duration, Instant},
};
use tauri::{
    AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindow,
    WebviewWindowBuilder,
};
use windows::Win32::{
    Foundation::POINT,
    UI::{
        HiDpi::{SetThreadDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2},
        Input::KeyboardAndMouse::{GetAsyncKeyState, VK_LBUTTON, VK_RBUTTON},
        WindowsAndMessaging::{GetCursorPos, GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN},
    },
};

/// Pencere boyutu (mantıksal piksel): 56×176 kapsül + gölge payı.
pub const WIN_W: f64 = 84.0;
pub const WIN_H: f64 = 204.0;
const EDGE_MARGIN: f64 = 8.0;
const HOTSPOT_PX: i32 = 3;

pub static ENABLED: AtomicBool = AtomicBool::new(true);
/// Kapanma gecikmesi (ms), ayarlardan gelir. Açılışta gecikme yoktur.
pub static CLOSE_DELAY_MS: AtomicU32 = AtomicU32::new(300);
type Rect4 = Option<(i32, i32, i32, i32)>;
static RECTS: Mutex<[Rect4; 2]> = Mutex::new([None, None]);

#[derive(Clone, Copy, PartialEq)]
enum Side {
    Left,
    Right,
}

impl Side {
    fn idx(self) -> usize {
        match self {
            Side::Left => 0,
            Side::Right => 1,
        }
    }
    fn label(self) -> &'static str {
        match self {
            Side::Left => "hud-left",
            Side::Right => "hud-right",
        }
    }
    pub fn parse(s: &str) -> Side {
        if s == "right" {
            Side::Right
        } else {
            Side::Left
        }
    }
}

fn cursor() -> Option<(i32, i32)> {
    let mut p = POINT::default();
    unsafe { GetCursorPos(&mut p).ok()? };
    Some((p.x, p.y))
}

fn button_down() -> bool {
    unsafe {
        (GetAsyncKeyState(VK_LBUTTON.0 as i32) as u16 & 0x8000) != 0
            || (GetAsyncKeyState(VK_RBUTTON.0 as i32) as u16 & 0x8000) != 0
    }
}

fn foreground_is_fullscreen(_sw: i32, _sh: i32) -> bool {
    crate::fullscreen::fullscreen_app_active()
}

fn build(app: &AppHandle, side: Side) -> Option<WebviewWindow> {
    WebviewWindowBuilder::new(app, side.label(), WebviewUrl::default())
        .title("Petal Kenar Kontrolü")
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .shadow(false)
        .focused(false)
        .visible(false)
        .inner_size(WIN_W, WIN_H)
        .build()
        .ok()
}

/// İki pencereyi gizli olarak önceden oluşturur; sayfalar yüklenmiş ve dinleyiciler hazır olur.
pub fn prewarm(app: &AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        for side in [Side::Left, Side::Right] {
            if app.get_webview_window(side.label()).is_none() {
                let _ = build(&app, side);
            }
        }
    });
}

fn show(app: &AppHandle, side: Side, sw: i32, sh: i32) {
    let scale = app
        .primary_monitor()
        .ok()
        .flatten()
        .map(|m| m.scale_factor())
        .unwrap_or(1.0);
    let (w, h) = (
        (WIN_W * scale).round() as i32,
        (WIN_H * scale).round() as i32,
    );
    let margin = (EDGE_MARGIN * scale).round() as i32;
    let x = if side == Side::Left {
        margin
    } else {
        sw - w - margin
    };
    let y = (sh - h) / 2;
    RECTS.lock().unwrap()[side.idx()] = Some((x, y, x + w, y + h));
    let app2 = app.clone();
    let place = move |win: WebviewWindow| {
        let _ = win.set_size(PhysicalSize::new(w as u32, h as u32));
        let _ = win.set_position(PhysicalPosition::new(x, y));
        let _ = app2.emit_to(side.label(), "hud-open", ());
        let _ = win.show();
    };
    match app.get_webview_window(side.label()) {
        Some(win) => place(win),
        None => {
            // Önceden ısıtma tamamlanmadıysa yedek yol.
            let app3 = app.clone();
            tauri::async_runtime::spawn(async move {
                if let Some(win) = build(&app3, side) {
                    thread::sleep(Duration::from_millis(300));
                    place(win);
                }
            });
        }
    }
}

fn hide(app: &AppHandle, side: Side) {
    RECTS.lock().unwrap()[side.idx()] = None;
    if let Some(w) = app.get_webview_window(side.label()) {
        let _ = app.emit_to(side.label(), "hud-close", ());
        let _ = w.hide();
    }
}

/// Komuttan (test veya kısayol) bir kapsülü doğrudan açar.
pub fn show_hud(app: &AppHandle, side: &str, sw: i32, sh: i32) {
    show(app, Side::parse(side), sw, sh);
}

/// Kenar izleyici. 25 ms'de bir tek `GetCursorPos` çağrısı yapar; iki taraf bağımsız izlenir.
pub fn start_watcher(app: AppHandle) {
    thread::spawn(move || {
        unsafe {
            // İmleç ve ekran ölçüleri fiziksel piksel olmalı (ölçekli ekranlarda doğru kenar için).
            SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        }
        let tick = Duration::from_millis(25);
        let mut shown = [false, false];
        let mut last_inside = [Instant::now(), Instant::now()];
        loop {
            thread::sleep(tick);
            if !ENABLED.load(Ordering::Relaxed) {
                for s in [Side::Left, Side::Right] {
                    if shown[s.idx()] {
                        hide(&app, s);
                        shown[s.idx()] = false;
                    }
                }
                continue;
            }
            let Some((x, y)) = cursor() else { continue };
            let (sw, sh) =
                unsafe { (GetSystemMetrics(SM_CXSCREEN), GetSystemMetrics(SM_CYSCREEN)) };
            let in_band = y > sh * 15 / 100 && y < sh * 85 / 100;
            let at_edge = |s: Side| -> bool {
                in_band
                    && match s {
                        Side::Left => x < HOTSPOT_PX,
                        Side::Right => x >= sw - HOTSPOT_PX,
                    }
            };
            let down = button_down();
            let fullscreen = if at_edge(Side::Left) || at_edge(Side::Right) {
                foreground_is_fullscreen(sw, sh)
            } else {
                false
            };

            for s in [Side::Left, Side::Right] {
                let i = s.idx();
                if !shown[i] {
                    if at_edge(s) && !down && !fullscreen {
                        show(&app, s, sw, sh);
                        shown[i] = true;
                        last_inside[i] = Instant::now();
                    }
                } else {
                    let inside =
                        RECTS
                            .lock()
                            .ok()
                            .and_then(|r| r[i])
                            .is_some_and(|(l, t, r, b)| {
                                x >= l - 24 && x <= r + 24 && y >= t - 24 && y <= b + 24
                            });
                    if inside || at_edge(s) || down {
                        last_inside[i] = Instant::now();
                    } else if last_inside[i].elapsed()
                        > Duration::from_millis(CLOSE_DELAY_MS.load(Ordering::Relaxed) as u64)
                    {
                        hide(&app, s);
                        shown[i] = false;
                    }
                }
            }
        }
    });
}
