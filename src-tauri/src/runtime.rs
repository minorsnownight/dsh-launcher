use semver::Version;
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::Mutex,
    time::Duration,
};
use tauri::{AppHandle, Manager, State};

const PACKAGE_NAME: &str = "@deepseek-ai/dsh";
const SERVICE_URL: &str = "http://127.0.0.1:3080";

#[derive(Clone)]
pub struct RuntimePaths {
    pub data_dir: PathBuf,
    pub runtime_dir: PathBuf,
    pub settings_file: PathBuf,
    pub default_workspace: PathBuf,
}

impl RuntimePaths {
    pub fn new(app: &AppHandle) -> Result<Self, Box<dyn std::error::Error>> {
        let data_dir = app.path().app_data_dir()?;
        let default_workspace = app.path().home_dir()?;
        Ok(Self {
            runtime_dir: data_dir.join("runtime"),
            settings_file: data_dir.join("settings.json"),
            data_dir,
            default_workspace,
        })
    }

    fn package_dir(&self) -> PathBuf {
        self.runtime_dir.join("node_modules/@deepseek-ai/dsh")
    }

    fn workspace(&self) -> PathBuf {
        read_settings(&self.settings_file)
            .map(|settings| PathBuf::from(settings.workspace))
            .filter(|path| path.is_dir())
            .unwrap_or_else(|| self.default_workspace.clone())
    }
}

pub struct LauncherState {
    pub paths: RuntimePaths,
    child: Option<Child>,
}

impl LauncherState {
    pub fn new(paths: RuntimePaths) -> Self {
        Self { paths, child: None }
    }

