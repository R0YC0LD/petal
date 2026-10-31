//! Açık pencereleri listeleme, öne getirme ve küçük önizleme alma.

use base64::Engine;
use serde::Serialize;
use std::ffi::c_void;
use windows::Win32::UI::WindowsAndMessaging::{
    GetForegroundWindow, PostMessageW, SW_MINIMIZE, WM_CLOSE,
};
use windows::{
    core::{BOOL, PWSTR},
    Win32::{
        Foundation::{CloseHandle, HWND, LPARAM, RECT},
        Graphics::{
            Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED},
            Gdi::{
                CreateCompatibleBitmap, CreateCompatibleDC, DeleteDC, DeleteObject, GetDC,
                GetDIBits, ReleaseDC, SelectObject, BITMAPINFO, BITMAPINFOHEADER, BI_RGB,
                DIB_RGB_COLORS,
            },
        },
        Storage::Xps::{PrintWindow, PRINT_WINDOW_FLAGS},
        System::Threading::{
            GetCurrentProcessId, OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32,
            PROCESS_QUERY_LIMITED_INFORMATION,
        },
        UI::{
            HiDpi::{SetThreadDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2},
            Input::KeyboardAndMouse::{keybd_event, KEYEVENTF_KEYUP, VK_D, VK_LWIN, VK_MENU},
            WindowsAndMessaging::{
                EnumWindows, GetWindow, GetWindowLongW, GetWindowRect, GetWindowTextW,
                GetWindowThreadProcessId, IsIconic, IsWindowVisible, SetForegroundWindow,
                ShowWindow, GWL_EXSTYLE, GW_OWNER, SW_RESTORE, WS_EX_TOOLWINDOW,
            },
        },
    },
};

#[derive(Serialize, Clone)]
pub struct WinInfo {
    pub hwnd: isize,
    pub title: String,
    /// Küçük harfli, .exe uzantısız işlem adı.
    pub exe: String,
    /// Çalıştırılabilir dosyanın tam yolu (simge ve sabitleme için).
    pub path: String,
    pub minimized: bool,
    /// Şu an ön plandaki pencere.
    pub active: bool,
}

fn exe_of(pid: u32) -> Option<(String, String)> {
    unsafe {
        let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buf = [0u16; 520];
        let mut len = buf.len() as u32;
        let ok =
            QueryFullProcessImageNameW(h, PROCESS_NAME_WIN32, PWSTR(buf.as_mut_ptr()), &mut len)
                .is_ok();
        let _ = CloseHandle(h);
        if !ok {
            return None;
        }
        let path = String::from_utf16_lossy(&buf[..len as usize]);
        let file = path.rsplit('\\').next()?.to_lowercase();
        Some((file.strip_suffix(".exe").unwrap_or(&file).to_string(), path))
    }
}

unsafe extern "system" fn collect(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let out = &mut *(lparam.0 as *mut Vec<WinInfo>);
    if !IsWindowVisible(hwnd).as_bool() {
        return BOOL(1);
    }
    if GetWindow(hwnd, GW_OWNER).is_ok_and(|o| !o.0.is_null()) {
        return BOOL(1);
    }
    if GetWindowLongW(hwnd, GWL_EXSTYLE) as u32 & WS_EX_TOOLWINDOW.0 != 0 {
        return BOOL(1);
    }
    let mut cloaked = 0u32;
    let _ = DwmGetWindowAttribute(
        hwnd,
        DWMWA_CLOAKED,
        &mut cloaked as *mut u32 as *mut c_void,
        4,
    );
    if cloaked != 0 {
        return BOOL(1);
    }
    let mut t = [0u16; 256];
    let n = GetWindowTextW(hwnd, &mut t);
    if n <= 0 {
        return BOOL(1);
    }
    let mut pid = 0u32;
    GetWindowThreadProcessId(hwnd, Some(&mut pid));
    if pid == GetCurrentProcessId() {
        return BOOL(1);
    }
    let Some((exe, path)) = exe_of(pid) else {
        return BOOL(1);
    };
    if matches!(
        exe.as_str(),
        "textinputhost" | "searchhost" | "startmenuexperiencehost" | "shellexperiencehost"
    ) {
        return BOOL(1);
    }
    out.push(WinInfo {
        hwnd: hwnd.0 as isize,
        title: String::from_utf16_lossy(&t[..n as usize]),
        exe,
        path,
        minimized: IsIconic(hwnd).as_bool(),
        active: false,
    });
    BOOL(1)
}

/// Üst düzey, görünür ve kullanıcıya ait pencereler (z sırasına göre, öndeki ilk).
pub fn list() -> Vec<WinInfo> {
    let mut out: Vec<WinInfo> = Vec::new();
    unsafe {
        let _ = EnumWindows(
            Some(collect),
            LPARAM(&mut out as *mut Vec<WinInfo> as isize),
        );
    }
    // Ön plandaki pencere (veya sahibi) etkin kabul edilir.
    let fg = unsafe { GetForegroundWindow() }.0 as isize;
    for w in out.iter_mut() {
        w.active = w.hwnd == fg;
    }
    out
}

/// Pencereyi simge durumuna küçültür.
pub fn minimize(hwnd: isize) {
    unsafe {
        let _ = ShowWindow(HWND(hwnd as *mut c_void), SW_MINIMIZE);
    }
}

