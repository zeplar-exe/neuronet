macro_rules! ffi_catch {
    ($default:expr, $body:expr) => {
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| $body)) {
            Ok(v) => v,
            Err(e) => {
                let msg = if let Some(s) = e.downcast_ref::<&str>() {
                    s.to_string()
                } else if let Some(s) = e.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "unknown panic".to_string()
                };
                eprintln!("neuronet panic: {msg}");
                $default
            }
        }
    };
}

macro_rules! ffi_catch_ptr {
    ($body:expr) => { ffi_catch!(std::ptr::null_mut(), $body) };
}

macro_rules! ffi_catch_num {
    ($body:expr) => { ffi_catch!(Default::default(), $body) };
}

macro_rules! ffi_catch_void {
    ($body:expr) => { ffi_catch!((), { $body; }) };
}

macro_rules! ffi_catch_struct {
    ($default:expr, $body:expr) => { ffi_catch!($default, $body) };
}
