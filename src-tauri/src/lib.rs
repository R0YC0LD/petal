mod audio;
mod brightness;
mod fullscreen;
mod gpu;
mod hud;
mod media;
mod store;
mod sysmon;
mod taskbar;
mod tray;
mod winevents;
mod winlist;

use base64::Engine;
use serde::Serialize;
use serde_json::{json, Value};
use std::{fs, path::PathBuf, process::Command, sync::Mutex};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, WebviewUrl, WebviewWindow,
    WebviewWindowBuilder,
};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

const NO_WINDOW: u32 = 0x0800_0000;
const NOTCH_MARGIN: f64 = 0.0;
const DOCK_MARGIN: f64 = 6.0;

#[derive(Serialize)]
struct StartApp {
    name: String,
    id: String,
}

fn settings_path(app: &AppHandle) -> PathBuf {
    let dir = app
        .path()
        .app_config_dir()
        .unwrap_or_else(|_| PathBuf::from("."));
    let _ = fs::create_dir_all(&dir);
    dir.join("settings.json")
}

fn config_dir(app: &AppHandle) -> PathBuf {
    app.path()
        .app_config_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
}

fn apply_taskbar(app: &AppHandle, value: &Value) {
    let dir = config_dir(app);
    if value
        .get("hideTaskbar")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        taskbar::hide(&dir);
    } else if taskbar::is_hidden_by_us(&dir) {
        taskbar::restore(&dir);
    }
}

static SEARCH_HOTKEY: Mutex<Option<String>> = Mutex::new(None);

#[tauri::command]
fn search_hotkey() -> Option<String> {
    SEARCH_HOTKEY.lock().unwrap().clone()
}

fn apply_runtime(v: &Value) {
    use std::sync::atomic::Ordering::Relaxed;
    fullscreen::HIDE_ENABLED.store(
        v.get("hideInFullscreen")
            .and_then(Value::as_bool)
            .unwrap_or(true),
        Relaxed,
    );
    hud::ENABLED.store(
        v.get("hudEnabled").and_then(Value::as_bool).unwrap_or(true),
        Relaxed,
    );
    hud::CLOSE_DELAY_MS.store(
        v.get("closeDelay").and_then(Value::as_u64).unwrap_or(300) as u32,
        Relaxed,
    );
    sysmon::INTERVAL_MS.store(
        v.get("statsInterval")
            .and_then(Value::as_u64)
            .unwrap_or(1000) as u32,
        Relaxed,
    );
}

fn read_settings(app: &AppHandle) -> Value {
    let (v, note) = store::load(&settings_path(app));
    if let Some(n) = note {
        eprintln!("petal: {n}");
    }
    v
}

fn write_settings(app: &AppHandle, value: &Value) -> Result<(), String> {
    store::save(&settings_path(app), value).map_err(|e| e.to_string())?;
    let _ = app.emit("settings-changed", value);
    apply_taskbar(app, value);
    apply_runtime(value);
    let want = value
        .get("launchAtLogin")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let auto = app.autolaunch();
    if auto.is_enabled().unwrap_or(false) != want {
        let _ = if want { auto.enable() } else { auto.disable() };
    }
    Ok(())
}

#[tauri::command]
fn load_settings(app: AppHandle) -> Value {
    read_settings(&app)
}

#[tauri::command]
fn save_settings(app: AppHandle, value: Value) -> Result<(), String> {
    write_settings(&app, &value)
}

#[tauri::command]
fn get_stats() -> sysmon::Stats {
    sysmon::latest()
}

#[tauri::command]
async fn get_media() -> Option<media::MediaInfo> {
    tauri::async_runtime::spawn_blocking(media::current)
        .await
        .ok()
        .flatten()
}

#[tauri::command]
async fn media_control(action: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || media::control(&action))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn get_audio_peak() -> Option<f32> {
    tauri::async_runtime::spawn_blocking(media::audio_peak)
        .await
        .ok()
        .flatten()
}

