//! GPU kullanımı: Windows performans sayacı "GPU Engine" (Görev Yöneticisi ile aynı kaynak).
//!
//! Yöntem (RESEARCH.md §4): her (adaptör LUID, motor türü) için örnekleri toplar, adaptör
//! başına motor türleri arasında en büyüğünü, adaptörler arasında en büyüğünü raporlar.
//! Motor türlerini toplamak %100'ü aşabilir; bu yüzden toplanmaz.

use std::{collections::HashMap, sync::Mutex};
use windows::{
    core::{w, PCWSTR},
    Win32::System::Performance::{
        PdhAddEnglishCounterW, PdhCollectQueryData, PdhGetFormattedCounterArrayW, PdhOpenQueryW,
        PDH_FMT_COUNTERVALUE_ITEM_W, PDH_FMT_DOUBLE, PDH_HCOUNTER, PDH_HQUERY,
    },
};

const PDH_MORE_DATA: u32 = 0x800007D2;

struct Query {
    query: usize,
    counter: usize,
}

static QUERY: Mutex<Option<Query>> = Mutex::new(None);

fn open() -> Option<Query> {
    unsafe {
        let mut q = PDH_HQUERY::default();
        if PdhOpenQueryW(PCWSTR::null(), 0, &mut q) != 0 {
            return None;
        }
        let mut c = PDH_HCOUNTER::default();
        if PdhAddEnglishCounterW(q, w!("\\GPU Engine(*)\\Utilization Percentage"), 0, &mut c) != 0 {
            return None;
        }
        // İlk örnek yalnızca taban çizgisi oluşturur.
        PdhCollectQueryData(q);
        Some(Query {
            query: q.0 as usize,
            counter: c.0 as usize,
        })
    }
}

/// Örnek adından (adaptör, motor türü) çıkarır.
/// Biçim: `pid_1234_luid_0x00000000_0x0000C739_phys_0_eng_3_engtype_3D`
pub fn parse_instance(name: &str) -> Option<(String, String)> {
    let luid = name.split("_phys_").next()?;
    let luid = luid.split("_luid_").nth(1)?.to_string();
    let engtype = name.split("_engtype_").nth(1)?.to_string();
    Some((luid, engtype))
}

/// Örnek değerlerinden toplam GPU yüzdesi.
pub fn aggregate<'a>(samples: impl IntoIterator<Item = (&'a str, f64)>) -> Option<f32> {
    let mut per: HashMap<(String, String), f64> = HashMap::new();
    for (name, v) in samples {
        if let Some(k) = parse_instance(name) {
            *per.entry(k).or_default() += v;
        }
    }
    let mut per_adapter: HashMap<String, f64> = HashMap::new();
    for ((luid, _eng), v) in per {
        let e = per_adapter.entry(luid).or_default();
        *e = e.max(v);
    }
    per_adapter
        .values()
        .cloned()
        .reduce(f64::max)
        .map(|v| v.clamp(0.0, 100.0) as f32)
}

/// En yüklü GPU'nun kullanımı (0-100). Sayaç yoksa veya ilk örnekse `None`.
pub fn usage() -> Option<f32> {
    let mut guard = QUERY.lock().ok()?;
    if guard.is_none() {
        *guard = Some(open()?);
        return None;
    }
    let q = guard.as_ref()?;
    unsafe {
        let query = PDH_HQUERY(q.query as *mut _);
        let counter = PDH_HCOUNTER(q.counter as *mut _);
        if PdhCollectQueryData(query) != 0 {
            return None;
        }
        let mut size = 0u32;
        let mut count = 0u32;
        let r = PdhGetFormattedCounterArrayW(counter, PDH_FMT_DOUBLE, &mut size, &mut count, None);
        if r != PDH_MORE_DATA || size == 0 {
            return None;
        }
        // Tampon hizalaması için u64 dilimi kullanılır.
        let mut buf = vec![0u64; (size as usize).div_ceil(8)];
        let ptr = buf.as_mut_ptr() as *mut PDH_FMT_COUNTERVALUE_ITEM_W;
        if PdhGetFormattedCounterArrayW(counter, PDH_FMT_DOUBLE, &mut size, &mut count, Some(ptr))
            != 0
        {
            return None;
        }
        let names: Vec<(String, f64)> = (0..count as usize)
            .map(|i| {
                let item = &*ptr.add(i);
                (
                    item.szName.to_string().unwrap_or_default(),
                    item.FmtValue.Anonymous.doubleValue,
                )
            })
            .collect();
        aggregate(names.iter().map(|(n, v)| (n.as_str(), *v)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const A3D: &str = "pid_1_luid_0x00000000_0x0000AAAA_phys_0_eng_0_engtype_3D";
    const A3D_OTHER_PID: &str = "pid_2_luid_0x00000000_0x0000AAAA_phys_0_eng_0_engtype_3D";
    const ACOPY: &str = "pid_1_luid_0x00000000_0x0000AAAA_phys_0_eng_1_engtype_Copy";
    const B3D: &str = "pid_1_luid_0x00000000_0x0000BBBB_phys_0_eng_0_engtype_3D";

    #[test]
    fn parses_instance_name() {
        let (l, e) = parse_instance(A3D).unwrap();
        assert_eq!(l, "0x00000000_0x0000AAAA");
        assert_eq!(e, "3D");
    }

    #[test]
    fn sums_processes_within_engine_type() {
        let v = aggregate([(A3D, 10.0), (A3D_OTHER_PID, 15.0)]).unwrap();
        assert!((v - 25.0).abs() < 0.01);
    }

    #[test]
    fn takes_max_of_engine_types_not_sum() {
        let v = aggregate([(A3D, 40.0), (ACOPY, 30.0)]).unwrap();
        assert!((v - 40.0).abs() < 0.01);
    }

    #[test]
    fn takes_max_across_adapters() {
        let v = aggregate([(A3D, 20.0), (B3D, 55.0)]).unwrap();
        assert!((v - 55.0).abs() < 0.01);
    }

    #[test]
    fn no_samples_is_unknown_not_zero() {
        assert_eq!(aggregate(std::iter::empty::<(&str, f64)>()), None);
    }

    #[test]
    fn caps_at_100() {
        assert_eq!(aggregate([(A3D, 150.0)]), Some(100.0));
    }
}
