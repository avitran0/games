use crate::metadata;
use std::{
    env,
    ffi::OsStr,
    fs, io,
    path::{Path, PathBuf},
    time::Duration,
};

use ureq::Agent;

const API: &str = "https://api.github.com/repos/avitran0/games";
const WORKFLOW: &str = "rocknix-arm64.yml";
const ARTIFACT: &str = "games-rocknix-arm64";

pub enum Progress {
    Status(String),
    Finished,
    Failed(String),
}

pub fn run(mut report: impl FnMut(&str)) -> Result<(), String> {
    let ports_dir = ports_dir()?;
    let workspace = TempWorkspace::new(env::temp_dir()).map_err(display_error)?;
    let token = token(&ports_dir);
    let config = Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(300)))
        .build();
    let agent: Agent = config.into();

    report("Checking for the latest successful build…");
    let runs_url =
        format!("{API}/actions/workflows/{WORKFLOW}/runs?branch=main&status=success&per_page=1");
    let runs: WorkflowRuns = request_json(&agent, token.as_deref(), &runs_url)?;
    let run = runs
        .workflow_runs
        .first()
        .ok_or_else(|| "No successful ROCKNIX build is available on main.".to_owned())?;

    report("Finding the game update…");
    let artifacts_url = format!("{API}/actions/runs/{}/artifacts", run.id);
    let artifacts: Artifacts = request_json(&agent, token.as_deref(), &artifacts_url)?;
    let artifact = artifacts
        .artifacts
        .into_iter()
        .find(|artifact| artifact.name == ARTIFACT && !artifact.expired)
        .ok_or_else(|| format!("The game update is missing or has expired. ({ARTIFACT})"))?;

    report("Downloading game files…");
    let archive = workspace.path().join("build.zip");
    let download_url = format!("{API}/actions/artifacts/{}/zip", artifact.id);
    download(&agent, token.as_deref(), &download_url, &archive)?;

    report("Checking the downloaded files…");
    let build_dir = workspace.path().join("build");
    fs::create_dir(&build_dir).map_err(display_error)?;
    extract_archive(&archive, &build_dir)?;

    let sources = update_sources(&build_dir)?;
    report("Installing game files…");
    install(&ports_dir, &sources, &build_dir).map_err(display_error)?;
    Ok(())
}

#[derive(serde::Deserialize)]
struct WorkflowRuns {
    workflow_runs: Vec<WorkflowRun>,
}

#[derive(serde::Deserialize)]
struct WorkflowRun {
    id: u64,
}

#[derive(serde::Deserialize)]
struct Artifacts {
    artifacts: Vec<Artifact>,
}

#[derive(serde::Deserialize)]
struct Artifact {
    id: u64,
    name: String,
    expired: bool,
}

fn ports_dir() -> Result<PathBuf, String> {
    if let Some(path) = env::var_os("PORTS_DIR") {
        return Ok(PathBuf::from(path));
    }
    let executable_dir = env::current_exe()
        .map_err(|error| format!("Failed to locate the updater: {error}"))?
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| "Failed to find the updater's parent directory".to_owned())?;
    if executable_dir.file_name() == Some(OsStr::new("Updater")) {
        return executable_dir
            .parent()
            .map(Path::to_path_buf)
            .ok_or_else(|| "Failed to find the ports directory above the updater".to_owned());
    }
    Ok(executable_dir)
}

fn token(ports_dir: &Path) -> Option<String> {
    env::var("GH_TOKEN")
        .ok()
        .filter(|value| !value.is_empty())
        .or_else(|| {
            env::var("GITHUB_TOKEN")
                .ok()
                .filter(|value| !value.is_empty())
        })
        .or_else(|| {
            fs::read_to_string(ports_dir.join(".update-token"))
                .ok()
                .and_then(|value| value.lines().next().map(str::to_owned))
                .filter(|value| !value.is_empty())
        })
}

fn request_json<T: serde::de::DeserializeOwned>(
    agent: &Agent,
    token: Option<&str>,
    url: &str,
) -> Result<T, String> {
    let mut response = authorized_get(agent, token, url)
        .map_err(|error| format!("Failed to request data from GitHub: {error}"))?;
    response
        .body_mut()
        .read_json()
        .map_err(|error| format!("Failed to read the GitHub response: {error}"))
}