#[tauri::command]
async fn list_windows() -> Vec<winlist::WinInfo> {
    tauri::async_runtime::spawn_blocking(winlist::list)
        .await
        .unwrap_or_default()
}

#[tauri::command]
fn activate_window(hwnd: isize) {
    winlist::activate(hwnd);
}

#[tauri::command]
async fn window_preview(hwnd: isize) -> Option<String> {
    tauri::async_runtime::spawn_blocking(move || winlist::preview(hwnd, 360))
        .await
        .ok()
        .flatten()
}

#[tauri::command]
async fn list_tray_apps() -> Vec<tray::TrayApp> {
    tauri::async_runtime::spawn_blocking(tray::list)
        .await
        .unwrap_or_default()
}

/// Uygulamanın penceresi varsa öne getirir, yoksa uygulamayı başlatır.
#[tauri::command]
fn tray_open(path: String, exe: String) -> Result<(), String> {
    if let Some(w) = winlist::list().into_iter().find(|w| w.exe == exe) {
        winlist::activate(w.hwnd);
        return Ok(());
    }
    let mut c = Command::new("cmd");
    c.args(["/C", "start", "", &path]);
    #[cfg(windows)]
    c.creation_flags(NO_WINDOW);
    c.spawn().map(|_| ()).map_err(|e| e.to_string())
}

/// Uygulamanın pencerelerine kapatma iletisi gönderir (süreci zorla sonlandırmaz).
#[tauri::command]
fn tray_close(exe: String) -> usize {
    winlist::list()
        .into_iter()
        .filter(|w| w.exe == exe)
        .filter(|w| winlist::close(w.hwnd))
        .count()
}

/// Masaüstünü göster / geri al (Win+D).
#[tauri::command]
fn focus_notch(app: AppHandle) {
    if let Some(w) = app.get_webview_window("notch") {
        let _ = w.set_focus();
    }
}

#[tauri::command]
fn minimize_window(hwnd: isize) {
    winlist::minimize(hwnd);
}

#[tauri::command]
fn show_desktop() {
    winlist::toggle_desktop();
}

#[tauri::command]
fn open_hud(app: AppHandle, side: String) {
    use windows::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};
    let (sw, sh) = unsafe { (GetSystemMetrics(SM_CXSCREEN), GetSystemMetrics(SM_CYSCREEN)) };
    hud::show_hud(&app, if side == "right" { "right" } else { "left" }, sw, sh);
}

#[tauri::command]
async fn get_volume() -> Option<audio::Volume> {
    tauri::async_runtime::spawn_blocking(audio::get)
        .await
        .ok()
        .flatten()
}

#[tauri::command]
async fn media_seek(seconds: f64) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || media::seek(seconds))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
fn set_volume(level: f32) {
    audio::set(level);
}

#[tauri::command]
fn set_mute(muted: bool) {
    audio::mute(muted);
}

#[tauri::command]
async fn get_brightness() -> Option<u32> {
    tauri::async_runtime::spawn_blocking(brightness::get)
        .await
        .ok()
        .flatten()
}

#[tauri::command]
fn set_brightness(percent: u32) {
    brightness::set(percent);
}

#[tauri::command]
async fn get_running() -> Vec<String> {
    tauri::async_runtime::spawn_blocking(|| {
        let mut v: Vec<String> = winlist::list().into_iter().map(|w| w.exe).collect();
        v.sort();
        v.dedup();
        v
    })
    .await
    .unwrap_or_default()
}

#[tauri::command]
fn set_visible(app: AppHandle, label: String, visible: bool) {
    fullscreen::set_visible(&app, &label, visible);
}

fn place_top(win: &WebviewWindow, w: f64, h: f64) -> tauri::Result<()> {
    let Some(mon) = win.primary_monitor()? else {
        return Ok(());
    };
    let scale = mon.scale_factor();
    let screen_w = mon.size().width as f64 / scale;
    let x0 = mon.position().x as f64 / scale;
    let y0 = mon.position().y as f64 / scale;
    win.set_size(LogicalSize::new(w, h))?;
    win.set_position(LogicalPosition::new(
        x0 + (screen_w - w) / 2.0,
        y0 + NOTCH_MARGIN,
    ))
}

