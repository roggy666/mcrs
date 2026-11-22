use std::mem::{MaybeUninit, size_of};

use windows_sys::Win32::Foundation::{CloseHandle, FILETIME, HANDLE};
use windows_sys::Win32::System::ProcessStatus::{
    EnumProcesses, GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS,
};
use windows_sys::Win32::System::Threading::{
    GetProcessTimes, GetSystemTimes, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
    QueryFullProcessImageNameW,
};

use crate::os::{CoreTicks, ProcessUsage};

const HUNDRED_NANOS: u64 = 100;
const PID_SLOTS: usize = 4096;
const NAME_CHARS: usize = 1024;

fn filetime_nanos(time: &FILETIME) -> u64 {
    ((u64::from(time.dwHighDateTime) << 32) | u64::from(time.dwLowDateTime)) * HUNDRED_NANOS
}

fn zero_time() -> FILETIME {
    FILETIME {
        dwLowDateTime: 0,
        dwHighDateTime: 0,
    }
}

pub fn core_ticks() -> Vec<CoreTicks> {
    let mut idle = zero_time();
    let mut kernel = zero_time();
    let mut user = zero_time();
    // SAFETY: the three out-pointers point to live locals of the type the call writes.
    let ok = unsafe { GetSystemTimes(&mut idle, &mut kernel, &mut user) };
    if ok == 0 {
        return Vec::new();
    }
    let cores = std::thread::available_parallelism().map_or(1, |count| count.get() as u64);
    let idle = filetime_nanos(&idle) / HUNDRED_NANOS;
    let kernel = filetime_nanos(&kernel) / HUNDRED_NANOS;
    let user = filetime_nanos(&user) / HUNDRED_NANOS;
    let busy_system = kernel.saturating_sub(idle);
    (0..cores)
        .map(|_| CoreTicks {
            user: user / cores,
            system: busy_system / cores,
            nice: 0,
            idle: idle / cores,
        })
        .collect()
}

fn open(pid: i32) -> Option<HANDLE> {
    let Ok(pid) = u32::try_from(pid) else {
        return None;
    };
    // SAFETY: a plain call with the limited query right; a null handle means a failure.
    let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
    (handle != 0).then_some(handle)
}

fn close(handle: HANDLE) {
    // SAFETY: the handle came from a successful OpenProcess and is closed once.
    unsafe {
        CloseHandle(handle);
    }
}

pub fn process(pid: i32) -> Option<ProcessUsage> {
    let handle = open(pid)?;
    let mut creation = zero_time();
    let mut exit = zero_time();
    let mut kernel = zero_time();
    let mut user = zero_time();
    // SAFETY: the out-pointers point to live locals; the handle is open with the query right.
    let timed =
        unsafe { GetProcessTimes(handle, &mut creation, &mut exit, &mut kernel, &mut user) };
    let mut counters = MaybeUninit::<PROCESS_MEMORY_COUNTERS>::zeroed();
    // SAFETY: the buffer is the size the call is told; it writes at most that many bytes.
    let measured = unsafe {
        GetProcessMemoryInfo(
            handle,
            counters.as_mut_ptr(),
            size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
        )
    };
    close(handle);
    if timed == 0 {
        return None;
    }
    let rss_bytes = if measured == 0 {
        0
    } else {
        // SAFETY: the call succeeded and filled the struct.
        unsafe { counters.assume_init() }.WorkingSetSize as u64
    };
    Some(ProcessUsage {
        cpu_nanos: filetime_nanos(&kernel) + filetime_nanos(&user),
        rss_bytes,
    })
}

pub fn load_average() -> [f64; 3] {
    [0.0; 3]
}

pub fn pids() -> Vec<i32> {
    let mut slots = vec![0u32; PID_SLOTS];
    let mut written: u32 = 0;
    // SAFETY: the buffer holds `PID_SLOTS` u32 values, and the byte size passed matches it.
    let ok = unsafe {
        EnumProcesses(
            slots.as_mut_ptr(),
            (slots.len() * size_of::<u32>()) as u32,
            &mut written,
        )
    };
    if ok == 0 {
        return Vec::new();
    }
    let count = (written as usize / size_of::<u32>()).min(slots.len());
    slots
        .iter()
        .take(count)
        .filter_map(|pid| i32::try_from(*pid).ok())
        .collect()
}

pub fn name(pid: i32) -> String {
    let Some(handle) = open(pid) else {
        return String::new();
    };
    let mut buffer = vec![0u16; NAME_CHARS];
    let mut length = NAME_CHARS as u32;
    // SAFETY: the buffer has `length` UTF-16 units; the call writes at most that many.
    let ok = unsafe { QueryFullProcessImageNameW(handle, 0, buffer.as_mut_ptr(), &mut length) };
    close(handle);
    if ok == 0 {
        return String::new();
    }
    let path = String::from_utf16_lossy(&buffer[..length as usize]);
    path.rsplit(['\\', '/'])
        .next()
        .unwrap_or_default()
        .to_owned()
}
