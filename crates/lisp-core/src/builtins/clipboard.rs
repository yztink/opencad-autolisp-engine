use crate::ast::Value;
use crate::env::Env;
use crate::error::LispError;

#[cfg(windows)]
mod platform {
    use std::ffi::c_void;

    const CF_UNICODETEXT: u32 = 13;
    const GMEM_MOVEABLE: u32 = 0x0002;

    #[link(name = "user32")]
    extern "system" {
        fn OpenClipboard(hWndNewOwner: *mut c_void) -> i32;
        fn CloseClipboard() -> i32;
        fn EmptyClipboard() -> i32;
        fn GetClipboardData(uFormat: u32) -> *mut c_void;
        fn SetClipboardData(uFormat: u32, hMem: *mut c_void) -> *mut c_void;
        fn IsClipboardFormatAvailable(uFormat: u32) -> i32;
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn GlobalAlloc(uFlags: u32, dwBytes: usize) -> *mut c_void;
        fn GlobalLock(hMem: *mut c_void) -> *mut c_void;
        fn GlobalUnlock(hMem: *mut c_void) -> i32;
    }

    pub fn get_text() -> Result<Option<String>, String> {
        unsafe {
            if OpenClipboard(std::ptr::null_mut()) == 0 {
                return Err("Failed to open clipboard".to_string());
            }
            struct ClipGuard;
            impl Drop for ClipGuard {
                fn drop(&mut self) {
                    unsafe {
                        CloseClipboard();
                    }
                }
            }
            let _guard = ClipGuard;

            if IsClipboardFormatAvailable(CF_UNICODETEXT) == 0 {
                return Ok(None);
            }

            let handle = GetClipboardData(CF_UNICODETEXT);
            if handle.is_null() {
                return Ok(None);
            }

            let ptr = GlobalLock(handle) as *const u16;
            if ptr.is_null() {
                return Ok(None);
            }

            let mut len = 0;
            while *ptr.add(len) != 0 {
                len += 1;
            }
            let slice = std::slice::from_raw_parts(ptr, len);
            let text = String::from_utf16_lossy(slice);
            GlobalUnlock(handle);

            Ok(Some(text))
        }
    }

    pub fn set_text(text: &str) -> Result<(), String> {
        unsafe {
            if OpenClipboard(std::ptr::null_mut()) == 0 {
                return Err("Failed to open clipboard".to_string());
            }
            struct ClipGuard;
            impl Drop for ClipGuard {
                fn drop(&mut self) {
                    unsafe {
                        CloseClipboard();
                    }
                }
            }
            let _guard = ClipGuard;

            EmptyClipboard();

            let utf16: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
            let byte_count = utf16.len() * std::mem::size_of::<u16>();

            let handle = GlobalAlloc(GMEM_MOVEABLE, byte_count);
            if handle.is_null() {
                return Err("Failed to allocate global memory for clipboard".to_string());
            }

            let dest = GlobalLock(handle) as *mut u16;
            if dest.is_null() {
                return Err("Failed to lock global memory".to_string());
            }

            std::ptr::copy_nonoverlapping(utf16.as_ptr(), dest, utf16.len());
            GlobalUnlock(handle);

            if SetClipboardData(CF_UNICODETEXT, handle).is_null() {
                return Err("Failed to set clipboard data".to_string());
            }

            Ok(())
        }
    }
}

#[cfg(not(windows))]
mod platform {
    pub fn get_text() -> Result<Option<String>, String> {
        // Fallback for non-windows platforms (e.g. tests or linux builds)
        Ok(None)
    }

    pub fn set_text(_text: &str) -> Result<(), String> {
        Ok(())
    }
}

pub fn register(env: &mut Env) {
    // ── GET_CLIPBOARD ─────────────────────────────────────────────────────────
    // Usage: (get_clipboard)
    // Returns: string content of clipboard, or nil if empty / non-text
    env.def_subr("GET_CLIPBOARD", |_args, _| {
        match platform::get_text() {
            Ok(Some(s)) => Ok(Value::Str(s)),
            Ok(None) => Ok(Value::Nil),
            Err(e) => Err(LispError::Io(format!("get_clipboard error: {}", e))),
        }
    });

    // Alias: dos_getclipboard
    env.def_subr("DOS_GETCLIPBOARD", |_args, _| {
        match platform::get_text() {
            Ok(Some(s)) => Ok(Value::Str(s)),
            Ok(None) => Ok(Value::Nil),
            Err(e) => Err(LispError::Io(format!("dos_getclipboard error: {}", e))),
        }
    });

    // ── SET_CLIPBOARD ─────────────────────────────────────────────────────────
    // Usage: (set_clipboard "text to copy")
    // Returns: "text to copy" on success, or nil / error
    env.def_subr("SET_CLIPBOARD", |args, _| {
        if args.len() != 1 {
            return Err(LispError::WrongArgCount { expected: "1 (text)", actual: args.len() });
        }
        let text = match &args[0] {
            Value::Str(s) => s.clone(),
            Value::Nil => String::new(),
            other => other.to_string(),
        };

        match platform::set_text(&text) {
            Ok(_) => Ok(Value::Str(text)),
            Err(e) => Err(LispError::Io(format!("set_clipboard error: {}", e))),
        }
    });

    // Alias: dos_setclipboard
    env.def_subr("DOS_SETCLIPBOARD", |args, _| {
        if args.len() != 1 {
            return Err(LispError::WrongArgCount { expected: "1 (text)", actual: args.len() });
        }
        let text = match &args[0] {
            Value::Str(s) => s.clone(),
            Value::Nil => String::new(),
            other => other.to_string(),
        };

        match platform::set_text(&text) {
            Ok(_) => Ok(Value::Str(text)),
            Err(e) => Err(LispError::Io(format!("dos_setclipboard error: {}", e))),
        }
    });

    // ── DOS_CLIPBOARD (polymorphic get / set matching DOSLib specification) ──
    // Usage:
    //   (dos_clipboard)        => get clipboard text
    //   (dos_clipboard "text") => set clipboard text
    env.def_subr("DOS_CLIPBOARD", |args, _| {
        if args.is_empty() {
            match platform::get_text() {
                Ok(Some(s)) => Ok(Value::Str(s)),
                Ok(None) => Ok(Value::Nil),
                Err(e) => Err(LispError::Io(format!("dos_clipboard error: {}", e))),
            }
        } else if args.len() == 1 {
            let text = match &args[0] {
                Value::Str(s) => s.clone(),
                Value::Nil => String::new(),
                other => other.to_string(),
            };

            match platform::set_text(&text) {
                Ok(_) => Ok(Value::Str(text)),
                Err(e) => Err(LispError::Io(format!("dos_clipboard error: {}", e))),
            }
        } else {
            Err(LispError::WrongArgCount { expected: "0 or 1 ([text])", actual: args.len() })
        }
    });
}
