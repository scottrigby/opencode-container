use anyhow::Result;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use std::fs;
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

pub fn resolve_path(path: &Path) -> Result<PathBuf> {
    let canonical = fs::canonicalize(path)?;
    Ok(canonical)
}

pub fn find_free_port(start: u16) -> u16 {
    let mut port = start;
    while port < u16::MAX {
        if !port_in_use(port) {
            return port;
        }
        port += 1;
    }
    eprintln!("Warning: all ports from {} upward appear in use", start);
    start
}

fn port_in_use(port: u16) -> bool {
    // Try macOS-style lsof first
    if Command::new("lsof")
        .args(["-Pi", &format!(":{}", port), "-sTCP:LISTEN", "-t"])
        .output()
        .map(|o| o.status.success() && !o.stdout.is_empty())
        .unwrap_or(false)
    {
        return true;
    }
    // Fall back to Linux ss
    if Command::new("ss")
        .args(["-tln"])
        .output()
        .map(|o| {
            o.status.success()
                && String::from_utf8_lossy(&o.stdout)
                    .contains(&format!("LISTEN.*:{}", port))
        })
        .unwrap_or(false)
    {
        return true;
    }
    false
}

pub fn port_is_open(host: &str, port: u16) -> bool {
    TcpStream::connect_timeout(
        &format!("{}:{}", host, port).parse().unwrap(),
        Duration::from_secs(2),
    )
    .is_ok()
}

pub fn will_start_web_server(args: &[String]) -> bool {
    for arg in args {
        match arg.as_str() {
            "--version" | "-v" | "--help" | "-h" | "help" => return false,
            _ => {}
        }
    }
    true
}

pub fn detect_web_mode(
    args: &[String],
    default_port: u16,
) -> (bool, u16, bool, Vec<String>) {
    let mut web_port = default_port;
    let mut custom_hostname = false;
    let mut custom_port = false;
    let mut result_args = args.to_vec();

    if args.is_empty() || args[0] != "web" {
        return (false, default_port, false, result_args);
    }

    let mut i = 1;
    while i < result_args.len() {
        if result_args[i] == "--port" && i + 1 < result_args.len() {
            if let Ok(p) = result_args[i + 1].parse::<u16>() {
                web_port = p;
                custom_port = true;
            }
        }
        if result_args[i] == "--hostname" {
            custom_hostname = true;
        }
        i += 1;
    }

    if !custom_hostname {
        result_args.push("--hostname".to_string());
        result_args.push("0.0.0.0".to_string());
    }
    if !custom_port {
        result_args.push("--port".to_string());
        result_args.push(default_port.to_string());
    }

    (true, web_port, custom_hostname, result_args)
}

pub fn build_image(container_dir: &Path) -> Result<()> {
    let dockerfile = container_dir.join("Containerfile.debian");
    if !dockerfile.exists() {
        anyhow::bail!(
            "Image not found and no {}. Please run: podman build -t localhost/opencode-container -f {} {}",
            dockerfile.display(),
            dockerfile.display(),
            container_dir.display()
        );
    }

    eprintln!("Building localhost/opencode-container ...");
    let status = Command::new("docker")
        .args([
            "build",
            "-t",
            "localhost/opencode-container",
            "-f",
            &dockerfile.to_string_lossy(),
            &container_dir.to_string_lossy(),
        ])
        .status()?;

    if !status.success() {
        anyhow::bail!("docker build failed");
    }
    Ok(())
}

pub fn decode_base64url(input: &str) -> Result<String> {
    let mut padded = input.to_string();
    let mod_len = input.len() % 4;
    if mod_len == 2 {
        padded.push_str("==");
    } else if mod_len == 3 {
        padded.push('=');
    }
    let standard = padded.replace('_', "/").replace('-', "+");
    let bytes = URL_SAFE_NO_PAD.decode(&standard)?;
    Ok(String::from_utf8_lossy(&bytes).to_string())
}

pub fn compute_project_id(path: &str) -> String {
    URL_SAFE_NO_PAD.encode(path.as_bytes())
}

pub fn container_cmd() -> &'static str {
    if Command::new("podman").arg("--version").status().is_ok() {
        "podman"
    } else {
        "docker"
    }
}

pub fn devcontainer_cmd() -> Vec<String> {
    if Command::new("devcontainer")
        .arg("--version")
        .status()
        .is_ok()
    {
        vec!["devcontainer".to_string()]
    } else if Command::new("npx")
        .args(["--yes", "@devcontainers/cli", "--version"])
        .status()
        .is_ok()
    {
        vec!["npx".to_string(), "--yes".to_string(), "@devcontainers/cli".to_string()]
    } else {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_project_id_roundtrip() {
        let path = "/home/user/my-project";
        let encoded = compute_project_id(path);
        let decoded = decode_base64url(&encoded).unwrap();
        assert_eq!(decoded, path);
    }

    #[test]
    fn test_decode_base64url_with_padding() {
        // Test that padding is correctly added for base64url decoding
        let input = "L2hvbWUvdXNlci9teS1wcm9qZWN0";
        let decoded = decode_base64url(input).unwrap();
        assert_eq!(decoded, "/home/user/my-project");
    }

    #[test]
    fn test_will_start_web_server() {
        assert!(will_start_web_server(&["web".to_string()])); // true - no help/version args
        assert!(!will_start_web_server(&["--help".to_string()])); // false - help arg
        assert!(!will_start_web_server(&["-h".to_string()])); // false - short help
        assert!(!will_start_web_server(&["--version".to_string()])); // false - version
        assert!(!will_start_web_server(&["-v".to_string()])); // false - short version
        assert!(!will_start_web_server(&["help".to_string()])); // false - help subcommand
    }

    #[test]
    fn test_detect_web_mode_no_web() {
        let args = vec!["other".to_string()];
        let (web_mode, port, custom_host, result_args) = detect_web_mode(&args, 4096);
        assert!(!web_mode);
        assert_eq!(port, 4096);
        assert!(!custom_host);
        assert_eq!(result_args, args);
    }

    #[test]
    fn test_detect_web_mode_with_web() {
        let args = vec!["web".to_string()];
        let (web_mode, port, custom_host, result_args) = detect_web_mode(&args, 4096);
        assert!(web_mode);
        assert_eq!(port, 4096);
        assert!(!custom_host);
        // Should inject --hostname 0.0.0.0 and --port 4096
        assert!(result_args.contains(&"--hostname".to_string()));
        assert!(result_args.contains(&"0.0.0.0".to_string()));
        assert!(result_args.contains(&"--port".to_string()));
    }

    #[test]
    fn test_detect_web_mode_custom_port() {
        let args = vec!["web".to_string(), "--port".to_string(), "5000".to_string()];
        let (web_mode, port, custom_host, _) = detect_web_mode(&args, 4096);
        assert!(web_mode);
        assert_eq!(port, 5000);
        assert!(!custom_host);
    }

    #[test]
    fn test_detect_web_mode_custom_hostname() {
        let args = vec![
            "web".to_string(),
            "--hostname".to_string(),
            "127.0.0.1".to_string(),
        ];
        let (web_mode, _, custom_host, result_args) = detect_web_mode(&args, 4096);
        assert!(web_mode);
        assert!(custom_host);
        // Should NOT inject --hostname since user provided it
        assert_eq!(
            result_args.iter().filter(|a| *a == "--hostname").count(),
            1
        );
    }

    #[test]
    fn test_port_is_open_localhost() {
        // Port 1 (TCPMUX) is very unlikely to be open in a test environment
        assert!(!port_is_open("127.0.0.1", 1));
    }
}
