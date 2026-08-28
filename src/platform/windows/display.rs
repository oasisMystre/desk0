use ironrdp::server::{BitmapUpdate, DisplayUpdate, PixelFormat};
use tokio::{sync::mpsc::Sender, time::Duration};
use windows::Win32::{
  Graphics::Gdi::{
    BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BitBlt, CreateCompatibleBitmap, CreateCompatibleDC, DIB_RGB_COLORS, DeleteDC, DeleteObject, GetDC, GetDIBits, ReleaseDC, SRCCOPY, SelectObject,
  }, UI::WindowsAndMessaging::{GetDesktopWindow, GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN},
};

pub struct Display;

impl crate::platform::traits::Display for Display {
  async fn capture(width: u16, height: u16, sender: Sender<DisplayUpdate>) {
    let mut ticker = tokio::time::interval(Duration::from_millis(100));
    let mut last_error: Option<String> = None;
    loop {
      ticker.tick().await;
      let frame_data = match Self::capture_screen_gdi(width, height) {
        Ok(data) => {
          if last_error.is_some() {
            println!("Screen capture recovered.");
            last_error = None;
          }
          data
        }
        Err(error) => {
          let err_msg = error.to_string();
          if last_error.as_ref() != Some(&err_msg) {
            eprintln!("Screen capture error: {}", err_msg);
            last_error = Some(err_msg);
          }
          continue;
        }
      };

      let bitmap = BitmapUpdate {
        x: 0,
        y: 0,
        format: PixelFormat::BgrX32,
        data: bytes::Bytes::from(frame_data),
        width: std::num::NonZeroU16::new(width).unwrap(),
        height: std::num::NonZeroU16::new(height).unwrap(),
        stride: std::num::NonZeroUsize::new(width as usize * 4).unwrap(),
      };

      if sender.send(DisplayUpdate::Bitmap(bitmap)).await.is_err() {
        break;
      }
    }
  }
}

impl Display {
  pub fn get_desktop_size() -> (u16, u16) {
    unsafe {
      let width = GetSystemMetrics(SM_CXSCREEN) as u16;
      let height = GetSystemMetrics(SM_CYSCREEN) as u16;
      (width, height)
    }
  }

  fn capture_screen_gdi(width: u16, height: u16) -> anyhow::Result<Vec<u8>> {
    unsafe {
      let hwnd = GetDesktopWindow();
      let optional_hwnd = Some(hwnd);

      let hdc_screen = GetDC(optional_hwnd);
      let optional_hdc_screen = Some(hdc_screen);

      if hdc_screen.0.is_null() {
        anyhow::bail!("Failed to get desktop device context");
      }

      let hdc_screen_mem = CreateCompatibleDC(optional_hdc_screen);
      if hdc_screen_mem.0.is_null() {
        ReleaseDC(optional_hwnd, hdc_screen);
        anyhow::bail!("Failed to create compatible device context");
      }

      let hbitmap = CreateCompatibleBitmap(hdc_screen, width as i32, height as i32);
      if hbitmap.0.is_null() {
        DeleteDC(hdc_screen_mem);
        ReleaseDC(optional_hwnd, hdc_screen);
        anyhow::bail!("Failed to create compatible bitmap");
      }

      let old_object = SelectObject(hdc_screen_mem, hbitmap.into());
      if let Err(error) = BitBlt(
        hdc_screen_mem,
        0,
        0,
        width as i32,
        height as i32,
        optional_hdc_screen,
        0,
        0,
        SRCCOPY,
      ) {
        SelectObject(hdc_screen_mem, old_object);
        DeleteObject(hbitmap.into());
        DeleteDC(hdc_screen_mem);
        ReleaseDC(optional_hwnd, hdc_screen);
        return Err(error.into());
      }

      let mut bmi = BITMAPINFO {
        bmiHeader: BITMAPINFOHEADER {
          biPlanes: 1,
          biBitCount: 32,
          biWidth: width as i32,
          biHeight: -(height as i32),
          biCompression: BI_RGB.0 as u32,
          biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
          ..Default::default()
        },
        ..Default::default()
      };

      let buffer_size = width as usize * height as usize * 4;
      let mut buffer = vec![0u8; buffer_size];

      GetDIBits(
        hdc_screen_mem,
        hbitmap,
        0,
        height as u32,
        Some(buffer.as_mut_ptr() as *mut _),
        &mut bmi,
        DIB_RGB_COLORS,
      );

      SelectObject(hdc_screen_mem, old_object);
      DeleteObject(hbitmap.into());
      DeleteDC(hdc_screen_mem);
      ReleaseDC(optional_hwnd, hdc_screen);

      Ok(buffer)
    }
  }
}