fn download(agent: &Agent, token: Option<&str>, url: &str, path: &Path) -> Result<(), String> {
    let mut response = authorized_get(agent, token, url)
        .map_err(|error| format!("Failed to download the game update: {error}"))?;
    let mut output = fs::File::create(path)
        .map_err(|error| format!("Failed to create the download file: {error}"))?;
    let mut reader = response.body_mut().as_reader();
    io::copy(&mut reader, &mut output)
        .map_err(|error| format!("Failed to save the game update: {error}"))?;
    output
        .sync_all()
        .map_err(|error| format!("Failed to finish saving the game update: {error}"))
}

fn authorized_get(
    agent: &Agent,
    token: Option<&str>,
    url: &str,
) -> Result<ureq::http::Response<ureq::Body>, ureq::Error> {
    let request = agent.get(url);
    let request = match token {
        Some(token) => request.header("Authorization", &format!("Bearer {token}")),
        None => request,
    };
    request.call()
}

fn extract_archive(archive_path: &Path, destination: &Path) -> Result<(), String> {
    let archive_file = fs::File::open(archive_path)
        .map_err(|error| format!("Failed to open the game update: {error}"))?;
    let mut archive = zip::ZipArchive::new(archive_file)
        .map_err(|error| format!("Failed to read the game update: {error}"))?;

    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|error| format!("Failed to read an update file: {error}"))?;
        let name = entry
            .name()
            .map_err(|error| format!("Failed to read an update file name: {error}"))?
            .into_owned();
        if entry.is_symlink() {
            return Err(format!("The game update contains a symbolic link: {name}"));
        }
        let relative = entry
            .enclosed_name()
            .ok_or_else(|| format!("The game update contains an unsafe path: {name}"))?;
        let path = destination.join(relative);
        if entry.is_dir() {
            fs::create_dir_all(&path)
                .map_err(|error| format!("Failed to create an update folder: {error}"))?;
            continue;
        }

        let parent = path
            .parent()
            .ok_or_else(|| format!("The game update contains an invalid path: {name}"))?;
        fs::create_dir_all(parent)
            .map_err(|error| format!("Failed to create an update folder: {error}"))?;
        let mut output = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .map_err(|error| format!("Failed to create an update file: {error}"))?;
        io::copy(&mut entry, &mut output)
            .map_err(|error| format!("Failed to extract an update file: {error}"))?;
        output
            .sync_all()
            .map_err(|error| format!("Failed to save an update file: {error}"))?;
    }
    Ok(())
}

fn update_sources(build_dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut sources = Vec::new();
    for entry in fs::read_dir(build_dir).map_err(display_error)? {
        let path = entry.map_err(display_error)?.path();
        let metadata = fs::symlink_metadata(&path).map_err(display_error)?;
        if metadata.file_type().is_symlink() {
            return Err(format!(
                "The updater does not install symbolic links: {}",
                path.display()
            ));
        }
        if !metadata.is_file() && !metadata.is_dir() {
            return Err(format!(
                "The update contains a file type that the updater does not support: {}",
                path.display()
            ));
        }
        sources.push(path);
    }
    sources.sort();
    if !sources.iter().any(|path| {
        path.extension() == Some(OsStr::new("sh"))
            && fs::symlink_metadata(path).is_ok_and(|metadata| metadata.is_file())
    }) {
        return Err("The update has no game launcher scripts".into());
    }
    Ok(sources)
}

