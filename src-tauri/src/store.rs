//! Ayar dosyası için güvenli okuma/yazma: geçici dosyaya yaz, yedek al, sonra yer değiştir.
//! Dosya bozulursa son yedekten kurtarır; bozuk dosya silinmez, `.corrupt` olarak saklanır.

use serde_json::{json, Value};
use std::{
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

fn sibling(path: &Path, ext: &str) -> PathBuf {
    let mut name = path
        .file_name()
        .map(|n| n.to_os_string())
        .unwrap_or_default();
    name.push(ext);
    path.with_file_name(name)
}

fn parse(path: &Path) -> Option<Value> {
    let text = fs::read_to_string(path).ok()?;
    // Windows editörleri dosyanın başına BOM ekleyebilir.
    let v: Value = serde_json::from_str(text.trim_start_matches('\u{feff}')).ok()?;
    v.is_object().then_some(v)
}

/// Ayarları okur. Ana dosya bozuksa yedeği dener, ikisi de yoksa boş nesne döner.
pub fn load(path: &Path) -> (Value, Option<&'static str>) {
    if !path.exists() {
        return (json!({}), None);
    }
    if let Some(v) = parse(path) {
        return (v, None);
    }
    let _ = fs::rename(path, sibling(path, ".corrupt"));
    if let Some(v) = parse(&sibling(path, ".bak")) {
        return (v, Some("ayar dosyası bozuktu, yedekten kurtarıldı"));
    }
    (
        json!({}),
        Some("ayar dosyası bozuktu ve yedek bulunamadı, varsayılanlar kullanıldı"),
    )
}

/// Atomik yazım: `.tmp` dosyasına yaz ve diske boşalt, mevcut dosyayı `.bak` yap, `.tmp` dosyasını yerine taşı.
pub fn save(path: &Path, value: &Value) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    let tmp = sibling(path, ".tmp");
    let text = serde_json::to_string_pretty(value)?;
    {
        let mut f = File::create(&tmp)?;
        f.write_all(text.as_bytes())?;
        f.sync_all()?;
    }
    if path.exists() && parse(path).is_some() {
        let _ = fs::copy(path, sibling(path, ".bak"));
    }
    fs::rename(&tmp, path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("petal-test-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&d);
        fs::create_dir_all(&d).unwrap();
        d.join("settings.json")
    }

    #[test]
    fn roundtrip() {
        let p = scratch("roundtrip");
        save(&p, &json!({"a": 1})).unwrap();
        let (v, note) = load(&p);
        assert_eq!(v["a"], 1);
        assert!(note.is_none());
    }

    #[test]
    fn missing_file_gives_defaults() {
        let p = scratch("missing");
        let (v, note) = load(&p);
        assert_eq!(v, json!({}));
        assert!(note.is_none());
    }

    #[test]
    fn corrupt_file_recovers_from_backup() {
        let p = scratch("corrupt");
        save(&p, &json!({"v": 1})).unwrap();
        save(&p, &json!({"v": 2})).unwrap();
        fs::write(&p, "{ bozuk").unwrap();
        let (v, note) = load(&p);
        assert_eq!(v["v"], 1);
        assert!(note.is_some());
        assert!(sibling(&p, ".corrupt").exists());
    }

    #[test]
    fn corrupt_without_backup_gives_defaults() {
        let p = scratch("nobak");
        fs::write(&p, "[1,2").unwrap();
        let (v, note) = load(&p);
        assert_eq!(v, json!({}));
        assert!(note.is_some());
    }

    #[test]
    fn bom_prefixed_file_is_accepted() {
        let p = scratch("bom");
        fs::write(&p, "\u{feff}{\"a\": 2}").unwrap();
        let (v, note) = load(&p);
        assert_eq!(v["a"], 2);
        assert!(note.is_none());
    }

    #[test]
    fn non_object_is_rejected() {
        let p = scratch("array");
        fs::write(&p, "[1,2,3]").unwrap();
        let (v, note) = load(&p);
        assert_eq!(v, json!({}));
        assert!(note.is_some());
    }
}