fn place_bottom(win: &WebviewWindow, w: f64, h: f64) -> tauri::Result<()> {
    let Some(mon) = win.primary_monitor()? else {
        return Ok(());
    };
    let scale = mon.scale_factor();
    let work = mon.work_area();
    let x0 = work.position.x as f64 / scale;
    let width = work.size.width as f64 / scale;
    let bottom = (work.position.y as f64 + work.size.height as f64) / scale;
    win.set_size(LogicalSize::new(w, h))?;
    win.set_position(LogicalPosition::new(
        x0 + (width - w) / 2.0,
        bottom - h - DOCK_MARGIN,
    ))
}

#[tauri::command]
fn resize_notch(win: WebviewWindow, width: f64, height: f64) -> Result<(), String> {
    place_top(&win, width, height).map_err(|e| e.to_string())
}

#[tauri::command]
fn resize_dock(win: WebviewWindow, width: f64, height: f64) -> Result<(), String> {
    place_bottom(&win, width, height).map_err(|e| e.to_string())
}

#[tauri::command]
fn launch_app(kind: String, target: String) -> Result<(), String> {
    let mut cmd = if kind == "appid" {
        let mut c = Command::new("explorer.exe");
        c.arg(format!("shell:AppsFolder\\{target}"));
        c
    } else {
        let mut c = Command::new("cmd");
        c.args(["/C", "start", "", &target]);
        c
    };
    #[cfg(windows)]
    cmd.creation_flags(NO_WINDOW);
    cmd.spawn().map(|_| ()).map_err(|e| e.to_string())
}

fn icon_for(path: &str) -> Option<String> {
    let icon = match file_icon_provider::get_file_icon(path, 96) {
        Ok(i) => i,
        Err(e) => {
            eprintln!("icon error for {path}: {e:?}");
            return None;
        }
    };
    let mut out = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut out, icon.width, icon.height);
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        let mut w = enc.write_header().ok()?;
        w.write_image_data(&icon.pixels).ok()?;
    }
    Some(format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(out)
    ))
}

fn store_icon(appid: &str) -> Option<String> {
    let family = appid.split('!').next()?;
    if !family
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-'))
    {
        return None;
    }
    let mut cmd = Command::new("powershell");
    cmd.args([
        "-NoProfile",
        "-Command",
        &format!("(Get-AppxPackage | Where-Object {{ $_.PackageFamilyName -eq '{family}' }} | Select-Object -First 1).InstallLocation"),
    ]);
    #[cfg(windows)]
    cmd.creation_flags(NO_WINDOW);
    let out = cmd.output().ok()?;
    let root = PathBuf::from(String::from_utf8_lossy(&out.stdout).trim());
    let manifest = fs::read_to_string(root.join("AppxManifest.xml")).ok()?;
    let attr = |name: &str| -> Option<String> {
        let start = manifest.find(&format!("{name}=\""))? + name.len() + 2;
        let end = manifest[start..].find('"')? + start;
        Some(manifest[start..end].replace('/', "\\"))
    };
    let logo = attr("Square44x44Logo").or_else(|| attr("Square150x150Logo"))?;
    let logo_path = root.join(&logo);
    let dir = logo_path.parent()?;
    let stem = logo_path.file_stem()?.to_string_lossy().to_lowercase();
    let best = fs::read_dir(dir)
        .ok()?
        .filter_map(|e| e.ok())
        .filter(|e| {
            let n = e.file_name().to_string_lossy().to_lowercase();
            n.starts_with(&stem)
                && n.ends_with(".png")
                && !n.contains("contrast")
                && !n.contains("lightunplated")
        })
        .max_by_key(|e| e.metadata().map(|m| m.len()).unwrap_or(0))?;
    let bytes = fs::read(best.path()).ok()?;
    Some(format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(bytes)
    ))
}

