mod runtime;

use runtime::{ChangelogInfo, LauncherState, LauncherStatus};
use std::sync::Mutex;
use tauri::{Manager, RunEvent, State};

#[tauri::command]
async fn get_status(state: State<'_, Mutex<LauncherState>>) -> Result<LauncherStatus, String> {
    runtime::status_for_state(&state).await
}

#[tauri::command]
async fn perform_action(
    action: String,
    state: State<'_, Mutex<LauncherState>>,
) -> Result<LauncherStatus, String> {
    runtime::perform_action(action, state).await
}

#[tauri::command]
async fn choose_workspace(
    state: State<'_, Mutex<LauncherState>>,
) -> Result<LauncherStatus, String> {
    runtime::choose_workspace(state).await
}

#[tauri::command]
async fn fetch_changelog(version: String) -> Result<ChangelogInfo, String> {
    runtime::fetch_release_notes(&version).await
}

#[tauri::command]
fn open_service() -> Result<(), String> {
    open_url("http://127.0.0.1:3080")
}

#[tauri::command]
fn open_external(url: String) -> Result<(), String> {
    if !url.starts_with("https://") {
        return Err("Only HTTPS URLs are allowed".into());
    }
    open_url(&url)
}

fn open_url(url: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    let mut command = std::process::Command::new("open");
    #[cfg(target_os = "windows")]
    let mut command = {
        let mut cmd = std::process::Command::new("cmd");
        cmd.args(["/C", "start", "", url]);
        return cmd.spawn().map(|_| ()).map_err(|error| error.to_string());
    };
    #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
    let mut command = std::process::Command::new("xdg-open");

    command
        .arg(url)
        .spawn()
        .map(|_| ())
        .map_err(|error| error.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .setup(|app| {
            let paths = runtime::RuntimePaths::new(app.handle())?;
            app.manage(Mutex::new(LauncherState::new(paths)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_status,
            perform_action,
            choose_workspace,
            fetch_changelog,
            open_service,
            open_external
        ])
        .build(tauri::generate_context!())
        .expect("error while building DSH Launcher");

    app.run(|app_handle, event| {
        if let RunEvent::Exit = event
            && let Some(state) = app_handle.try_state::<Mutex<LauncherState>>()
            && let Ok(mut state) = state.lock()
        {
            state.shutdown();
        }
    });
}
