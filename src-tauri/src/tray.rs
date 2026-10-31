//! Arka plan (bildirim alanı) uygulamaları paneli için veri kaynağı.
//!
//! Windows 11'de gizli simgeler XAML ile çizilir ve görev çubuğu gizliyken başka bir sürecin
//! okuyabileceği bir arayüz sunmaz. Bu yüzden kaynak olarak Windows'un kendi kayıt defteri
//! listesi kullanılır (`HKCU\Control Panel\NotifyIconSettings`, "Diğer sistem tepsisi simgeleri"
//! ayarının veri kaynağı) ve yalnızca şu an çalışan uygulamalar gösterilir.
//! Her uygulamanın kendi tepsi menüsü (ör. "Çıkış") okunamaz; bunun yerine uygulamayı öne
//! getirme/açma ve pencerelerini kapatma sunulur.

use serde::Serialize;
use std::{collections::HashSet, path::PathBuf};
use windows::{
    core::{GUID, PWSTR},
    Win32::{
        Foundation::CloseHandle,
        System::{
            Com::CoTaskMemFree,
            Diagnostics::ToolHelp::{
                CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
                TH32CS_SNAPPROCESS,
            },
        },
        UI::Shell::{SHGetKnownFolderPath, KF_FLAG_DEFAULT},
    },
};
use winreg::{enums::HKEY_CURRENT_USER, RegKey};

#[derive(Serialize, Clone)]
pub struct TrayApp {
    pub name: String,
    pub path: String,
    /// Küçük harfli, .exe uzantısız işlem adı (pencere listesiyle eşleşmek için).
    pub exe: String,
}

fn parse_guid(s: &str) -> Option<GUID> {
    let h = s.trim_matches(|c| c == '{' || c == '}').replace('-', "");
    if h.len() != 32 {
        return None;
    }
    let p = |a: usize, b: usize| u64::from_str_radix(&h[a..b], 16).ok();
    Some(GUID {
        data1: p(0, 8)? as u32,
        data2: p(8, 12)? as u16,
        data3: p(12, 16)? as u16,
        data4: [
            p(16, 18)? as u8,
            p(18, 20)? as u8,
            p(20, 22)? as u8,
            p(22, 24)? as u8,
            p(24, 26)? as u8,
            p(26, 28)? as u8,
            p(28, 30)? as u8,
            p(30, 32)? as u8,
        ],
    })
}

fn known_folder(g: &GUID) -> Option<String> {
    unsafe {
        let p: PWSTR = SHGetKnownFolderPath(g, KF_FLAG_DEFAULT, None).ok()?;
        let s = p.to_string().ok();
        CoTaskMemFree(Some(p.0 as *const _));
        s
    }
}

/// `{KNOWN-FOLDER-GUID}\alt\yol.exe` biçimini gerçek yola çevirir.
pub fn expand_path(raw: &str) -> Option<PathBuf> {
    if let Some(rest) = raw.strip_prefix('{') {
        let (guid, tail) = rest.split_once('}')?;
        let base = known_folder(&parse_guid(guid)?)?;
        return Some(PathBuf::from(base).join(tail.trim_start_matches('\\')));
    }
    Some(PathBuf::from(raw))
}

fn running_exes() -> HashSet<String> {
    let mut out = HashSet::new();
    unsafe {
        let Ok(snap) = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) else {
            return out;
        };
        let mut e = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };
        if Process32FirstW(snap, &mut e).is_ok() {
            loop {
                let len = e
                    .szExeFile
                    .iter()
                    .position(|&c| c == 0)
                    .unwrap_or(e.szExeFile.len());
                out.insert(String::from_utf16_lossy(&e.szExeFile[..len]).to_lowercase());
                if Process32NextW(snap, &mut e).is_err() {
                    break;
                }
            }
        }
        let _ = CloseHandle(snap);
    }
    out
}

fn pretty(stem: &str) -> String {
    let stem = stem.split('.').next().unwrap_or(stem);
    let mut c = stem.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

/// Bildirim alanında simgesi olan ve şu an çalışan uygulamalar.
pub fn list() -> Vec<TrayApp> {
    let running = running_exes();
    let Ok(key) =
        RegKey::predef(HKEY_CURRENT_USER).open_subkey(r"Control Panel\NotifyIconSettings")
    else {
        return vec![];
    };
    let mut seen = HashSet::new();
    let mut apps = Vec::new();
    for sub in key.enum_keys().flatten() {
        let Ok(k) = key.open_subkey(&sub) else {
            continue;
        };
        let Ok(raw) = k.get_value::<String, _>("ExecutablePath") else {
            continue;
        };
        let Some(path) = expand_path(&raw) else {
            continue;
        };
        let Some(file) = path.file_name().map(|f| f.to_string_lossy().to_lowercase()) else {
            continue;
        };
        // Kabuk ve yardımcı süreçler gösterilmez.
        if matches!(
            file.as_str(),
            "explorer.exe" | "jucheck.exe" | "sihost.exe" | "petal.exe"
        ) {
            continue;
        }
        if !running.contains(&file) || !path.exists() || !seen.insert(file.clone()) {
            continue;
        }
        let stem = file.trim_end_matches(".exe").to_string();
        let tip: String = k.get_value("InitialTooltip").unwrap_or_default();
        let name = if tip.trim().is_empty() || tip.contains('\n') || tip.len() > 28 {
            pretty(&stem)
        } else {
            tip.trim().to_string()
        };
        // Aynı görünen adı taşıyan yardımcı süreçleri (ör. iki NVIDIA hizmeti) tek satırda göster.
        if apps.iter().any(|a: &TrayApp| a.name == name) {
            continue;
        }
        apps.push(TrayApp {
            name,
            path: path.to_string_lossy().to_string(),
            exe: stem,
        });
    }
    apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    apps
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_known_folder_guid() {
        let g = parse_guid("{F38BF404-1D43-42F2-9305-67DE0B28FC23}").unwrap();
        assert_eq!(g.data1, 0xF38BF404);
        assert_eq!(g.data4[7], 0x23);
    }

    #[test]
    fn expands_windows_folder() {
        let p = expand_path(r"{F38BF404-1D43-42F2-9305-67DE0B28FC23}\explorer.exe").unwrap();
        assert!(p
            .to_string_lossy()
            .to_lowercase()
            .ends_with(r"\windows\explorer.exe"));
    }

    #[test]
    fn plain_paths_pass_through() {
        assert_eq!(
            expand_path(r"C:\a\b.exe").unwrap(),
            PathBuf::from(r"C:\a\b.exe")
        );
    }

    #[test]
    fn lists_without_panicking() {
        let _ = list();
    }
}
