use crate::cli::Cli;
use crate::util;
use anyhow::{Context, Result};
use serde_json::json;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

const IMAGE: &str = "localhost/opencode-container";
const LABEL_KEY: &str = "opencode.project.id";

pub fn run(cli: Cli) -> Result<()> {
    // --- Common setup ---
    let script_dir = find_script_dir()?;
    let container_dir = script_dir.join("container").canonicalize().unwrap_or_else(|_| {
        script_dir.join("container")
    });

    let pwd_resolved = util::resolve_path(Path::new("."))?;

    // Detect git root
    let git_root = if cli.no_git_root {
        None
    } else {
        detect_git_root(&pwd_resolved).ok()
    };

    let (code_dir, project_id) = if let Some(git_root) = git_root {
        let resolved = util::resolve_path(&git_root)?;
        let id = util::compute_project_id(&resolved.to_string_lossy());
        eprintln!("Git repo detected; mounting root: {}", resolved.display());
        (resolved, id)
    } else {
        let id = util::compute_project_id(&pwd_resolved.to_string_lossy());
        (pwd_resolved.clone(), id)
    };

    let data_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from(env::var("HOME").unwrap_or_else(|_| String::from("~"))))
        .join("opencode")
        .join("data")
        .join(&project_id);
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from(env::var("HOME").unwrap_or_else(|_| String::from("~"))))
        .join("opencode")
        .join("config")
        .join(&project_id);

    fs::create_dir_all(&data_dir)?;
    fs::create_dir_all(&config_dir)?;

    let mut env_files = cli.env_file.clone();

    // Auto-detect .env in project root
    let dotenv = code_dir.join(".env");
    if dotenv.exists() {
        env_files.push(dotenv.clone());
        eprintln!("Auto-detected env file: {}", dotenv.display());
    }

    // Hint if project has its own .devcontainer directory
    let devcontainer_json = code_dir.join(".devcontainer/devcontainer.json");
    if devcontainer_json.exists() {
        eprintln!("Hint: Found .devcontainer/devcontainer.json — use --feature-file .devcontainer/devcontainer.json to include its features");
    }

    // Detect web mode
    let (web_mode, web_port, custom_hostname, opencode_args) =
        util::detect_web_mode(&cli.opencode_args, cli.port);

    // --- Path B: Devcontainer mode ---
    if !cli.feature_file.is_empty() {
        return run_devcontainer(
            &cli, &container_dir, &code_dir, &data_dir, &config_dir, &project_id,
            web_mode, web_port, custom_hostname, &opencode_args, &env_files,
        );
    }

    // --- Path A: Fast path ---
    run_fast_path(
        &cli, &container_dir, &code_dir, &data_dir, &config_dir, &project_id,
        web_mode, web_port, custom_hostname, &opencode_args, &env_files,
    )
}

fn find_script_dir() -> Result<PathBuf> {
    // Allow explicit override
    if let Ok(dir) = env::var("OPENCODE_CONTAINER_DIR") {
        return Ok(PathBuf::from(dir));
    }

    let current_exe = env::current_exe()?;
    let mut dir = current_exe.parent().map(PathBuf::from);

    // Walk up looking for container/Containerfile.debian
    while let Some(ref d) = dir {
        if d.join("container/Containerfile.debian").exists() {
            return Ok(d.clone());
        }
        // Also check parent (for target/release/ or target/debug/)
        let parent = d.parent().map(PathBuf::from);
        if parent.as_ref() == dir.as_ref() {
            break;
        }
        dir = parent;
    }

    // Fallback: use exe directory and hope container/ is there
    if let Some(d) = current_exe.parent() {
        return Ok(d.to_path_buf());
    }

    anyhow::bail!("Could not determine script directory")
}

fn detect_git_root(cwd: &Path) -> Result<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(cwd)
        .output()?;
    if output.status.success() {
        let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(PathBuf::from(path))
    } else {
        anyhow::bail!("Not in a git repository")
    }
}

