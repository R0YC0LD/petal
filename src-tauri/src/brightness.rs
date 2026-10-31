//! Dahili ekran parlaklığı.
//!
//! 1) Ekran sürücüsü IOCTL'i (`\\.\LCD`): süreç başlatmadan, mikrosaniye düzeyinde.
//! 2) IOCTL desteklenmiyorsa tek ve kalıcı bir PowerShell oturumu üzerinden WMI
//!    (`WmiMonitorBrightnessMethods`). Her fare hareketinde yeni süreç BAŞLATILMAZ.
//!
//! Komutlar kuyruğa girer, art arda gelenlerden yalnızca en son değer uygulanır.
//! `set` hemen döner (iyimser arayüz); donanım yanıtı arka planda gerçekleşir.

use std::{
    io::Write,
    process::{Child, ChildStdin, Command, Stdio},
    sync::{
        atomic::{AtomicI32, Ordering},
        Condvar, Mutex,
    },
    thread,
};
use windows::{
    core::w,
    Win32::{
        Foundation::{CloseHandle, GENERIC_READ, GENERIC_WRITE, HANDLE},
        Storage::FileSystem::{
            CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING,
        },
        System::IO::DeviceIoControl,
    },
};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

const NO_WINDOW: u32 = 0x0800_0000;
// CTL_CODE(FILE_DEVICE_VIDEO=0x23, fonksiyon, METHOD_BUFFERED, FILE_ANY_ACCESS)
const IOCTL_QUERY_DISPLAY_BRIGHTNESS: u32 = 0x0023_0498;
const IOCTL_SET_DISPLAY_BRIGHTNESS: u32 = 0x0023_049C;
const POLICY_BOTH: u8 = 3;

#[repr(C)]
#[derive(Default, Clone, Copy)]
struct DisplayBrightness {
    policy: u8,
    ac: u8,
    dc: u8,
}

static TARGET: AtomicI32 = AtomicI32::new(-1);
static WAKE: (Mutex<bool>, Condvar) = (Mutex::new(false), Condvar::new());
static STARTED: Mutex<bool> = Mutex::new(false);

fn open_lcd() -> Option<HANDLE> {
    unsafe {
        CreateFileW(
            w!("\\\\.\\LCD"),
            (GENERIC_READ | GENERIC_WRITE).0,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            None,
            OPEN_EXISTING,
            FILE_ATTRIBUTE_NORMAL,
            None,
        )
        .ok()
    }
}

fn ioctl_get() -> Option<u32> {
    let h = open_lcd()?;
    let mut out = DisplayBrightness::default();
    let mut ret = 0u32;
    let ok = unsafe {
        DeviceIoControl(
            h,
            IOCTL_QUERY_DISPLAY_BRIGHTNESS,
            None,
            0,
            Some(&mut out as *mut _ as *mut _),
            std::mem::size_of::<DisplayBrightness>() as u32,
            Some(&mut ret),
            None,
        )
        .is_ok()
    };
    unsafe {
        let _ = CloseHandle(h);
    }
    // Güçte ve pilde aynı değer ayarlandığı için AC değeri kullanılır.
    ok.then_some(out.ac as u32)
}

fn ioctl_set(percent: u8) -> bool {
    let Some(h) = open_lcd() else { return false };
    let v = DisplayBrightness {
        policy: POLICY_BOTH,
        ac: percent,
        dc: percent,
    };
    let mut ret = 0u32;
    let ok = unsafe {
        DeviceIoControl(
            h,
            IOCTL_SET_DISPLAY_BRIGHTNESS,
            Some(&v as *const _ as *const _),
            std::mem::size_of::<DisplayBrightness>() as u32,
            None,
            0,
            Some(&mut ret),
            None,
        )
        .is_ok()
    };
    unsafe {
        let _ = CloseHandle(h);
    }
    ok
}

struct Ps {
    child: Child,
    stdin: ChildStdin,
}

impl Ps {
    fn spawn() -> Option<Ps> {
        let mut c = Command::new("powershell");
        c.args(["-NoProfile", "-NonInteractive", "-Command", "-"])
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        #[cfg(windows)]
        c.creation_flags(NO_WINDOW);
        let mut child = c.spawn().ok()?;
        let mut stdin = child.stdin.take()?;
        // WMI nesnesi bir kez alınır, sonraki çağrılarda yeniden kullanılır.
        writeln!(stdin, "$m = Get-CimInstance -Namespace root/WMI -ClassName WmiMonitorBrightnessMethods | Select-Object -First 1").ok()?;
        Some(Ps { child, stdin })
    }

    fn set(&mut self, p: u8) -> bool {
        writeln!(self.stdin, "Invoke-CimMethod -InputObject $m -MethodName WmiSetBrightness -Arguments @{{Timeout=[uint32]1;Brightness=[byte]{p}}} | Out-Null").is_ok()
            && self.stdin.flush().is_ok()
    }
}

impl Drop for Ps {
    fn drop(&mut self) {
        let _ = self.child.kill();
    }
}

fn worker() {
    let mut ps: Option<Ps> = None;
    let use_ioctl = ioctl_get().is_some();
    loop {
        {
            let (lock, cv) = &WAKE;
            let mut flag = lock.lock().unwrap();
            while !*flag {
                flag = cv.wait(flag).unwrap();
            }
            *flag = false;
        }
        // Yalnızca en son hedef uygulanır.
        let t = TARGET.swap(-1, Ordering::SeqCst);
        if t < 0 {
            continue;
        }
        let p = t.clamp(0, 100) as u8;
        if use_ioctl && ioctl_set(p) {
            continue;
        }
        if ps.is_none() {
            ps = Ps::spawn();
        }
        if let Some(s) = ps.as_mut() {
            if !s.set(p) {
                ps = None;
            }
        }
    }
}

fn ensure_worker() {
    let mut started = STARTED.lock().unwrap();
    if !*started {
        *started = true;
        thread::Builder::new()
            .name("petal-brightness".into())
            .spawn(worker)
            .ok();
    }
}

/// Anlık parlaklık (0-100). Dahili panel yoksa veya desteklenmiyorsa `None`.
pub fn get() -> Option<u32> {
    ioctl_get().or_else(|| {
        let mut c = Command::new("powershell");
        c.args(["-NoProfile", "-NonInteractive", "-Command", "(Get-CimInstance -Namespace root/WMI -ClassName WmiMonitorBrightness -ErrorAction Stop | Select-Object -First 1).CurrentBrightness"]);
        #[cfg(windows)]
        c.creation_flags(NO_WINDOW);
        let out = c.output().ok()?;
        String::from_utf8_lossy(&out.stdout).trim().parse().ok()
    })
}

/// Hedef parlaklığı kuyruğa koyar ve hemen döner.
pub fn set(percent: u32) {
    ensure_worker();
    TARGET.store(percent.min(100) as i32, Ordering::SeqCst);
    let (lock, cv) = &WAKE;
    *lock.lock().unwrap() = true;
    cv.notify_one();
}
