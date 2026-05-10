use anyhow::Result;
use std::fs;
use std::path::PathBuf;

pub fn run() -> Result<()> {
    let data_root = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| String::from("~"))))
        .join("opencode")
        .join("data");

    if !data_root.exists() {
        eprintln!("No projects found in {}", data_root.display());
        return Ok(());
    }

    for entry in fs::read_dir(&data_root)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        let decoded = crate::util::decode_base64url(&name_str);
        match decoded {
            Ok(path) => {
                // Resolve symlinks for consistent display
                let resolved = std::fs::canonicalize(&path).unwrap_or_else(|_| PathBuf::from(&path));
                println!("{}", resolved.display());
            }
            Err(_) => {
                eprintln!("[unreadable]  {}", name_str);
            }
        }
    }

    Ok(())
}
