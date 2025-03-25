use windows::Win32::System::DataExchange::{OpenClipboard, GetClipboardData, CloseClipboard};
use windows::Win32::Foundation::HWND;

use std::ffi::CStr;

pub fn get_clipboard_datas() -> Result<String, String> {
  unsafe {
      let hwnd = HWND::default(); // Utilise HWND par défaut pour représenter une fenêtre globale

      if let Ok(_) = OpenClipboard(hwnd) {
          let clipboard_data = match GetClipboardData(13) {
              Ok(handle) => handle, // Extraire le handle si la récupération réussit
              Err(_) => {
                  CloseClipboard();
                  return Err("Failed to get clipboard data".to_string());
              }
          };

          if !clipboard_data.is_invalid() {
              let text = CStr::from_ptr(clipboard_data.0 as *const i8).to_string_lossy().into_owned();
              println!("Clipboard Content: {}", text);
              CloseClipboard();
              Ok(text)
          } else {
              CloseClipboard();
              Err("Clipboard data is null".to_string())
          }
      } else {
          return Err("Error while opening clipboard through Windows API".to_string());
      }
  }
}


// pub fn get_clipboard_datas() -> Result<String, String> {
//     Ok("Hello from Windows API !".to_string())
// }
