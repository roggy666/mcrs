use std::thread::available_parallelism;

use bevy_tasks::TaskPool;
use bevy_tasks::TaskPoolBuilder;

const LEAST_THREADS: usize = 1;

#[must_use]
pub fn default_threads() -> usize {
    available_parallelism().map_or(2, |count| (count.get() / 2).max(LEAST_THREADS))
}

#[must_use]
pub fn ai_pool(threads: usize) -> TaskPool {
    TaskPoolBuilder::new()
        .thread_name("ai".to_owned())
        .num_threads(threads.max(LEAST_THREADS))
        .on_thread_spawn(lower_priority)
        .build()
}

#[cfg(windows)]
pub fn lower_priority() {
    use windows_sys::Win32::System::Threading::GetCurrentThread;
    use windows_sys::Win32::System::Threading::SetThreadPriority;
    use windows_sys::Win32::System::Threading::THREAD_PRIORITY_BELOW_NORMAL;
    // SAFETY: both calls take no pointers; the pseudo handle of the current thread is always
    // valid, and a failed priority change leaves the thread as it was.
    unsafe {
        SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_BELOW_NORMAL);
    }
}

#[cfg(target_os = "linux")]
pub fn lower_priority() {
    const NICE: libc::c_int = 5;
    // SAFETY: setpriority with PRIO_PROCESS and the id of the calling thread changes only the
    // nice value of that thread; it reads no memory and a failure leaves the value as it was.
    unsafe {
        let thread = libc::gettid();
        libc::setpriority(libc::PRIO_PROCESS, thread as libc::id_t, NICE);
    }
}

#[cfg(not(any(windows, target_os = "linux")))]
pub fn lower_priority() {}
