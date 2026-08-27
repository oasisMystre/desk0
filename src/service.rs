use std::{ffi::OsString, time::Duration};
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

fn service_main(_args: Vec<OsString>) {
    let rt = tokio::runtime::Runtime::new().expect("failed to build tokio runtime");

    rt.block_on(async {
        if let Err(error) = run_service().await {
            eprintln!("{}: run service error", error);
        }
    });
}

pub async fn run_service() -> Result<()> {
    let (shutdown_tx, mut shutdown_rx) = tokio::sync::mpsc::channel::<()>(1);
    let status_handle =
        service_control_handler::register(
            SERVICE_NAME,
            move |control_event| match control_event {
                ServiceControl::Stop => match shutdown_tx.try_send(()) {
                    Ok(()) => ServiceControlHandlerResult::NoError,
                    Err(error) => {
                        eprintln!("{}: stop service error", error);
                        ServiceControlHandlerResult::Other(0)
                    }
                },
                _ => ServiceControlHandlerResult::NotImplemented,
            },
        )?;

    status_handle.set_service_status(ServiceStatus {
        service_type: ServiceType::OWN_PROCESS,
        current_state: ServiceState::Running,
        controls_accepted: ServiceControlAccept::STOP,
        exit_code: ServiceExitCode::Win32(0),
        checkpoint: 0,
        wait_hint: Duration::from_secs(10),
        process_id: None,
    })?;

    loop {
        if shutdown_rx.try_recv().is_ok() {
            break;
        }

        tokio::time::sleep(Duration::from_secs(1)).await;
    }

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

pub const SERVICE_NAME: &str = "Desk0AgentService";

pub fn install() -> anyhow::Result<()> {
    let manager =
        ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CREATE_SERVICE)?;

    let service_info = ServiceInfo {
        name: SERVICE_NAME.into(),
        display_name: format!("{}", rust_i18n::t!("service_description")).into(),
        service_type: ServiceType::OWN_PROCESS,
        start_type: ServiceStartType::AutoStart,
        error_control: ServiceErrorControl::Severe,
        executable_path: std::env::current_exe()?,
        launch_arguments: vec!["--service".into()],
        dependencies: vec![],
        account_name: None,
        account_password: None,
    };

    manager.create_service(&service_info, ServiceAccess::START | ServiceAccess::STOP)?;

    Ok(())
}

pub fn uninstall() -> anyhow::Result<()> {
    let manager = ServiceManager::local_computer(None::<&str>, ServiceManagerAccess::CONNECT)?;
    let service = manager.open_service(SERVICE_NAME, ServiceAccess::DELETE)?;
    service.delete()?;

    Ok(())
}

pub fn start() -> anyhow::Result<()> {
    service_dispatcher::start(SERVICE_NAME, ffi_service_main)?;
    Ok(())
}
