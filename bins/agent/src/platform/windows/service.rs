use std::{ffi::OsString, fs, os::windows::ffi::OsStrExt, path::PathBuf, time::Duration};

use tokio::process::Command;
use windows_service::{
  service::{
    ServiceAccess, ServiceControl, ServiceControlAccept, ServiceErrorControl, ServiceExitCode,
    ServiceInfo, ServiceStartType, ServiceState, ServiceStatus, ServiceType,
  },
  service_control_handler::{self, ServiceControlHandlerResult},
  service_dispatcher,
  service_manager::{ServiceManager, ServiceManagerAccess},
};

windows_service::define_windows_service!(ffi_service_main, service_main);

pub const SERVICE_NAME: &str = "Desk0AgentService";
pub const SERVICE_DISPLAY_NAME: &str = "Desk0 Remote Access Agent";

fn service_main(_args: Vec<OsString>) {
  let rt = tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .build()
    .expect("failed to build tokio runtime");

  rt.block_on(async {
    if let Err(error) = run_service().await {
      tracing::error!(?error, "Run service error");
    }
  });
}

pub async fn run_service() -> anyhow::Result<()> {
  let (shutdown_tx, mut shutdown_rx) = tokio::sync::mpsc::channel::<()>(1);
  let status_handle =
    service_control_handler::register(SERVICE_NAME, move |control_event| match control_event {
      ServiceControl::Stop => match shutdown_tx.try_send(()) {
        Ok(()) => ServiceControlHandlerResult::NoError,
        Err(error) => {
          tracing::error!(?error, "Stop service error");
          ServiceControlHandlerResult::Other(0)
        }
      },
      _ => ServiceControlHandlerResult::NotImplemented,
    })?;

  status_handle.set_service_status(ServiceStatus {
    checkpoint: 0,
    process_id: None,
    wait_hint: Duration::from_secs(10),
    current_state: ServiceState::Running,
    exit_code: ServiceExitCode::Win32(0),
    service_type: ServiceType::OWN_PROCESS,
    controls_accepted: ServiceControlAccept::STOP,
  })?;

  let exe = std::env::current_exe()?;
  let mut child = Command::new(exe)
    ..stdin(std::process::Stdio::null())
      .stdout(std::process::Stdio::null())
      .stderr(std::process::Stdio::null())
      .spawn()?;

  tokio::select! {
      _ = shutdown_rx.recv() => {
          let _ = child.kill().await;
          let _ = child.wait().await;
      },
      status = child.wait() => {
          tracing::info!(?status, "desk0 exited");
      }
  }

  status_handle.set_service_status(ServiceStatus {
    checkpoint: 0,
    process_id: None,
    wait_hint: Duration::ZERO,
    exit_code: ServiceExitCode::Win32(0),
    current_state: ServiceState::Stopped,
    service_type: ServiceType::OWN_PROCESS,
    controls_accepted: ServiceControlAccept::empty(),
  })?;

  Ok(())
}

pub fn check_and_request_elevation() -> anyhow::Result<bool> {
  if is_elevated() {
    return Ok(true);
  }

  let current_exe = std::env::current_exe()?;
  let current_exe_w: Vec<u16> = current_exe
    .as_os_str()
    .encode_wide()
    .chain(std::iter::once(0))
    .collect();
  let verb_w: Vec<u16> = "runas\0".encode_utf16().collect();
  let args_w: Vec<u16> = "--install\0".encode_utf16().collect();

  unsafe {
    use windows::{
      Win32::UI::{Shell::ShellExecuteW, WindowsAndMessaging::SW_SHOWNORMAL},
      core::PCWSTR,
    };

    let res = ShellExecuteW(
      None,
      PCWSTR(verb_w.as_ptr()),
      PCWSTR(current_exe_w.as_ptr()),
      PCWSTR(args_w.as_ptr()),
      PCWSTR::null(),
      SW_SHOWNORMAL,
    );

    if res.0 as isize > 32 {
      std::process::exit(0);
    } else {
      anyhow::bail!("Failed to obtain administrator privileges (UAC prompt declined).");
    }
  }
}

