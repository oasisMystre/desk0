use std::{ffi::OsString, fs, path::PathBuf, time::Duration};

use tokio_util::sync::CancellationToken;
use windows_service::{
  Result,
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
      eprintln!("Run service error: {error:?}");
    }
  });
}

pub async fn run_service() -> Result<()> {
  let (shutdown_tx, mut shutdown_rx) = tokio::sync::mpsc::channel::<()>(1);
  let status_handle =
    service_control_handler::register(SERVICE_NAME, move |control_event| match control_event {
      ServiceControl::Stop => match shutdown_tx.try_send(()) {
        Ok(()) => ServiceControlHandlerResult::NoError,
        Err(error) => {
          eprintln!("Stop service error: {error:?}");
          ServiceControlHandlerResult::Other(0)
        }
      },
      _ => ServiceControlHandlerResult::NotImplemented,
    })?;

  status_handle.set_service_status(ServiceStatus {
    service_type: ServiceType::OWN_PROCESS,
    current_state: ServiceState::Running,
    controls_accepted: ServiceControlAccept::STOP,
    exit_code: ServiceExitCode::Win32(0),
    checkpoint: 0,
    wait_hint: Duration::from_secs(10),
    process_id: None,
  })?;

  let cancellation_token = CancellationToken::new();
  let token_clone = cancellation_token.clone();

  let server_addr =
    std::env::var("DESK0_SERVER_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".to_string());
  let identity = crate::identity::DeviceIdentity::load_or_create(None)
    .expect("failed to load/create device identity");

  let tunnel_task = tokio::spawn(async move {
    if let Err(err) = crate::tunnel::run_agent_tunnel(&server_addr, identity, token_clone).await {
      eprintln!("Tunnel error in service: {err:?}");
    }
  });

  let _ = shutdown_rx.recv().await;
  cancellation_token.cancel();
  let _ = tunnel_task.await;

  status_handle.set_service_status(ServiceStatus {
    service_type: ServiceType::OWN_PROCESS,
    current_state: ServiceState::Stopped,
    controls_accepted: ServiceControlAccept::empty(),
    exit_code: ServiceExitCode::Win32(0),
    checkpoint: 0,
    wait_hint: Duration::ZERO,
    process_id: None,
  })?;

  Ok(())
}

pub fn check_and_request_elevation() -> anyhow::Result<bool> {
  if is_elevated() {
    return Ok(true);
  }

  println!("Requesting Administrator elevation...");
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
      println!("Elevated installer launched successfully.");
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

    let res = GetTokenInformation(
      token,
      TokenElevation,
      Some(&mut elevation as *mut _ as *mut _),
      size,
      &mut return_length,
    );

    let _ = windows::Win32::Foundation::CloseHandle(token);

    if res.is_ok() {
      elevation.TokenIsElevated != 0
    } else {
      false
    }
  }
}

pub fn install(server_addr: Option<&str>, org_id: Option<&str>) -> anyhow::Result<()> {
  if !is_elevated() {
    check_and_request_elevation()?;
    return Ok(());
  }

  // 1. Initialize & persist identity
  let identity = crate::identity::DeviceIdentity::load_or_create(org_id)?;
  println!(
    "Device registered: id={}, org={}",
    identity.device_id, identity.org_id
  );

  // 2. Prepare Program Files directory
  let prog_files =
    std::env::var("ProgramFiles").unwrap_or_else(|_| r"C:\Program Files".to_string());
  let target_dir = PathBuf::from(prog_files).join("Desk0");
  fs::create_dir_all(&target_dir)?;

  let target_exe = target_dir.join("desk0-agent.exe");
  let current_exe = std::env::current_exe()?;

  if current_exe != target_exe {
    println!("Installing executable to {}...", target_exe.display());
    fs::copy(&current_exe, &target_exe)?;
  }

  // 3. Register & Start Windows Service
  let manager = ServiceManager::local_computer(
    None::<&str>,
    ServiceManagerAccess::CREATE_SERVICE | ServiceManagerAccess::CONNECT,
  )?;

  let mut launch_args = vec![OsString::from("--service")];
  if let Some(addr) = server_addr {
    launch_args.push(OsString::from("--server-addr"));
    launch_args.push(OsString::from(addr));
  }

  let service_info = ServiceInfo {
    name: SERVICE_NAME.into(),
    display_name: SERVICE_DISPLAY_NAME.into(),
    service_type: ServiceType::OWN_PROCESS,
    start_type: ServiceStartType::AutoStart,
    error_control: ServiceErrorControl::Normal,
    executable_path: target_exe.clone(),
    launch_arguments: launch_args,
    dependencies: vec![],
    account_name: None,
    account_password: None,
  };

  // If service exists, delete it first to reinstall cleanly
  if let Ok(existing) =
    manager.open_service(SERVICE_NAME, ServiceAccess::STOP | ServiceAccess::DELETE)
  {
    let _ = existing.stop();
    let _ = existing.delete();
    std::thread::sleep(Duration::from_millis(500));
  }

  let service =
    manager.create_service(&service_info, ServiceAccess::START | ServiceAccess::STOP)?;
  println!("Windows service '{SERVICE_NAME}' registered successfully with AutoStart.");

  if let Err(e) = service.start(&[]) {
    eprintln!("Warning: Service registered but could not be started immediately: {e:?}");
  } else {
    println!("Windows service '{SERVICE_NAME}' started.");
  }

  // 4. Clean up temporary installer if executed from temp folder
  if current_exe != target_exe {
    println!("Installation complete. Installer will now exit and clean up.");
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
    println!("Windows service '{SERVICE_NAME}' stopped and removed.");
  } else {
    println!("Windows service '{SERVICE_NAME}' is not installed.");
  }

  Ok(())
}

pub fn start() -> anyhow::Result<()> {
  service_dispatcher::start(SERVICE_NAME, ffi_service_main)?;
  Ok(())
}
