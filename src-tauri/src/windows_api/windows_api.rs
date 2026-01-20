use windows::Win32::System::DataExchange::{
    OpenClipboard, CloseClipboard, GetClipboardData, EnumClipboardFormats,
    GetClipboardFormatNameW,
};
use windows::Win32::System::Memory::{GlobalLock};
use windows::Win32::Foundation::{HWND, HGLOBAL};
use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::slice;

const CF_UNICODETEXT: u32 = 13;

pub fn get_clipboard_datas() -> Result<String, String> {
    unsafe {
        let hwnd = HWND::default(); // Fenêtre par défaut

        if OpenClipboard(hwnd).is_ok() {
            // Récupère les données du presse-papiers dans le format Unicode
            let handle = match GetClipboardData(CF_UNICODETEXT) {
                Ok(h) => h,
                Err(_) => {
                    CloseClipboard();
                    return Err("Échec GetClipboardData(CF_UNICODETEXT)".into());
                }
            };

            if !handle.is_invalid() {
                // Accès brut à la mémoire via GlobalLock
                let ptr = GlobalLock(HGLOBAL(handle.0)) as *const u16;
                if ptr.is_null() {
                    CloseClipboard();
                    return Err("Pointeur null (GlobalLock)".into());
                }

                // Lecture UTF-16 → Rust String
                let mut len = 0;
                while *ptr.add(len) != 0 {
                    len += 1;
                }

                let slice = slice::from_raw_parts(ptr, len);
                let text = OsString::from_wide(slice).to_string_lossy().into_owned();

                CloseClipboard();
                Ok(text)
            } else {
                CloseClipboard();
                Err("Données invalides".into())
            }
        } else {
            Err("Impossible d’ouvrir le presse-papiers".into())
        }
    }
}

pub fn list_clipboard_elements() -> Result<(), String> {
    unsafe {
        if OpenClipboard(HWND::default()).is_err() {
            return Err("Impossible d'ouvrir le presse-papiers".into());
        }

        println!("📋 Liste des formats présents dans le presse-papiers :");

        let mut format: u32 = 0;

        while {
            format = EnumClipboardFormats(format);
            format != 0
        } {
            // Tente de récupérer le nom du format
            let mut name_buf = [0u16; 128];
            let len = GetClipboardFormatNameW(format, &mut name_buf);

            let format_name = if len > 0 {
                OsString::from_wide(&name_buf[..len as usize])
                    .to_string_lossy()
                    .into_owned()
            } else {
                format!("Format standard #{format}")
            };

            println!("🧾 Format détecté : {}", format_name);
        }

        CloseClipboard();
        Ok(())
    }
}