pub fn is_elevated() -> bool {
  use windows::Win32::{
    Foundation::HANDLE,
    Security::{GetTokenInformation, TOKEN_ELEVATION, TOKEN_QUERY, TokenElevation},
    System::Threading::{GetCurrentProcess, OpenProcessToken},
  };

  unsafe {
    let mut token = HANDLE::default();
    if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() {
      return false;
    }

    let mut elevation = TOKEN_ELEVATION::default();
    let mut return_length = 0u32;
    let size = std::mem::size_of::<TOKEN_ELEVATION>() as u32;

    let result = GetTokenInformation(
      token,
      TokenElevation,
      Some(&mut elevation as *mut _ as *mut _),
      size,
      &mut return_length,
    );

    let _ = windows::Win32::Foundation::CloseHandle(token);

    if result.is_ok() {
      elevation.TokenIsElevated != 0
    } else {
      false
    }
  }
}

pub fn install() -> anyhow::Result<()> {
  if !is_elevated() {
    check_and_request_elevation()?;
    return Ok(());
  }

  let prog_files =
    std::env::var("ProgramFiles").unwrap_or_else(|_| r"C:\Program Files".to_string());
  let target_dir = PathBuf::from(prog_files).join("Desk0");
  fs::create_dir_all(&target_dir)?;

  let target_exe = target_dir.join("desk0-agent.exe");
  let current_exe = std::env::current_exe()?;

  if current_exe != target_exe {
    fs::copy(&current_exe, &target_exe)?;
  }

  let manager = ServiceManager::local_computer(
    None::<&str>,
    ServiceManagerAccess::CREATE_SERVICE | ServiceManagerAccess::CONNECT,
  )?;

  let service_info = ServiceInfo {
    account_name: None,
    account_password: None,
    name: SERVICE_NAME.into(),
    display_name: SERVICE_DISPLAY_NAME.into(),
    service_type: ServiceType::OWN_PROCESS,
    start_type: ServiceStartType::AutoStart,
    error_control: ServiceErrorControl::Normal,
    executable_path: target_exe.clone(),
    dependencies: vec![],
    launch_arguments: vec![OsString::from("--service")],
  };

  if let Ok(existing) =
    manager.open_service(SERVICE_NAME, ServiceAccess::STOP | ServiceAccess::DELETE)
  {
    let _ = existing.stop();
    let _ = existing.delete();
    std::thread::sleep(Duration::from_millis(500));
  }

  let service =
    manager.create_service(&service_info, ServiceAccess::START | ServiceAccess::STOP)?;

  if let Err(error) = service.start::<OsString>(&[]) {
    tracing::warn!(
      ?error,
      "Service registered but could not be started immediately"
    );
  }

  if current_exe != target_exe {
    schedule_self_delete(&current_exe);
  }

  Ok(())
}

fn schedule_self_delete(path: &std::path::Path) {
  let path_str = path.to_string_lossy();
  let cmd = format!("ping 127.0.0.1 -n 2 > nul & del /f /q \"{}\"", path_str);
  let _ = std::process::Command::new("cmd").args(["/C", &cmd]).spawn();
}

pub fn uninstall() -> anyhow::Result<()> {
  if !is_elevated() {
    check_and_request_elevation()?;
    return Ok(());
  }

  let manager = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT)?;
  if let Ok(service) =
    manager.open_service(SERVICE_NAME, ServiceAccess::STOP | ServiceAccess::DELETE)
  {
    let _ = service.stop();
    service.delete()?;
  }

  Ok(())
}

pub fn start() -> anyhow::Result<()> {
  service_dispatcher::start(SERVICE_NAME, ffi_service_main)?;
  Ok(())
}
