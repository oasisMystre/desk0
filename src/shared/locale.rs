use windows::Win32::{
    Globalization::GetSystemDefaultLocaleName, System::SystemServices::LOCALE_NAME_MAX_LENGTH,
};

pub fn get_system_locale() -> String {
    let mut buffer = [0u16, LOCALE_NAME_MAX_LENGTH as u16];
    let len = unsafe { GetSystemDefaultLocaleName(&mut buffer) };
    String::from_utf16_lossy(&buffer[..(len as usize).saturating_sub(1)])
}