fn run_devcontainer(
    cli: &Cli,
    container_dir: &Path,
    code_dir: &Path,
    data_dir: &Path,
    config_dir: &Path,
    project_id: &str,
    web_mode: bool,
    web_port: u16,
    custom_hostname: bool,
    opencode_args: &[String],
    env_files: &[PathBuf],
) -> Result<()> {
    let devcontainer_cmd = util::devcontainer_cmd();
    if devcontainer_cmd.is_empty() {
        anyhow::bail!(
            "Error: devcontainer CLI is required for --feature-file.\nInstall one of:\n  npm install -g npx                     (recommended)\n  npm install -g @devcontainers/cli    (global install)"
        );
    }

    // Build base image if needed
    if cli.build || !image_exists(IMAGE)? {
        util::build_image(container_dir)?;
    }

    // Prevent duplicate containers
    if let Some(name) = find_running_container(project_id)? {
        anyhow::bail!("opencode is already running for this project: {}", name);
    }

    // Merge features from all --feature-file args
    let mut merged_features = json!({});
    for feature_file in &cli.feature_file {
        if !feature_file.exists() {
            anyhow::bail!("Error: feature file not found: {}", feature_file.display());
        }
        let content = fs::read_to_string(feature_file)
            .with_context(|| format!("Failed to read {}", feature_file.display()))?;
        let json: serde_json::Value = serde_json::from_str(&content)
            .with_context(|| format!("Invalid JSON in feature file: {}", feature_file.display()))?;
        let empty = json!({});
        let features = json.get("features").unwrap_or(&empty);
        if let Some(obj) = features.as_object() {
            if let Some(merged_obj) = merged_features.as_object_mut() {
                for (k, v) in obj {
                    merged_obj.insert(k.clone(), v.clone());
                }
            }
        }
    }

    // Build forwardPorts and runArgs for web mode
    let forward_ports = if web_mode { json!([web_port]) } else { json!([]) };
    let mut extra_run_args = vec![json!("--init")];

    if web_mode {
        let port = util::find_free_port(web_port);
        if port != web_port {
            eprintln!("Port {} in use, using port {} instead", web_port, port);
        }
        extra_run_args.push(json!(format!("-p={}:{}", port, port)));
    }

    // Build env-file args
    let mut env_args = vec![];
    for f in env_files {
        env_args.push(json!(format!("--env-file={}", f.display())));
    }

    // Build environment map
    let mut container_env = HashMap::new();
    container_env.insert("OPENCODE_DISABLE_DEFAULT_PLUGINS".to_string(), json!("true"));
    if cli.no_git_init {
        container_env.insert("OPENCODE_NO_GIT_INIT".to_string(), json!("1"));
    }
    for entry in &cli.env {
        if let Some((k, v)) = entry.split_once('=') {
            container_env.insert(k.to_string(), json!(v));
        }
    }
    for entry in &cli.local_env {
        container_env.insert(entry.clone(), json!(format!("${{localEnv:{}}}", entry)));
    }

    let cache_dir = dirs::cache_dir()
        .unwrap_or_else(|| PathBuf::from(env::var("HOME").unwrap_or_else(|_| String::from("~"))))
        .join("opencode")
        .join("cache")
        .join(project_id);
    fs::create_dir_all(&cache_dir)?;
    let devcontainer_json_path = cache_dir.join("devcontainer.json");

    let mut run_args = vec![
        json!(format!("--label={}={}", LABEL_KEY, project_id)),
        json!("--init"),
    ];
    run_args.extend(env_args);
    run_args.extend(extra_run_args);

    let mut config = json!({
        "name": "opencode-container",
        "image": IMAGE,
        "build": {
            "options": ["--progress=plain"]
        },
        "forwardPorts": forward_ports,
        "containerEnv": container_env,
        "mounts": [
            {"source": data_dir.to_string_lossy().to_string(), "target": "/home/node/.local/share/opencode", "type": "bind"},
            {"source": config_dir.to_string_lossy().to_string(), "target": "/home/node/.config/opencode", "type": "bind"},
            {"source": code_dir.to_string_lossy().to_string(), "target": "/code", "type": "bind"}
        ],
        "runArgs": run_args,
        "overrideCommand": false,
        "remoteUser": "node",
    });

    // Only add features key when non-empty to avoid CLI bugs
    if let Some(obj) = merged_features.as_object() {
        if !obj.is_empty() {
            config["features"] = merged_features;
        }
    }

    fs::write(&devcontainer_json_path, serde_json::to_string_pretty(&config)?)
        .with_context(|| "Failed to write devcontainer.json")?;
    eprintln!("Generated devcontainer config: {}", devcontainer_json_path.display());

    // Start the devcontainer
    let up_stdout = tempfile::NamedTempFile::new()?;
    eprintln!("Starting devcontainer... (features may take time to build)");

    let mut up_cmd = Command::new(&devcontainer_cmd[0]);
    for arg in &devcontainer_cmd[1..] {
        up_cmd.arg(arg);
    }
    up_cmd.args([
        "up",
        "--docker-path", "podman",
        "--workspace-folder", &code_dir.to_string_lossy(),
        "--config", &devcontainer_json_path.to_string_lossy(),
        "--remove-existing-container",
    ]);
    up_cmd.stdout(Stdio::from(up_stdout.reopen()?));

    let status = up_cmd.status()
        .with_context(|| "Failed to run devcontainer up")?;
    if !status.success() {
        anyhow::bail!("Error: devcontainer up failed");
    }

    // Find container ID for cleanup
    let container_id = find_container_id(project_id)?;

    // Setup cleanup
    let should_cleanup = Arc::new(AtomicBool::new(true));
    let cleanup_clone = Arc::clone(&should_cleanup);
    ctrlc::set_handler(move || {
        eprintln!("\nStopping opencode container...");
        cleanup_clone.store(false, Ordering::SeqCst);
    })?;

    // Web mode handling
    if web_mode {
        let port = util::find_free_port(web_port);
        let code_b64 = util::compute_project_id("/code");
        let url = format!("http://localhost:{}/{}", port, code_b64);

        if util::will_start_web_server(opencode_args) {
            println!("URL: {}", url);
            eprint!("Waiting for server to start");
            let mut http_ready = false;
            for _ in 0..30 {
                if util::port_is_open("localhost", port) {
                    http_ready = true;
                    break;
                }
                eprint!(".");
                thread::sleep(Duration::from_secs(1));
            }
            eprintln!();

            if !http_ready {
                eprintln!("Warning: server did not respond on port {}", port);
            }

            if http_ready && !cli.no_open && !custom_hostname {
                open_browser(&url)?;
            }
        }
    }

    // Exec opencode
    let mut exec_cmd = Command::new(&devcontainer_cmd[0]);
    for arg in &devcontainer_cmd[1..] {
        exec_cmd.arg(arg);
    }
    exec_cmd.args([
        "exec",
        "--docker-path", "podman",
        "--workspace-folder", &code_dir.to_string_lossy(),
        "--config", &devcontainer_json_path.to_string_lossy(),
        "opencode",
    ]);
    for arg in opencode_args {
        exec_cmd.arg(arg);
    }

    let status = exec_cmd.status()?;

    // Cleanup
    if should_cleanup.load(Ordering::SeqCst) {
        if let Some(cid) = container_id {
            let _ = Command::new(&devcontainer_cmd[0])
                .args(&devcontainer_cmd[1..])
                .args([
                    "stop",
                    "--docker-path", "podman",
                    "--workspace-folder", &code_dir.to_string_lossy(),
                    "--config", &devcontainer_json_path.to_string_lossy(),
                ])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
            let _ = Command::new(util::container_cmd())
                .args(["rm", "-f", &cid])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }
    }

    if let Some(code) = status.code() {
        if code != 0 {
            std::process::exit(code);
        }
    }

    Ok(())
}

