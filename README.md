# kosmos-plugin-sdk (Rust)

Shared guest-side runtime for [Kosmos](https://github.com/kosmos-suite/kosmos) WASM plugins — the
pointer/length-packed-into-i64 ABI, host-memory `alloc`/`dealloc`, and the permission-scoped
`env.http_fetch` host import every plugin gets.

A new plugin depends on it like any [crates.io](https://crates.io/crates/kosmos-plugin-sdk) crate:

```toml
[dependencies]
kosmos-plugin-sdk = "0.1.0"
```

and, at minimum:

```rust
kosmos_plugin_sdk::export_memory_abi!();

use kosmos_plugin_sdk::{http_fetch, read_string, write_string};

#[no_mangle]
pub extern "C" fn my_export(arg_ptr: i32, arg_len: i32) -> i64 {
    let arg = unsafe { read_string(arg_ptr, arg_len) };
    write_string(&format!("got: {arg}"))
}
```

See `src/lib.rs` for the full API and why `export_memory_abi!()` is a macro rather than a plain
function call. Compile guests with `cargo build --release --target wasm32-unknown-unknown`.

Example guests built on this SDK: `ping` in
[kosmos-plugin-examples](https://github.com/kosmos-suite/kosmos-plugin-examples) (a synthetic ABI
check), and the real, production-used
[kosmos-plugin-tmdb-search](https://github.com/kosmos-suite/kosmos-plugin-tmdb-search).
