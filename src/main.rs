use std::mem;
use std::ptr;
use std::io::{self, Write};

#[repr(C)] 
#[derive(Clone, Copy)]
struct COORD { x: i16, y: i16 }

#[link(name = "advapi32")]
#[link(name = "kernel32")]
#[link(name = "psapi")]
unsafe extern "system" {
    fn GetCurrentProcess() -> *mut std::ffi::c_void;
    fn OpenProcessToken(h: *mut std::ffi::c_void, acc: u32, t: *mut *mut std::ffi::c_void) -> i32;
    fn LookupPrivilegeValueA(sys: *const u8, name: *const u8, luid: *mut LUID) -> i32;
    fn AdjustTokenPrivileges(t: *mut std::ffi::c_void, dis: i32, new: *const TOKEN_PRIVILEGES, len: u32, prev: *mut std::ffi::c_void, ret_len: *mut u32) -> i32;
    fn K32EnumProcesses(lpid: *mut u32, cb: u32, lpcb: *mut u32) -> i32;
    fn OpenProcess(acc: u32, inh: i32, id: u32) -> *mut std::ffi::c_void;
    fn CloseHandle(h: *mut std::ffi::c_void) -> i32;
    fn K32GetProcessMemoryInfo(h: *mut std::ffi::c_void, pps: *mut PROCESS_MEMORY_COUNTERS, cb: u32) -> i32;
    fn K32GetProcessImageFileNameA(h: *mut std::ffi::c_void, lp: *mut u8, sz: u32) -> u32;
}

#[repr(C)] struct LUID { low: u32, high: i32 }
#[repr(C)] struct LUID_AND_ATTRIBUTES { luid: LUID, attrs: u32 }
#[repr(C)] struct TOKEN_PRIVILEGES { count: u32, privs: [LUID_AND_ATTRIBUTES; 1] }
#[repr(C)] pub struct PROCESS_MEMORY_COUNTERS {
    pub cb: u32, pub pf: u32, pub peak_ws: usize, pub ws: usize,
    pub qp_pp: usize, pub q_pp: usize, pub qp_npp: usize, pub q_npp: usize,
    pub pf_u: usize, pub peak_pf_u: usize,
}

#[derive(Clone)]
struct ProcessEntry { name: String, pid: u32, mem_mb: f32 }

fn enable_debug_privilege() {
    unsafe {
        let mut token = ptr::null_mut();
        if OpenProcessToken(GetCurrentProcess(), 0x0020 | 0x0008, &mut token) != 0 {
            let mut luid = LUID { low: 0, high: 0 };
            if LookupPrivilegeValueA(ptr::null(), "SeDebugPrivilege\0".as_ptr(), &mut luid) != 0 {
                let tp = TOKEN_PRIVILEGES { count: 1, privs: [LUID_AND_ATTRIBUTES { luid, attrs: 0x00000002 }] };
                AdjustTokenPrivileges(token, 0, &tp, 0, ptr::null_mut(), ptr::null_mut());
            }
            CloseHandle(token);
        }
    }
}

fn main() {
    enable_debug_privilege();
    print!("\x1B[2J");

    loop {
        let mut processes = Vec::new();
        let mut pids = [0u32; 1024];
        let mut cb_needed = 0u32;
        let mut our_stats: Option<ProcessEntry> = None;

        unsafe {
            if K32EnumProcesses(pids.as_mut_ptr(), 4096, &mut cb_needed) != 0 {
                let count = (cb_needed / 4) as usize;
                for &pid in pids.iter().take(count) {
                    if pid == 0 { continue; }
                    let handle = OpenProcess(0x1000 | 0x0010, 0, pid);
                    if !handle.is_null() {
                        let mut counters: PROCESS_MEMORY_COUNTERS = mem::zeroed();
                        counters.cb = mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32;
                        if K32GetProcessMemoryInfo(handle, &mut counters, counters.cb) != 0 {
                            let mut buf = [0u8; 260];
                            let len = K32GetProcessImageFileNameA(handle, buf.as_mut_ptr(), 260);
                            let name = if len > 0 {
                                String::from_utf8_lossy(&buf[..len as usize]).split('\\').last().unwrap_or("Unknown").to_string()
                            } else { "Access Denied".to_string() };
                            
                            let mem_mb = (counters.ws as f32) / 1024.0 / 1024.0;
                            let entry = ProcessEntry { name: name.clone(), pid, mem_mb };
                            
                            // Check if this is us
                            if name.to_lowercase().contains("experiments") {
                                our_stats = Some(entry.clone());
                            }
                            processes.push(entry);
                        }
                        CloseHandle(handle);
                    }
                }
            }
        }

        processes.sort_by(|a, b| b.mem_mb.partial_cmp(&a.mem_mb).unwrap_or(std::cmp::Ordering::Equal));

        let mut out = String::new();
        out.push_str("\x1B[H"); // Cursor to home
        
        // Header
        out.push_str(&format!(" {:<5} | {:<25} | {:<10} | {:<12}\n", "RANK", "PROCESS NAME", "PID", "MEM (MB)"));
        out.push_str(&format!("{}\n", "-".repeat(65)));
        
        // TOP 24 PROCESSES (leave room for separators and our program)
        for (i, p) in processes.iter().take(24).enumerate() {
            let display_name = if p.name.len() > 25 { &p.name[..22] } else { &p.name };
            let line = format!(" {:<5} | {:<25} | {:<10} | {:<9.2} MB", i + 1, display_name, p.pid, p.mem_mb);
            out.push_str(&format!("{:<65}\n", line));
        }

        // DEDICATED ROW 26 FOR OUR PROGRAM
        out.push_str(&format!("{}\n", "=".repeat(65))); // Visual separator
        if let Some(p) = our_stats {
            let line = format!(" SELF  | {:<25} | {:<10} | {:<9.2} MB", p.name, p.pid, p.mem_mb);
            out.push_str(&format!("{:<65}\n", line));
        } else {
            out.push_str(&format!("{:<65}\n", " SELF  | Not Found                 | N/A        | 0.00 MB"));
        }

        print!("{}", out);
        let _ = io::stdout().flush();
        std::thread::sleep(std::time::Duration::from_millis(800));
    }
}