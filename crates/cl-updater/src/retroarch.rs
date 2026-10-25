//! Curated Windows core pack. Separate from runtime versions and user saves.
use crate::{sources, ManagedEmulator, Release, UpdateError};
use std::path::{Path, PathBuf};

pub fn ensure_core(core: &str, runtime: &Path, emulators: &Path) -> Result<PathBuf, UpdateError> {
    if !cfg!(windows) {
        return Err(UpdateError::Other(
            "Automatic Libretro core installation is currently Windows-only".into(),
        ));
    }
    if core.is_empty()
        || !core
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
    {
        return Err(UpdateError::Other("Invalid core identifier".into()));
    }
    let name = format!("{core}_libretro.dll");
    let target = runtime.join("cores").join(&name);
    if target.is_file() {
        return Ok(target);
    }
    let pack = ManagedEmulator::new(emulators, "libretro-cores");
    let version = "1.22.2";
    let mut source = find(&pack.version_dir(version), &name);
    if source.is_none() {
        let release = Release {
            version: version.into(),
            asset_name: "RetroArch_cores.7z".into(),
            url: "https://buildbot.libretro.com/stable/1.22.2/windows/x86_64/RetroArch_cores.7z"
                .into(),
            size: 229761684,
            sha256: Some("86B871E11B9B4772AC644B40A38F2C8E9449DA1F355EAE7DA08AA061148547B0".into()),
        };
        let archive = sources::download(&release, &pack.root.join("downloads"))?;
        pack.install_archive(version, &archive, &name)?;
        source = find(&pack.version_dir(version), &name);
    }
    let source = source
        .ok_or_else(|| UpdateError::Other(format!("Core {core} absent from the curated pack")))?;
    std::fs::create_dir_all(target.parent().unwrap())?;
    let partial = target.with_extension("dll.partial");
    std::fs::copy(source, &partial)?;
    std::fs::rename(partial, &target)?;
    Ok(target)
}

fn find(root: &Path, name: &str) -> Option<PathBuf> {
    let mut dirs = vec![root.to_path_buf()];
    while let Some(dir) = dirs.pop() {
        let Ok(entries) = std::fs::read_dir(dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let Ok(kind) = entry.file_type() else {
                continue;
            };
            if kind.is_dir() {
                dirs.push(entry.path());
            } else if kind.is_file() && entry.file_name() == name {
                return Some(entry.path());
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn installed_core_never_downloads_or_replaces_it() {
        if !cfg!(windows) {
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let target = dir.path().join("runtime/cores/snes9x_libretro.dll");
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        std::fs::write(&target, b"existing pinned core").unwrap();
        assert_eq!(
            ensure_core("snes9x", &dir.path().join("runtime"), dir.path()).unwrap(),
            target
        );
        assert_eq!(std::fs::read(target).unwrap(), b"existing pinned core");
        assert!(ensure_core("../escape", dir.path(), dir.path()).is_err());
    }
}
