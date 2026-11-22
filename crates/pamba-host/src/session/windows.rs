//! Leaving the screen on Windows: the session disconnected, so that Windows' sign-in stands for
//! whoever comes next.
//!
//! Each declaration below cites the line of Microsoft's `windows-sys` it was read from: at the
//! commit of `windows-rs` that `Cargo.lock` names, `crates/libs/sys/src/Windows/Win32/`, and for
//! the two that source does not hold, the published `windows-sys` 0.61.2,
//! `src/Windows/Win32/NetworkManagement/NetManagement/mod.rs`.

#![allow(
    unsafe_code,
    reason = "Windows offers the session's act and the three facts through its C interface alone, and nothing safe beneath this crate offers them"
)]

use std::ffi::c_void;
use std::io;
use std::iter::once;
use std::ptr;
use std::slice;

use super::{Accounts, Join, Policy, Read, Stayed};

/// `HKEY_LOCAL_MACHINE`, `winreg/mod.rs:190`.
const LOCAL_MACHINE: isize = -2_147_483_646;
/// `RRF_RT_REG_DWORD`, `winreg/mod.rs:274`.
const DWORD_ONLY: u32 = 16;
/// `ERROR_SUCCESS`, `winerror/mod.rs:3915`.
const SUCCESS: i32 = 0;
/// `ERROR_FILE_NOT_FOUND`, `winerror/mod.rs:2104`.
const NOT_FOUND: i32 = 2;
/// `NERR_Success`, 0.61.2 `mod.rs:2113`.
const NET_SUCCESS: u32 = 0;
/// `FILTER_NORMAL_ACCOUNT`, `lmaccess/mod.rs:171`.
const NORMAL_ACCOUNTS: u32 = 2;
/// `MAX_PREFERRED_LENGTH`, `lmcons/mod.rs:35`.
const AS_LONG_AS_NEEDED: u32 = u32::MAX;
/// `UF_ACCOUNTDISABLE`, `lmaccess/mod.rs:743`.
const DISABLED: u32 = 2;
/// `NetSetupDomainName`, `lmjoin/mod.rs:124`.
const DOMAIN: i32 = 3;
/// `DSREG_DEVICE_JOIN`, `lmjoin/mod.rs:33`.
const DEVICE_JOINED: i32 = 1;
/// `WTS_CURRENT_SESSION`, `wtsapi32/mod.rs:769`.
const THIS_SESSION: u32 = u32::MAX;

/// `USER_INFO_1`, `lmaccess/mod.rs:798-807`, whole, since Windows gives an array of them and the
/// array's stride is the whole structure's. Only `flags` is read.
#[repr(C)]
struct UserInfo1 {
    name: *mut u16,
    password: *mut u16,
    password_age: u32,
    privilege: u32,
    home_dir: *mut u16,
    comment: *mut u16,
    flags: u32,
    script_path: *mut u16,
}

/// The first member of `DSREG_JOIN_INFO`, `lmjoin/mod.rs:37-38`, the one read. The structure is
/// Windows' own, reached through the pointer Windows gave and freed by Windows' own call.
#[repr(C)]
struct JoinInfoStart {
    join_type: i32,
}

#[link(name = "advapi32")]
unsafe extern "system" {
    /// `winreg/mod.rs:92`.
    fn RegGetValueW(
        key: isize,
        subkey: *const u16,
        value: *const u16,
        flags: u32,
        kind: *mut u32,
        data: *mut c_void,
        size: *mut u32,
    ) -> i32;
}

#[link(name = "netapi32")]
unsafe extern "system" {
    /// `lmaccess/mod.rs:80`, with `bufptr` typed as the array of level 1 that Windows writes there.
    fn NetUserEnum(
        server: *const u16,
        level: u32,
        filter: u32,
        buffer: *mut *mut UserInfo1,
        most: u32,
        read: *mut u32,
        total: *mut u32,
        resume: *mut u32,
    ) -> u32;
    /// `lmjoin/mod.rs:12`.
    fn NetGetJoinInformation(server: *const u16, name: *mut *mut u16, status: *mut i32) -> u32;
    /// `lmjoin/mod.rs:10`.
    fn NetGetAadJoinInformation(tenant: *const u16, info: *mut *mut JoinInfoStart) -> i32;
    /// `lmjoin/mod.rs:8`.
    fn NetFreeAadJoinInformation(info: *mut JoinInfoStart);
    /// 0.61.2 `mod.rs:20`.
    fn NetApiBufferFree(buffer: *const c_void) -> u32;
}

#[link(name = "wtsapi32")]
unsafe extern "system" {
    /// `wtsapi32/mod.rs:23`.
    fn WTSDisconnectSession(server: *mut c_void, session: u32, wait: i32) -> i32;
}

/// Windows' side of the act. It holds nothing: the act is on this session, of this computer.
#[derive(Debug)]
pub(super) struct Side;

impl Side {
    /// Asks Windows to disconnect this session, and does not wait for it.
    #[expect(
        clippy::unused_self,
        reason = "the act is had only through a `SwitchUser`, whose `Side` shows it was offered"
    )]
    pub(super) fn leave(&self) -> Result<(), Stayed> {
        // SAFETY: the null server is WTS_CURRENT_SERVER_HANDLE (`wtsapi32/mod.rs:766`) and
        // THIS_SESSION is WTS_CURRENT_SESSION, so the call names this computer and this session and
        // passes no memory at all.
        let taken = unsafe { WTSDisconnectSession(ptr::null_mut(), THIS_SESSION, 0) };
        if taken == 0 {
            Err(Stayed {
                code: io::Error::last_os_error()
                    .raw_os_error()
                    .unwrap_or_default(),
            })
        } else {
            Ok(())
        }
    }
}