/// Pencereyi öne getirir, simge durumundaysa geri yükler.
pub fn activate(hwnd: isize) {
    let h = HWND(hwnd as *mut c_void);
    unsafe {
        if IsIconic(h).as_bool() {
            let _ = ShowWindow(h, SW_RESTORE);
        }
        // Windows, ön plan kilidi nedeniyle arka plandaki süreçlerin odaklanmasını engeller.
        // Alt tuşuna kısa basış bu kilidi açar.
        keybd_event(VK_MENU.0 as u8, 0, Default::default(), 0);
        let _ = SetForegroundWindow(h);
        keybd_event(VK_MENU.0 as u8, 0, KEYEVENTF_KEYUP, 0);
    }
}

/// Pencerenin küçültülmüş PNG önizlemesi (data URL). Simge durumundaki pencereler için `None`.
pub fn preview(hwnd: isize, max_w: u32) -> Option<String> {
    let h = HWND(hwnd as *mut c_void);
    unsafe {
        SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        if IsIconic(h).as_bool() {
            return None;
        }
        let mut rc = RECT::default();
        GetWindowRect(h, &mut rc).ok()?;
        let (w, ht) = ((rc.right - rc.left).max(1), (rc.bottom - rc.top).max(1));
        if w > 8000 || ht > 8000 {
            return None;
        }
        let screen = GetDC(None);
        let mem = CreateCompatibleDC(Some(screen));
        let bmp = CreateCompatibleBitmap(screen, w, ht);
        let old = SelectObject(mem, bmp.into());
        let printed = PrintWindow(h, mem, PRINT_WINDOW_FLAGS(2)).as_bool();

        let mut bi = BITMAPINFO::default();
        bi.bmiHeader = BITMAPINFOHEADER {
            biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: w,
            biHeight: -ht,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB.0,
            ..Default::default()
        };
        let mut px = vec![0u8; (w * ht * 4) as usize];
        let lines = GetDIBits(
            mem,
            bmp,
            0,
            ht as u32,
            Some(px.as_mut_ptr() as *mut c_void),
            &mut bi,
            DIB_RGB_COLORS,
        );

        SelectObject(mem, old);
        let _ = DeleteObject(bmp.into());
        let _ = DeleteDC(mem);
        ReleaseDC(None, screen);
        if !printed || lines == 0 {
            return None;
        }

        // Kutu ortalamalı küçültme.
        let scale = (w as f32 / max_w as f32).max(1.0);
        let (tw, th) = (
            ((w as f32 / scale) as u32).max(1),
            ((ht as f32 / scale) as u32).max(1),
        );
        let mut out = vec![0u8; (tw * th * 4) as usize];
        for ty in 0..th {
            for tx in 0..tw {
                let (sx0, sy0) = ((tx as f32 * scale) as i32, (ty as f32 * scale) as i32);
                let (sx1, sy1) = (
                    (((tx + 1) as f32 * scale) as i32).min(w).max(sx0 + 1),
                    (((ty + 1) as f32 * scale) as i32).min(ht).max(sy0 + 1),
                );
                let (mut r, mut g, mut b, mut n) = (0u32, 0u32, 0u32, 0u32);
                for sy in sy0..sy1 {
                    for sx in sx0..sx1 {
                        let i = ((sy * w + sx) * 4) as usize;
                        b += px[i] as u32;
                        g += px[i + 1] as u32;
                        r += px[i + 2] as u32;
                        n += 1;
                    }
                }
                let o = ((ty * tw + tx) * 4) as usize;
                out[o] = (r / n) as u8;
                out[o + 1] = (g / n) as u8;
                out[o + 2] = (b / n) as u8;
                out[o + 3] = 255;
            }
        }
        // Tamamen siyah çıktı (PrintWindow desteklenmedi) önizleme sayılmaz.
        if out.chunks(4).all(|p| p[0] < 4 && p[1] < 4 && p[2] < 4) {
            return None;
        }
        let mut png_bytes = Vec::new();
        {
            let mut enc = png::Encoder::new(&mut png_bytes, tw, th);
            enc.set_color(png::ColorType::Rgba);
            enc.set_depth(png::BitDepth::Eight);
            enc.set_compression(png::Compression::Fast);
            let mut wr = enc.write_header().ok()?;
            wr.write_image_data(&out).ok()?;
        }
        Some(format!(
            "data:image/png;base64,{}",
            base64::engine::general_purpose::STANDARD.encode(png_bytes)
        ))
    }
}

/// Pencereye kapatma iletisi gönderir; uygulama kapanmayı reddedebilir (kaydedilmemiş iş vb.).
pub fn close(hwnd: isize) -> bool {
    let h = HWND(hwnd as *mut c_void);
    unsafe { PostMessageW(Some(h), WM_CLOSE, Default::default(), Default::default()).is_ok() }
}

/// Win+D: açık pencereleri küçültür; ikinci basışta geri getirir.
pub fn toggle_desktop() {
    unsafe {
        keybd_event(VK_LWIN.0 as u8, 0, Default::default(), 0);
        keybd_event(VK_D.0 as u8, 0, Default::default(), 0);
        keybd_event(VK_D.0 as u8, 0, KEYEVENTF_KEYUP, 0);
        keybd_event(VK_LWIN.0 as u8, 0, KEYEVENTF_KEYUP, 0);
    }
}