type IconReq = (String, std::sync::mpsc::Sender<Option<String>>);
static ICON_TX: std::sync::OnceLock<Mutex<std::sync::mpsc::Sender<IconReq>>> =
    std::sync::OnceLock::new();

fn icon_via_worker(path: &str) -> Option<String> {
    let tx = ICON_TX.get_or_init(|| {
        let (tx, rx) = std::sync::mpsc::channel::<IconReq>();
        std::thread::spawn(move || {
            for (p, reply) in rx {
                let _ = reply.send(icon_for(&p));
            }
        });
        Mutex::new(tx)
    });
    let (rtx, rrx) = std::sync::mpsc::channel();
    tx.lock().ok()?.send((path.to_string(), rtx)).ok()?;
    rrx.recv().ok().flatten()
}

#[tauri::command]
async fn get_icon(kind: String, target: String) -> Option<String> {
    tauri::async_runtime::spawn_blocking(move || {
        if kind == "appid" {
            store_icon(&target).or_else(|| icon_for(&format!("shell:AppsFolder\\{target}")))
        } else {
            (0..4).find_map(|i| {
                if i > 0 {
                    std::thread::sleep(std::time::Duration::from_millis(150));
                }
                icon_via_worker(&target)
            })
        }
    })
    .await
    .ok()
    .flatten()
}

#[tauri::command]
fn add_dock_path(app: AppHandle, path: String) -> Result<(), String> {
    let p = PathBuf::from(&path);
    let name = p
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| path.clone());
    let mut settings = read_settings(&app);
    let apps = settings
        .as_object_mut()
        .ok_or("bozuk ayar")?
        .entry("dockApps")
        .or_insert_with(|| json!([]));
    if let Some(list) = apps.as_array_mut() {
        if list.iter().any(|a| a["target"] == path) {
            return Ok(());
        }
        let id = format!("{}-{}", name.to_lowercase(), list.len());
        list.push(json!({"id": id, "name": name, "kind": "path", "target": path}));
    }
    write_settings(&app, &settings)
}

#[tauri::command]
async fn list_start_apps() -> Vec<StartApp> {
    tauri::async_runtime::spawn_blocking(|| {
        let mut cmd = Command::new("powershell");
        cmd.args([
            "-NoProfile",
            "-Command",
            "[Console]::OutputEncoding=[Text.Encoding]::UTF8; Get-StartApps | Sort-Object Name | ConvertTo-Json -Compress",
        ]);
        #[cfg(windows)]
        cmd.creation_flags(NO_WINDOW);
        let Ok(out) = cmd.output() else { return vec![] };
        let text = String::from_utf8_lossy(&out.stdout);
        let Ok(v) = serde_json::from_str::<Value>(text.trim()) else { return vec![] };
        let items = match v {
            Value::Array(a) => a,
            o @ Value::Object(_) => vec![o],
            _ => vec![],
        };
        items
            .iter()
            .filter_map(|i| {
                Some(StartApp { name: i["Name"].as_str()?.to_string(), id: i["AppID"].as_str()?.to_string() })
            })
            .collect()
    })
    .await
    .unwrap_or_default()
}

#[tauri::command]
fn open_settings(app: AppHandle) {
    show_settings(&app);
}

fn show_settings(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("settings") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
        return;
    }
    let app = app.clone();
    tauri::async_runtime::spawn(async move {
        let _ = WebviewWindowBuilder::new(&app, "settings", WebviewUrl::default())
            .title("Petal Ayarları")
            .inner_size(820.0, 600.0)
            .min_inner_size(700.0, 480.0)
            .center()
            .theme(Some(tauri::Theme::Dark))
            .build();
    });
}

fn fallback_config_dir() -> PathBuf {
    PathBuf::from(std::env::var("APPDATA").unwrap_or_default()).join("com.by599.petal")
}

