use crate::{Config, Hello, Node};
use std::{
    ffi::{CStr, CString, c_char},
    sync::{Mutex, OnceLock},
};

static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
static INSTANCE: Mutex<Option<(Node, Hello)>> = Mutex::new(None);

fn json_pointer(value: serde_json::Value) -> *mut c_char {
    CString::new(value.to_string())
        .unwrap_or_default()
        .into_raw()
}

/// Start once, off the main UI thread. Inputs must be valid NUL-terminated UTF-8 strings.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hopper_start(
    directory: *const c_char,
    name: *const c_char,
) -> *mut c_char {
    let result = std::panic::catch_unwind(|| -> anyhow::Result<Hello> {
        anyhow::ensure!(
            !directory.is_null() && !name.is_null(),
            "directory and name are required"
        );
        let mut instance = INSTANCE
            .lock()
            .map_err(|_| anyhow::anyhow!("node startup lock poisoned"))?;
        if let Some((_, hello)) = &*instance {
            return Ok(hello.clone());
        }
        let dir = unsafe { CStr::from_ptr(directory) }.to_str()?.into();
        let name = unsafe { CStr::from_ptr(name) }.to_str()?.to_owned();
        let runtime = RUNTIME.get_or_init(|| {
            tokio::runtime::Builder::new_multi_thread()
                .worker_threads(3)
                .enable_all()
                .build()
                .expect("Hopper runtime")
        });
        let (node, hello) = runtime.block_on(Node::start(Config {
            data_dir: dir,
            name,
            local_only: false,
        }))?;
        *instance = Some((node, hello.clone()));
        Ok(hello)
    });
    match result {
        Ok(Ok(hello)) => json_pointer(serde_json::to_value(hello).unwrap()),
        Ok(Err(e)) => json_pointer(serde_json::json!({"error":e.to_string()})),
        Err(_) => json_pointer(serde_json::json!({"error":"Hopper core failed to start"})),
    }
}

/// Only free pointers returned by hopper_start, exactly once. NULL is allowed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn hopper_free(value: *mut c_char) {
    if !value.is_null() {
        drop(unsafe { CString::from_raw(value) });
    }
}