/// The three facts, each as Windows gave it, or unread where it refused.
pub(super) fn read() -> Read {
    Read {
        policy: policy(),
        accounts: accounts(),
        join: join(),
    }
}

/// The text of a Windows string: UTF-16, ending in a zero.
fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(once(0)).collect()
}

/// "Hide entry points for Fast User Switching", as group policy writes it.
fn policy() -> Policy {
    let subkey = wide(r"SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\System");
    let name = wide("HideFastUserSwitching");
    let mut value = 0_u32;
    let mut size = 4_u32;
    // SAFETY: both strings end in a zero and outlive the call; `value` is four writable bytes, as
    // `size` says, and Windows writes no more than `size` (reggetvaluew.md, pcbData).
    let answer = unsafe {
        RegGetValueW(
            LOCAL_MACHINE,
            subkey.as_ptr(),
            name.as_ptr(),
            DWORD_ONLY,
            ptr::null_mut(),
            (&raw mut value).cast(),
            &raw mut size,
        )
    };
    match answer {
        SUCCESS if value == 0 => Policy::Shown,
        SUCCESS => Policy::Hidden,
        NOT_FOUND => Policy::Shown,
        _ => Policy::Unread,
    }
}

/// The local accounts that are not disabled.
fn accounts() -> Accounts {
    let mut buffer = ptr::null_mut::<UserInfo1>();
    let mut read = 0_u32;
    let mut total = 0_u32;
    // SAFETY: every out-pointer is to a local that outlives the call; a null resume handle asks for
    // no resumption (netuserenum.md, resume_handle).
    let answer = unsafe {
        NetUserEnum(
            ptr::null(),
            1,
            NORMAL_ACCOUNTS,
            &raw mut buffer,
            AS_LONG_AS_NEEDED,
            &raw mut read,
            &raw mut total,
            ptr::null_mut(),
        )
    };
    let counted = if answer == NET_SUCCESS && !buffer.is_null() {
        // SAFETY: on success Windows wrote `read` USER_INFO_1 structures as one array at `buffer`,
        // which it allocated; they stay valid until the buffer is freed below.
        let entries = unsafe { slice::from_raw_parts(buffer, read as usize) };
        let enabled = entries
            .iter()
            .filter(|entry| entry.flags & DISABLED == 0)
            .count();
        u32::try_from(enabled).map_or(Accounts::Unread, Accounts::Counted)
    } else if answer == NET_SUCCESS && read == 0 {
        Accounts::Counted(0)
    } else {
        Accounts::Unread
    };
    if !buffer.is_null() {
        // SAFETY: `buffer` is the one NetUserEnum allocated, freed once and not read after; it is
        // freed on failure too, as netuserenum.md asks of ERROR_MORE_DATA.
        unsafe { NetApiBufferFree(buffer.cast()) };
    }
    counted
}

/// Joined where either Windows' domain or its Entra answer says so, alone where both answered and
/// neither does, and unread otherwise.
fn join() -> Join {
    match (domain(), entra()) {
        (Some(true), _) | (_, Some(true)) => Join::Joined,
        (Some(false), Some(false)) => Join::Alone,
        _ => Join::Unread,
    }
}

/// Whether the computer is joined to a domain, or none where Windows did not say.
fn domain() -> Option<bool> {
    let mut name = ptr::null_mut::<u16>();
    let mut status = 0_i32;
    // SAFETY: both out-pointers are to locals that outlive the call.
    let answer = unsafe { NetGetJoinInformation(ptr::null(), &raw mut name, &raw mut status) };
    if !name.is_null() {
        // SAFETY: `name` is the buffer NetGetJoinInformation allocated, freed once and not read.
        unsafe { NetApiBufferFree(name.cast()) };
    }
    (answer == NET_SUCCESS).then_some(status == DOMAIN)
}

/// Whether the computer is joined to Entra, or none where Windows did not say. A work account the
/// person added is no join of the computer: it lets no other person sign in.
fn entra() -> Option<bool> {
    let mut info = ptr::null_mut::<JoinInfoStart>();
    // SAFETY: the out-pointer is to a local that outlives the call; a null tenant asks for the
    // computer's own join (netgetaadjoininformation.md, pcszTenantId).
    let answer = unsafe { NetGetAadJoinInformation(ptr::null(), &raw mut info) };
    let joined = if answer < 0 {
        None
    } else if info.is_null() {
        Some(false)
    } else {
        // SAFETY: on success a non-null `info` is Windows' DSREG_JOIN_INFO, whose first member is
        // the join's type; it is read before the structure is freed below.
        Some(unsafe { (*info).join_type } == DEVICE_JOINED)
    };
    if !info.is_null() {
        // SAFETY: `info` is the structure NetGetAadJoinInformation gave, freed once by Windows' own
        // call, on every path, and not read after.
        unsafe { NetFreeAadJoinInformation(info) };
    }
    joined
}