fn run_fast_path(
    cli: &Cli,
    container_dir: &Path,
    code_dir: &Path,
    data_dir: &Path,
    config_dir: &Path,
    project_id: &str,
    web_mode: bool,
    web_port: u16,
    custom_hostname: bool,
    opencode_args: &[String],
    env_files: &[PathBuf],
) -> Result<()> {
    let cmd = util::container_cmd();

    // Build image if needed
    if cli.build || !image_exists(IMAGE)? {
        util::build_image(container_dir)?;
    }

    // Prevent duplicate containers
    if let Some(name) = find_running_container(project_id)? {
        anyhow::bail!("opencode is already running for this project: {}", name);
    }

    // Common container args
    let mut container_args = vec![
        format!("--label={}={}", LABEL_KEY, project_id),
        format!("-v={}:/home/node/.local/share/opencode:Z", data_dir.display()),
        format!("-v={}:/home/node/.config/opencode:Z", config_dir.display()),
        format!("-v={}:/code:Z", code_dir.display()),
        "-e=OPENCODE_DISABLE_DEFAULT_PLUGINS=true".to_string(),
        "-e=XDG_DATA_HOME=/home/node/.local/share".to_string(),
        "-e=XDG_CONFIG_HOME=/home/node/.config".to_string(),
        "-e=XDG_CACHE_HOME=/home/node/.cache".to_string(),
        "-e=XDG_STATE_HOME=/home/node/.local/state".to_string(),
    ];

    if cli.no_git_init {
        container_args.push("-e=OPENCODE_NO_GIT_INIT=1".to_string());
    }

    for f in env_files {
        container_args.push(format!("--env-file={}", f.display()));
    }

    for entry in &cli.env {
        container_args.push(format!("-e={}", entry));
    }

    for entry in &cli.local_env {
        if let Ok(value) = env::var(entry) {
            container_args.push(format!("-e={}={}", entry, value));
        }
    }

    if web_mode {
        let port = util::find_free_port(web_port);
        if port != web_port {
            eprintln!("Port {} in use, using port {} instead", web_port, port);
        }

        let code_b64 = util::compute_project_id("/code");
        let url = format!("http://localhost:{}/{}", port, code_b64);

        // Start container in background
        let mut run_cmd = Command::new(cmd);
        run_cmd.args(["run", "-i", "--rm", "--init"]);
        for arg in &container_args {
            run_cmd.arg(arg);
        }
        run_cmd.arg(format!("-p={}:{}", port, port));
        run_cmd.arg(IMAGE);
        for arg in opencode_args {
            run_cmd.arg(arg);
        }

        let mut child = run_cmd.spawn()?;
        let _container_pid = child.id();

        // Wait for container name
        let mut container_name = None;
        for _ in 0..20 {
            if let Some(name) = find_running_container(project_id)? {
                container_name = Some(name.clone());
                break;
            }
            thread::sleep(Duration::from_millis(100));
        }

        if container_name.is_none() {
            eprintln!("Error: container failed to start");
            let _ = child.kill();
            let _ = child.wait();
            anyhow::bail!("Container failed to start");
        }

        let name = container_name.unwrap();
        println!("opencode web container: {}", name);

        if util::will_start_web_server(opencode_args) {
            eprint!("Waiting for server to start");
            let mut http_ready = false;
            for _ in 0..30 {
                if util::port_is_open("localhost", port) {
                    http_ready = true;
                    break;
                }
                eprint!(".");
                thread::sleep(Duration::from_secs(1));
            }
            eprintln!();

            if !http_ready {
                eprintln!("Warning: server did not respond on port {}", port);
            }

            println!("URL: {}", url);

            if http_ready && !cli.no_open && !custom_hostname {
                open_browser(&url)?;
            }
        }

        // Setup cleanup
        let should_cleanup = Arc::new(AtomicBool::new(true));
        let cleanup_clone = Arc::clone(&should_cleanup);
        ctrlc::set_handler(move || {
            eprintln!("\nStopping opencode container...");
            cleanup_clone.store(false, Ordering::SeqCst);
        })?;

        // Wait for container
        let status = child.wait()?;

        if should_cleanup.load(Ordering::SeqCst) {
            let _ = Command::new(cmd)
                .args(["stop", "-t", "5", &name])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }

        if let Some(code) = status.code() {
            if code != 0 {
                std::process::exit(code);
            }
        }
    } else {
        // TUI mode
        let mut run_cmd = Command::new(cmd);
        run_cmd.args(["run", "-it", "--rm"]);
        for arg in &container_args {
            run_cmd.arg(arg);
        }
        run_cmd.arg(IMAGE);
        for arg in opencode_args {
            run_cmd.arg(arg);
        }

        let status = run_cmd.status()?;
        if let Some(code) = status.code() {
            if code != 0 {
                std::process::exit(code);
            }
        }
    }

    Ok(())
}

