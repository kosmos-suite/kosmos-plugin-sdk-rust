//! Shared guest-side runtime for Kosmos WASM plugins: the pointer/length-packed-into-i64 ABI,
//! host-memory alloc/dealloc, and the one permission-scoped host import (`env.http_fetch`) every
//! plugin gets. See the kosmos-plugin-tmdb-search repo for a real plugin built on it.
//!
//! Every guest must call `export_memory_abi!()` once at crate root so the host has real `alloc`/
//! `dealloc` exports to call. A plain function call into this crate isn't enough for those two:
//! nothing in a guest crate ever calls `alloc`/`dealloc` itself (they exist purely for the host to
//! call), and an `.rlib` dependency's unreferenced code can get linked out of the final `cdylib`.

use std::alloc::{alloc as std_alloc, dealloc as std_dealloc, Layout};

#[link(wasm_import_module = "env")]
extern "C" {
    #[link_name = "http_fetch"]
    fn host_http_fetch(url_ptr: i32, url_len: i32) -> i64;
}

/// Packs a guest pointer/length pair into the single i64 every ABI call returns — WASM's native
/// multi-value returns aren't expressible through stable Rust's `extern "C"` FFI without
/// nightly/wasm-bindgen. The Java host (`plugins.PluginAbi`) unpacks the same way.
pub fn pack(ptr: i32, len: i32) -> i64 {
    ((len as i64) << 32) | (ptr as u32 as i64)
}

pub fn unpack(packed: i64) -> (i32, i32) {
    (((packed as u64) & 0xFFFF_FFFF) as i32, ((packed as u64) >> 32) as i32)
}

fn layout_for(len: usize) -> Layout {
    Layout::from_size_align(len.max(1), 1).unwrap()
}

pub fn alloc(len: i32) -> i32 {
    if len <= 0 {
        return 0;
    }
    unsafe { std_alloc(layout_for(len as usize)) as i32 }
}

pub fn dealloc(ptr: i32, len: i32) {
    if ptr == 0 || len <= 0 {
        return;
    }
    unsafe { std_dealloc(ptr as *mut u8, layout_for(len as usize)) }
}

/// # Safety
/// `ptr`/`len` must describe a live allocation of at least `len` bytes — the host is trusted to
/// pass back exactly what a prior `alloc` call handed it.
pub unsafe fn read_string(ptr: i32, len: i32) -> String {
    let slice = std::slice::from_raw_parts(ptr as *const u8, len as usize);
    String::from_utf8_lossy(slice).into_owned()
}

pub fn write_string(s: &str) -> i64 {
    let bytes = s.as_bytes();
    let ptr = alloc(bytes.len() as i32);
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr as *mut u8, bytes.len());
    }
    pack(ptr, bytes.len() as i32)
}

/// Calls the host's permission-scoped `http_fetch` import. Returns either the real response body
/// (host allowed the URL's host) or the host's own `"blocked:<host>"`/`"error:..."` text — the
/// guest can't distinguish the cases except by reading it, same as any real plugin would.
pub fn http_fetch(url: &str) -> String {
    let packed = unsafe { host_http_fetch(url.as_ptr() as i32, url.len() as i32) };
    let (ptr, len) = unpack(packed);
    unsafe { read_string(ptr, len) }
}

/// Generates the guest's `alloc`/`dealloc` exports. Invoke once at crate root — see the module
/// doc comment for why a direct dependency call isn't enough on its own.
#[macro_export]
macro_rules! export_memory_abi {
    () => {
        #[no_mangle]
        pub extern "C" fn alloc(len: i32) -> i32 {
            $crate::alloc(len)
        }

        #[no_mangle]
        pub extern "C" fn dealloc(ptr: i32, len: i32) {
            $crate::dealloc(ptr, len)
        }
    };
}