    pub fn shutdown(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LauncherStatus {
    pub installed: bool,
    pub installed_version: Option<String>,
    pub runtime_source: Option<String>,
    pub latest_version: Option<String>,
    pub update_available: bool,
    pub service: String,
    pub service_origin: Option<String>,
    pub service_url: String,
    pub node_available: bool,
    pub workspace: String,
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct PackageManifest {
    version: String,
}

#[derive(Serialize, Deserialize)]
struct RegistryMetadata {
    #[serde(rename = "dist-tags")]
    dist_tags: DistTags,
}

#[derive(Serialize, Deserialize)]
struct DistTags {
    latest: String,
}

#[derive(Serialize, Deserialize)]
struct Settings {
    workspace: String,
}

struct RuntimeInstallation {
    version: String,
    executable: PathBuf,
    source: &'static str,
}

struct ServiceProcess {
    pid: u32,
}

pub async fn status_for_state(
    state: &State<'_, Mutex<LauncherState>>,
) -> Result<LauncherStatus, String> {
    let (paths, managed) = {
        let mut state = state
            .lock()
            .map_err(|_| "Unable to access launcher state")?;
        let managed = match state.child.as_mut() {
            Some(child) => match child.try_wait() {
                Ok(None) => true,
                _ => {
                    state.child = None;
                    false
                }
            },
            None => false,
        };
        (state.paths.clone(), managed)
    };
    get_status(&paths, managed).await
}

pub async fn get_status(paths: &RuntimePaths, managed: bool) -> Result<LauncherStatus, String> {
    let installation = find_runtime(paths);
    let installed_version = installation.as_ref().map(|runtime| runtime.version.clone());
    let runtime_source = installation
        .as_ref()
        .map(|runtime| runtime.source.to_string());
    let latest_version = latest_version().await;
    let healthy = service_is_healthy().await;
    let service_process = if healthy {
        find_service_process()
    } else {
        None
    };
    let service = if healthy && (managed || service_process.is_some()) {
        "running"
    } else if healthy {
        "external"
    } else {
        "stopped"
    };
    let service_origin = if healthy && managed {
        Some("launcher".into())
    } else if healthy && service_process.is_some() {
        Some("terminal".into())
    } else if healthy {
        Some("unknown".into())
    } else {
        None
    };

    Ok(LauncherStatus {
        installed: installed_version.is_some(),
        runtime_source,
        update_available: versions_differ(installed_version.as_deref(), latest_version.as_deref()),
        installed_version,
        latest_version,
        service: service.into(),
        service_origin,
        service_url: SERVICE_URL.into(),
        node_available: find_command("node").is_some() && find_command(npm_command()).is_some(),
        workspace: paths.workspace().to_string_lossy().into_owned(),
        error: None,
    })
}

pub async fn perform_action(
    action: String,
    state: State<'_, Mutex<LauncherState>>,
) -> Result<LauncherStatus, String> {
    match action.as_str() {
        "install" | "update" => {
            let paths = state
                .lock()
                .map_err(|_| "Unable to access launcher state")?
                .paths
                .clone();
            install_runtime(&paths).await?;
        }
        "start" => start_service(&state).await?,
        "restart" => {
            stop_service(&state).await?;
            tokio::time::sleep(Duration::from_millis(400)).await;
            start_service(&state).await?;
        }
        "stop" => stop_service(&state).await?,
        _ => return Err("Unknown launcher action".into()),
    }

    status_for_state(&state).await
}

pub async fn choose_workspace(
    state: State<'_, Mutex<LauncherState>>,
) -> Result<LauncherStatus, String> {
    {
        let mut state = state
            .lock()
            .map_err(|_| "Unable to access launcher state")?;
        if matches!(state.child.as_mut().map(Child::try_wait), Some(Ok(None))) {
            return Err("Stop DSH before changing the workspace".into());
        }
    }

    let selected = rfd::AsyncFileDialog::new().pick_folder().await;
    if let Some(folder) = selected {
        let paths = state
            .lock()
            .map_err(|_| "Unable to access launcher state")?
            .paths
            .clone();
        fs::create_dir_all(&paths.data_dir).map_err(friendly_io_error)?;
        let settings = Settings {
            workspace: folder.path().to_string_lossy().into_owned(),
        };
        fs::write(
            &paths.settings_file,
            serde_json::to_vec_pretty(&settings).map_err(|error| error.to_string())?,
        )
        .map_err(friendly_io_error)?;
    }
    status_for_state(&state).await
}

async fn install_runtime(paths: &RuntimePaths) -> Result<(), String> {
    let npm = find_command(npm_command()).ok_or("Node.js and npm are required")?;
    fs::create_dir_all(&paths.runtime_dir).map_err(friendly_io_error)?;

    let output = tokio::process::Command::new(npm)
        .args([
            "install",
            "--no-save",
            "--no-audit",
            "--no-fund",
            "--prefix",
        ])
        .arg(&paths.runtime_dir)
        .arg(format!("{PACKAGE_NAME}@latest"))
        .output()
        .await
        .map_err(|error| format!("Could not start npm: {error}"))?;

    if output.status.success() {
        Ok(())
    } else {
        let message = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(if message.is_empty() {
            "DSH installation failed".into()
        } else {
            message
        })
    }
}

async fn start_service(state: &State<'_, Mutex<LauncherState>>) -> Result<(), String> {
    if service_is_healthy().await {
        return Err("A DSH service is already running on port 3080".into());
    }

    let paths = state
        .lock()
        .map_err(|_| "Unable to access launcher state")?
        .paths
        .clone();
    let installation = find_runtime(&paths).ok_or("Install DSH before starting it")?;
    if !installation.executable.is_file() {
        return Err("Install DSH before starting it".into());
    }
    let node = find_command("node").ok_or("Node.js is required")?;
    fs::create_dir_all(&paths.data_dir).map_err(friendly_io_error)?;
    let log = fs::File::create(paths.data_dir.join("dsh.log")).map_err(friendly_io_error)?;
    let log_error = log.try_clone().map_err(friendly_io_error)?;

    let child = Command::new(node)
        .arg(installation.executable)
        .arg("web")
        .current_dir(paths.workspace())
        .stdin(Stdio::null())
        .stdout(Stdio::from(log))
        .stderr(Stdio::from(log_error))
        .spawn()
        .map_err(|error| format!("Could not start DSH: {error}"))?;

    state
        .lock()
        .map_err(|_| "Unable to access launcher state")?
        .child = Some(child);

    for _ in 0..40 {
        if service_is_healthy().await {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }
    Err("DSH did not become ready within 10 seconds. Check dsh.log in the app data folder.".into())
}

async fn stop_service(state: &State<'_, Mutex<LauncherState>>) -> Result<(), String> {
    let child = state
        .lock()
        .map_err(|_| "Unable to access launcher state")?
        .child
        .take();

    if let Some(mut child) = child {
        child
            .kill()
            .map_err(|error| format!("Could not stop DSH: {error}"))?;
        let _ = child.wait();
    } else if let Some(process) = find_service_process() {
        terminate_process(process.pid)?;
    } else if service_is_healthy().await {
        return Err("Port 3080 is in use by a process that could not be verified as DSH".into());
    } else {
        return Ok(());
    }

    for _ in 0..30 {
        if !service_is_healthy().await {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    Err("DSH did not stop within 3 seconds".into())
}

fn find_runtime(paths: &RuntimePaths) -> Option<RuntimeInstallation> {
    runtime_at(&paths.package_dir(), "managed")
        .or_else(find_global_runtime)
        .or_else(|| find_npx_runtime(paths))
}

fn runtime_at(package_dir: &Path, source: &'static str) -> Option<RuntimeInstallation> {
    let contents = fs::read(package_dir.join("package.json")).ok()?;
    let manifest = serde_json::from_slice::<PackageManifest>(&contents).ok()?;
    let executable = package_dir.join("lib/bin.js");
    executable.is_file().then_some(RuntimeInstallation {
        version: manifest.version,
        executable,
        source,
    })
}

fn find_global_runtime() -> Option<RuntimeInstallation> {
    let npm = find_command(npm_command())?;
    let output = Command::new(npm).args(["root", "-g"]).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let root = String::from_utf8(output.stdout).ok()?;
    runtime_at(
        &PathBuf::from(root.trim()).join("@deepseek-ai/dsh"),
        "global",
    )
}

fn find_npx_runtime(paths: &RuntimePaths) -> Option<RuntimeInstallation> {
    let mut cache_roots = vec![paths.default_workspace.join(".npm/_npx")];

    #[cfg(target_os = "windows")]
    if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
        cache_roots.push(PathBuf::from(local_app_data).join("npm-cache/_npx"));
    }

    if let Some(npm) = find_command(npm_command())
        && let Ok(output) = Command::new(npm).args(["config", "get", "cache"]).output()
        && output.status.success()
        && let Ok(cache) = String::from_utf8(output.stdout)
    {
        cache_roots.push(PathBuf::from(cache.trim()).join("_npx"));
    }

    cache_roots
        .into_iter()
        .filter_map(|root| fs::read_dir(root).ok())
        .flatten()
        .filter_map(Result::ok)
        .filter_map(|entry| runtime_at(&entry.path().join("node_modules/@deepseek-ai/dsh"), "npx"))
        .max_by(|left, right| compare_versions(&left.version, &right.version))
}

fn compare_versions(left: &str, right: &str) -> std::cmp::Ordering {
    match (Version::parse(left), Version::parse(right)) {
        (Ok(left), Ok(right)) => left.cmp(&right),
        _ => left.cmp(right),
    }
}

fn is_dsh_command(command: &str) -> bool {
    let normalized = command.to_ascii_lowercase().replace('\\', "/");
    normalized.contains("@deepseek-ai/dsh")
        || normalized.contains("node_modules/.bin/dsh")
        || normalized.contains("@deepseek-ai/dsh/lib/bin.js")
}

#[cfg(not(target_os = "windows"))]
fn find_service_process() -> Option<ServiceProcess> {
    let output = Command::new("lsof")
        .args(["-nP", "-iTCP:3080", "-sTCP:LISTEN", "-t"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let pid = String::from_utf8(output.stdout)
        .ok()?
        .lines()
        .next()?
        .trim()
        .parse::<u32>()
        .ok()?;
    let command = Command::new("ps")
        .args(["-p", &pid.to_string(), "-o", "command="])
        .output()
        .ok()?;
    let command = String::from_utf8(command.stdout).ok()?;
    is_dsh_command(&command).then_some(ServiceProcess { pid })
}

#[cfg(target_os = "windows")]
fn find_service_process() -> Option<ServiceProcess> {
    let output = Command::new("netstat.exe")
        .args(["-ano", "-p", "TCP"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let pid = text.lines().find_map(parse_windows_listener_pid)?;
    let script = format!("(Get-CimInstance Win32_Process -Filter 'ProcessId = {pid}').CommandLine");
    let output = Command::new("powershell.exe")
        .args(["-NoProfile", "-Command", &script])
        .output()
        .ok()?;
    let command = String::from_utf8_lossy(&output.stdout);
    is_dsh_command(&command).then_some(ServiceProcess { pid })
}

#[cfg(target_os = "windows")]
fn parse_windows_listener_pid(line: &str) -> Option<u32> {
    let parts: Vec<_> = line.split_whitespace().collect();
    if parts.len() < 5 || parts[0] != "TCP" || parts[3] != "LISTENING" {
        return None;
    }
    let local_address = parts[1];
    local_address
        .ends_with(":3080")
        .then(|| parts[4].parse().ok())
        .flatten()
}

#[cfg(not(target_os = "windows"))]
fn terminate_process(pid: u32) -> Result<(), String> {
    let status = Command::new("kill")
        .args(["-TERM", &pid.to_string()])
        .status()
        .map_err(|error| format!("Could not stop terminal DSH: {error}"))?;
    status
        .success()
        .then_some(())
        .ok_or_else(|| "Could not stop terminal DSH".into())
}

#[cfg(target_os = "windows")]
fn terminate_process(pid: u32) -> Result<(), String> {
    let status = Command::new("taskkill.exe")
        .args(["/PID", &pid.to_string(), "/T"])
        .status()
        .map_err(|error| format!("Could not stop terminal DSH: {error}"))?;
    status
        .success()
        .then_some(())
        .ok_or_else(|| "Could not stop terminal DSH".into())
}

async fn latest_version() -> Option<String> {
    if let Ok(client) = reqwest::Client::builder()
        .timeout(Duration::from_secs(3))
        .build()
        && let Ok(response) = client
            .get("https://registry.npmjs.org/@deepseek-ai%2Fdsh")
            .send()
            .await
        && let Ok(metadata) = response.json::<RegistryMetadata>().await
    {
        return Some(metadata.dist_tags.latest);
    }

    let npm = find_command(npm_command())?;
    let output = tokio::process::Command::new(npm)
        .args(["view", PACKAGE_NAME, "version", "--json"])
        .output()
        .await
        .ok()?;
    output.status.success().then(|| {
        String::from_utf8_lossy(&output.stdout)
            .trim()
            .trim_matches('"')
            .to_string()
    })
}

async fn service_is_healthy() -> bool {
    let Ok(client) = reqwest::Client::builder()
        .timeout(Duration::from_millis(700))
        .build()
    else {
        return false;
    };
    client.get(SERVICE_URL).send().await.is_ok()
}

fn versions_differ(installed: Option<&str>, latest: Option<&str>) -> bool {
    let (Some(installed), Some(latest)) = (installed, latest) else {
        return false;
    };
    match (Version::parse(installed), Version::parse(latest)) {
        (Ok(installed), Ok(latest)) => latest > installed,
        _ => installed != latest,
    }
}

fn npm_command() -> &'static str {
    if cfg!(target_os = "windows") {
        "npm.cmd"
    } else {
        "npm"
    }
}

fn find_command(name: &str) -> Option<PathBuf> {
    if Command::new(name)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|s| s.success())
    {
        return Some(PathBuf::from(name));
    }

    let common = if cfg!(target_os = "windows") {
        vec![]
    } else {
        vec![
            PathBuf::from("/opt/homebrew/bin").join(name),
            PathBuf::from("/usr/local/bin").join(name),
        ]
    };
    common
        .into_iter()
        .find(|path| path.is_file())
        .or_else(|| find_in_login_shell(name))
}

#[cfg(not(target_os = "windows"))]
fn find_in_login_shell(name: &str) -> Option<PathBuf> {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/zsh".into());
    let output = Command::new(shell)
        .args(["-lc", &format!("command -v {name}")])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let path = PathBuf::from(String::from_utf8(output.stdout).ok()?.trim());
    path.is_file().then_some(path)
}

#[cfg(target_os = "windows")]
fn find_in_login_shell(name: &str) -> Option<PathBuf> {
    let output = Command::new("where.exe").arg(name).output().ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout)
        .ok()?
        .lines()
        .next()
        .map(PathBuf::from)
}

fn read_settings(path: &Path) -> Option<Settings> {
    serde_json::from_slice(&fs::read(path).ok()?).ok()
}

fn friendly_io_error(error: std::io::Error) -> String {
    error.to_string()
}

#[cfg(test)]
mod tests {
    use super::{compare_versions, is_dsh_command, versions_differ};

    #[test]
    fn update_detection_understands_prerelease_versions() {
        assert!(versions_differ(Some("0.1.0-rc.5"), Some("0.1.0-rc.6")));
        assert!(!versions_differ(Some("0.1.0-rc.6"), Some("0.1.0-rc.6")));
        assert!(!versions_differ(None, Some("0.1.0-rc.6")));
    }

    #[test]
    fn runtime_candidates_use_the_newest_semver() {
        assert!(compare_versions("0.1.0-rc.6", "0.1.0-rc.5").is_gt());
    }

    #[test]
    fn only_verified_dsh_commands_are_manageable() {
        assert!(is_dsh_command(
            "node /Users/me/.npm/_npx/abc/node_modules/.bin/dsh web"
        ));
        assert!(is_dsh_command(
            r#"node C:\cache\node_modules\@deepseek-ai\dsh\lib\bin.js web"#
        ));
        assert!(!is_dsh_command("python -m http.server 3080"));
    }
}