pub fn run() {
    if std::env::args().any(|a| a == "--restore-taskbar") {
        taskbar::restore(&fallback_config_dir());
        return;
    }
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let dir = fallback_config_dir();
        if taskbar::is_hidden_by_us(&dir) {
            taskbar::restore(&dir);
        }
        default_hook(info);
    }));
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            show_settings(app)
        }))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .invoke_handler(tauri::generate_handler![
            load_settings,
            save_settings,
            get_stats,
            get_running,
            list_tray_apps,
            tray_open,
            tray_close,
            show_desktop,
            minimize_window,
            focus_notch,
            search_hotkey,
            open_hud,
            get_volume,
            media_seek,
            set_volume,
            set_mute,
            get_brightness,
            set_brightness,
            list_windows,
            activate_window,
            window_preview,
            get_media,
            media_control,
            get_audio_peak,
            set_visible,
            resize_notch,
            resize_dock,
            launch_app,
            get_icon,
            add_dock_path,
            list_start_apps,
            open_settings
        ])
        .setup(|app| {
            let handle = app.handle().clone();
            let initial = read_settings(&handle);
            apply_taskbar(&handle, &initial);
            apply_runtime(&initial);
            hud::prewarm(&handle);
            hud::start_watcher(handle.clone());
            sysmon::start(handle.clone());
            winevents::start(handle.clone());
            {
                use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};
                // Hızlı arama kısayolu. Başka bir uygulama (ör. PowerToys) aynı kısayolu
                // tutuyorsa sıradaki alternatif denenir; kullanılan kısayol Ayarlar'da gösterilir.
                let configured = initial
                    .get("searchHotkey")
                    .and_then(Value::as_str)
                    .map(str::to_string);
                let mut candidates: Vec<String> = configured.into_iter().collect();
                for c in [
                    "Alt+Space",
                    "Ctrl+Alt+Space",
                    "Ctrl+Shift+Space",
                    "Alt+Shift+Space",
                ] {
                    if !candidates.iter().any(|x| x == c) {
                        candidates.push(c.to_string());
                    }
                }
                for cand in candidates {
                    let h = handle.clone();
                    let ok = handle
                        .global_shortcut()
                        .on_shortcut(cand.as_str(), move |_app, _sc, ev| {
                            if ev.state == ShortcutState::Pressed {
                                fullscreen::set_visible(&h, "notch", true);
                                if let Some(w) = h.get_webview_window("notch") {
                                    let _ = w.set_focus();
                                }
                                let _ = h.emit_to("notch", "search-toggle", ());
                            }
                        })
                        .is_ok();
                    if ok {
                        *SEARCH_HOTKEY.lock().unwrap() = Some(cand);
                        break;
                    }
                }
            }
            if let Some(notch) = app.get_webview_window("notch") {
                place_top(&notch, 190.0, 38.0)?;
            }
            if let Some(dock) = app.get_webview_window("dock") {
                place_bottom(&dock, 300.0, 110.0)?;
            }

            let open = MenuItem::with_id(app, "settings", "Ayarlar", true, None::<&str>)?;
            let bar = MenuItem::with_id(
                app,
                "taskbar",
                "Windows görev çubuğunu geri getir",
                true,
                None::<&str>,
            )?;
            let quit = MenuItem::with_id(app, "quit", "Çıkış", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&open, &bar, &quit])?;
            let mut tray = TrayIconBuilder::new()
                .tooltip("Petal")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app, ev| match ev.id.as_ref() {
                    "settings" => show_settings(app),
                    "taskbar" => {
                        let mut v = read_settings(app);
                        if let Some(o) = v.as_object_mut() {
                            o.insert("hideTaskbar".into(), Value::Bool(false));
                        }
                        let _ = write_settings(app, &v);
                        taskbar::restore(&config_dir(app));
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(move |_, ev| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = ev
                    {
                        show_settings(&handle);
                    }
                });
            if let Some(icon) = app.default_window_icon() {
                tray = tray.icon(icon.clone());
            }
            tray.build(app)?;
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("Petal başlatılamadı")
        .run(|app, event| {
            if let tauri::RunEvent::Exit = event {
                let dir = config_dir(app);
                if taskbar::is_hidden_by_us(&dir) {
                    taskbar::restore(&dir);
                }
            }
        });
}
