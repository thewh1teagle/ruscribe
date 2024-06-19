use crate::{cli, panic_hook};
use tauri::{App, Manager};

pub fn setup(app: &App) -> Result<(), Box<dyn std::error::Error>> {
    // Add panic hook
    panic_hook::set_panic_hook(app.app_handle());

    // Log some useful data
    if let Ok(version) = tauri::webview_version() {
        log::debug!("webview version: {}", version);
    }

    #[cfg(windows)]
    {
        if let Err(error) = crate::register_custom_protocol::register() {
            log::error!("{:?}", error);
        }
    }

    #[cfg(all(any(target_arch = "x86", target_arch = "x86_64"), target_os = "windows"))]
    log::debug!(
        "CPU Features\n{}",
        crate::cmd::get_x86_features()
            .map(|v| serde_json::to_string(&v).unwrap())
            .unwrap_or_default()
    );

    #[cfg(not(all(any(target_arch = "x86", target_arch = "x86_64"), target_os = "windows")))]
    log::debug!("CPU feature detection is not supported on this architecture.");

    log::debug!("COMMIT_HASH: {}", env!("COMMIT_HASH"));

    if cli::is_cli_detected() {
        cli::run(app);
    } else {
        // Create main window
        tauri::WebviewWindowBuilder::new(app, "main", tauri::WebviewUrl::App("index.html".into()))
            .inner_size(800.0, 700.0)
            .min_inner_size(800.0, 700.0)
            .center()
            .title("Vibe")
            .resizable(true)
            .focused(true)
            .shadow(true)
            .visible(false)
            .build()
            .expect("Can't create main window");
    }
    Ok(())
}