fn install(ports_dir: &Path, sources: &[PathBuf], build_dir: &Path) -> io::Result<()> {
    fs::create_dir_all(ports_dir)?;
    let staging = TempWorkspace::new(ports_dir.to_path_buf())?;
    let new_files = staging.path().join("new");
    let backups = staging.path().join("backups");
    fs::create_dir(&new_files)?;
    fs::create_dir(&backups)?;

    let script_name = env::current_exe()
        .ok()
        .and_then(|path| path.file_name().map(OsStr::to_os_string));
    let log_name = env::var_os("UPDATE_LOG")
        .and_then(|path| PathBuf::from(path).file_name().map(OsStr::to_os_string))
        .unwrap_or_else(|| OsStr::new("update.log").to_os_string());
    let mut names = Vec::new();

    for source in sources {
        let name = source.file_name().expect("artifact entries have names");
        if name == OsStr::new("gamelist-metadata.xml")
            || Some(name) == script_name.as_deref()
            || name == log_name
        {
            continue;
        }
        copy_entry(source, &new_files.join(name))?;
        set_launcher_permissions(&new_files.join(name))?;
        names.push(name.to_os_string());
    }
    if names.is_empty() {
        return Err(io::Error::other("The update has no files to install"));
    }

    let metadata_source = build_dir.join("gamelist-metadata.xml");
    if metadata_source.is_file() {
        let target = ports_dir.join("gamelist.xml");
        let original = if target.exists() {
            Some(fs::read_to_string(&target)?)
        } else {
            None
        };
        let incoming = fs::read_to_string(&metadata_source)?;
        let merged = metadata::merge(original.as_deref(), &incoming)
            .map_err(|error| io::Error::other(format!("Failed to merge game metadata: {error}")))?;
        fs::write(new_files.join("gamelist.xml"), merged)?;
        if target.exists() {
            fs::copy(&target, ports_dir.join("gamelist.xml.bak"))?;
        }
        names.push(OsStr::new("gamelist.xml").to_os_string());
    }

    let mut committed: Vec<(PathBuf, Option<PathBuf>)> = Vec::new();
    for name in names {
        let target = ports_dir.join(&name);
        let staged = new_files.join(&name);
        let backup = backups.join(&name);
        let had_target = target.exists();
        if had_target && let Err(error) = fs::rename(&target, &backup) {
            rollback(&committed);
            return Err(error);
        }
        if let Err(error) = fs::rename(&staged, &target) {
            if had_target {
                let _ = fs::rename(&backup, &target);
            }
            rollback(&committed);
            return Err(error);
        }
        committed.push((target, had_target.then_some(backup)));
    }
    Ok(())
}

fn copy_entry(source: &Path, destination: &Path) -> io::Result<()> {
    let metadata = fs::symlink_metadata(source)?;
    if metadata.file_type().is_symlink() {
        return Err(io::Error::other(format!(
            "The updater will not copy a symbolic link: {}",
            source.display()
        )));
    }
    if metadata.is_dir() {
        fs::create_dir(destination)?;
        for entry in fs::read_dir(source)? {
            let entry = entry?;
            copy_entry(&entry.path(), &destination.join(entry.file_name()))?;
        }
        fs::set_permissions(destination, metadata.permissions())?;
    } else if metadata.is_file() {
        fs::copy(source, destination)?;
    } else {
        return Err(io::Error::other(format!(
            "The update contains a file type that the updater does not support: {}",
            source.display()
        )));
    }
    Ok(())
}

fn set_launcher_permissions(path: &Path) -> io::Result<()> {
    let name = path.file_name();
    if path.is_file()
        && (path.extension() == Some(OsStr::new("sh")) || name == Some(OsStr::new("game")))
    {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(path, fs::Permissions::from_mode(0o755))?;
        }
    }
    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            set_launcher_permissions(&entry?.path())?;
        }
    }
    Ok(())
}

fn rollback(committed: &[(PathBuf, Option<PathBuf>)]) {
    for (target, backup) in committed.iter().rev() {
        let _ = remove_entry(target);
        if let Some(backup) = backup {
            let _ = fs::rename(backup, target);
        }
    }
}

fn remove_entry(path: &Path) -> io::Result<()> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error),
    };
    if metadata.is_dir() {
        fs::remove_dir_all(path)
    } else {
        fs::remove_file(path)
    }
}

fn display_error(error: impl std::fmt::Display) -> String {
    error.to_string()
}

struct TempWorkspace(PathBuf);

impl TempWorkspace {
    fn new(parent: PathBuf) -> io::Result<Self> {
        for attempt in 0..100 {
            let path = parent.join(format!(".updater-{}-{attempt}", std::process::id()));
            match fs::create_dir(&path) {
                Ok(()) => {
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        fs::set_permissions(&path, fs::Permissions::from_mode(0o700))?;
                    }
                    return Ok(Self(path));
                }
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
                Err(error) => return Err(error),
            }
        }
        Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "Failed to create a temporary updater directory",
        ))
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
