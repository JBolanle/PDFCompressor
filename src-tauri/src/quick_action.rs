use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

/// Filename of the Quick Action bundle as it ships in Resources and as it
/// must be named in ~/Library/Services/ for macOS Launch Services to pick
/// it up. Square brackets are avoided so Tauri's `bundle.resources` glob
/// doesn't read them as a character class; the user-facing menu label is
/// set separately by `NSMenuItem.default` inside Contents/Info.plist.
const WORKFLOW_BUNDLE_NAME: &str = "Compress PDF.workflow";

/// Absolute path where the Quick Action lives once installed.
/// `services_dir` is the user's `~/Library/Services` directory; injecting
/// it as a parameter keeps the function pure and testable.
pub fn services_install_path(services_dir: &Path) -> PathBuf {
    services_dir.join(WORKFLOW_BUNDLE_NAME)
}

/// Default `~/Library/Services` location, resolved from the OS home dir.
/// Returns None only on bizarre systems with no home directory.
fn default_services_dir() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;
    Some(PathBuf::from(home).join("Library").join("Services"))
}

/// Recursively copy a directory tree from `src` to `dst`. If `dst` exists
/// it is wiped first so an upgrade replaces the previous install rather
/// than mixing old + new files.
pub fn copy_workflow_bundle(src: &Path, dst: &Path) -> std::io::Result<()> {
    if dst.exists() {
        std::fs::remove_dir_all(dst)?;
    }
    copy_dir_recursive(src, dst)?;
    Ok(())
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_recursive(&from, &to)?;
        } else {
            std::fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

fn bundled_workflow_path(app: &AppHandle) -> Result<PathBuf, String> {
    let resource_dir = app.path().resource_dir().map_err(|e| e.to_string())?;
    let candidate = resource_dir.join("resources").join(WORKFLOW_BUNDLE_NAME);
    if candidate.exists() {
        return Ok(candidate);
    }
    // Some Tauri builds flatten resources/ — fall back to the resource root.
    let flat = resource_dir.join(WORKFLOW_BUNDLE_NAME);
    if flat.exists() {
        return Ok(flat);
    }
    Err(format!(
        "Bundled Quick Action not found in resource directory: {}",
        resource_dir.display()
    ))
}

#[tauri::command]
pub fn is_quick_action_installed() -> bool {
    let Some(services) = default_services_dir() else {
        return false;
    };
    services_install_path(&services).is_dir()
}

#[tauri::command]
pub fn install_quick_action(app: AppHandle) -> Result<(), String> {
    let src = bundled_workflow_path(&app)?;
    let services = default_services_dir().ok_or("Could not locate ~/Library/Services")?;
    std::fs::create_dir_all(&services).map_err(|e| e.to_string())?;
    let dst = services_install_path(&services);
    copy_workflow_bundle(&src, &dst).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn uninstall_quick_action() -> Result<(), String> {
    let services = default_services_dir().ok_or("Could not locate ~/Library/Services")?;
    let dst = services_install_path(&services);
    if dst.exists() {
        std::fs::remove_dir_all(&dst).map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn write_file(path: &Path, contents: &[u8]) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        let mut f = std::fs::File::create(path).unwrap();
        f.write_all(contents).unwrap();
    }

    #[test]
    fn services_install_path_joins_bundle_name() {
        let services = PathBuf::from("/Users/x/Library/Services");
        let path = services_install_path(&services);
        assert_eq!(
            path,
            PathBuf::from("/Users/x/Library/Services/Compress PDF.workflow")
        );
    }

    #[test]
    fn copy_workflow_bundle_copies_full_tree() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("src.workflow");
        let dst = tmp.path().join("services").join("dst.workflow");

        write_file(&src.join("Contents/Info.plist"), b"<plist/>");
        write_file(&src.join("Contents/document.wflow"), b"<plist/>");
        write_file(&src.join("Contents/Resources/icon.png"), b"\x89PNG\r\n");

        copy_workflow_bundle(&src, &dst).unwrap();

        assert!(dst.join("Contents/Info.plist").exists());
        assert!(dst.join("Contents/document.wflow").exists());
        assert!(dst.join("Contents/Resources/icon.png").exists());
        assert_eq!(
            std::fs::read(dst.join("Contents/Info.plist")).unwrap(),
            b"<plist/>"
        );
    }

    #[test]
    fn copy_workflow_bundle_replaces_existing_destination() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("src.workflow");
        let dst = tmp.path().join("dst.workflow");

        // Old contents at destination — must not survive the second copy.
        write_file(&dst.join("Contents/old-file.txt"), b"stale");

        // Fresh source.
        write_file(&src.join("Contents/Info.plist"), b"new");

        copy_workflow_bundle(&src, &dst).unwrap();

        assert!(dst.join("Contents/Info.plist").exists());
        assert!(
            !dst.join("Contents/old-file.txt").exists(),
            "stale file from previous install should have been removed"
        );
    }

    #[test]
    fn copy_workflow_bundle_creates_parent_dirs() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("src.workflow");
        // Destination's parent doesn't exist yet.
        let dst = tmp.path().join("nested/parent/dst.workflow");

        write_file(&src.join("Contents/Info.plist"), b"x");

        copy_workflow_bundle(&src, &dst).unwrap();
        assert!(dst.join("Contents/Info.plist").exists());
    }
}
