//! Pencere olayları: yeni pencere, kapanan pencere, ön plan değişimi, simge durumu.
//! Yoklama yerine Windows olay kancası (`SetWinEventHook`) kullanılır; olaylar 60 ms içinde
//! birleştirilip arayüze tek bir `windows-changed` olayı olarak iletilir.

use std::{
    sync::{
        mpsc::{channel, Sender},
        Mutex, OnceLock,
    },
    thread,
    time::Duration,
};
use tauri::{AppHandle, Emitter};
use windows::Win32::{
    Foundation::HWND,
    UI::{
        Accessibility::{SetWinEventHook, HWINEVENTHOOK},
        WindowsAndMessaging::{
            DispatchMessageW, GetAncestor, GetMessageW, TranslateMessage, EVENT_OBJECT_DESTROY,
            EVENT_OBJECT_HIDE, EVENT_OBJECT_SHOW, EVENT_SYSTEM_FOREGROUND,
            EVENT_SYSTEM_MINIMIZEEND, EVENT_SYSTEM_MINIMIZESTART, GA_ROOT, MSG, OBJID_WINDOW,
            WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS,
        },
    },
};

static TX: OnceLock<Mutex<Sender<()>>> = OnceLock::new();

unsafe extern "system" fn on_event(
    _hook: HWINEVENTHOOK,
    _event: u32,
    hwnd: HWND,
    id_object: i32,
    id_child: i32,
    _thread: u32,
    _time: u32,
) {
    // Yalnızca pencerenin kendisiyle ilgili olaylar (alt öğeler değil).
    if id_object != OBJID_WINDOW.0 || id_child != 0 {
        return;
    }
    // Yalnızca üst düzey pencereler (menü, ipucu, alt pencere olayları atlanır).
    if hwnd.0.is_null() || GetAncestor(hwnd, GA_ROOT) != hwnd {
        return;
    }
    if let Some(tx) = TX.get() {
        if let Ok(tx) = tx.lock() {
            let _ = tx.send(());
        }
    }
}

pub fn start(app: AppHandle) {
    let (tx, rx) = channel::<()>();
    let _ = TX.set(Mutex::new(tx));

    // Birleştirici: olay yağmurunu tek bir bildirime indirger.
    thread::Builder::new()
        .name("petal-winevents-debounce".into())
        .spawn(move || {
            while rx.recv().is_ok() {
                thread::sleep(Duration::from_millis(120));
                while rx.try_recv().is_ok() {}
                crate::fullscreen::check(&app);
                let _ = app.emit("windows-changed", ());
            }
        })
        .ok();

    // Kanca iş parçacığı: kancalar, bu iş parçacığındaki ileti döngüsü üzerinden çalışır.
    thread::Builder::new()
        .name("petal-winevents".into())
        .spawn(|| unsafe {
            let flags = WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS;
            for (lo, hi) in [
                (EVENT_SYSTEM_FOREGROUND, EVENT_SYSTEM_FOREGROUND),
                (EVENT_SYSTEM_MINIMIZESTART, EVENT_SYSTEM_MINIMIZEEND),
                (EVENT_OBJECT_DESTROY, EVENT_OBJECT_HIDE),
            ] {
                let _ = SetWinEventHook(lo, hi, None, Some(on_event), 0, 0, flags);
            }
            // EVENT_OBJECT_SHOW (0x8002) yukarıdaki aralıkta (0x8001-0x8003) yer alır.
            let _ = EVENT_OBJECT_SHOW;
            let mut msg = MSG::default();
            while GetMessageW(&mut msg, None, 0, 0).as_bool() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        })
        .ok();
}