fn image_exists(image: &str) -> Result<bool> {
    let output = Command::new(util::container_cmd())
        .args(["image", "exists", image])
        .output()?;
    Ok(output.status.success())
}

fn find_running_container(project_id: &str) -> Result<Option<String>> {
    let output = Command::new(util::container_cmd())
        .args([
            "ps",
            "--filter", &format!("label={}={}", LABEL_KEY, project_id),
            "--format", "{{.Names}}",
        ])
        .output()?;
    if !output.status.success() {
        return Ok(None);
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let name = stdout.lines().next().map(|s| s.trim().to_string());
    Ok(name.filter(|s| !s.is_empty()))
}

fn find_container_id(project_id: &str) -> Result<Option<String>> {
    let output = Command::new(util::container_cmd())
        .args([
            "ps",
            "--filter", &format!("label={}={}", LABEL_KEY, project_id),
            "--format", "{{.ID}}",
        ])
        .output()?;
    if !output.status.success() {
        return Ok(None);
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let id = stdout.lines().next().map(|s| s.trim().to_string());
    Ok(id.filter(|s| !s.is_empty()))
}

fn open_browser(url: &str) -> Result<()> {
    if Command::new("open").arg(url).status().map(|s| s.success()).unwrap_or(false) {
        return Ok(());
    }
    if Command::new("xdg-open").arg(url).status().map(|s| s.success()).unwrap_or(false) {
        return Ok(());
    }
    eprintln!("Warning: could not auto-open browser (tried 'open' and 'xdg-open')");
    eprintln!("         Open manually: {}", url);
    Ok(())
}
