use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

fn collect_files(root: impl AsRef<Path>, extensions: &[&str]) -> Vec<PathBuf> {
    fn visit(path: &Path, extensions: &[&str], files: &mut Vec<PathBuf>) {
        let Ok(entries) = fs::read_dir(path) else {
            return;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                visit(&path, extensions, files);
                continue;
            }

            if path
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| extensions.contains(&ext))
            {
                files.push(path);
            }
        }
    }

    let mut files = Vec::new();
    visit(root.as_ref(), extensions, &mut files);
    files
}

fn assert_files_do_not_contain(root: impl AsRef<Path>, extensions: &[&str], patterns: &[&str]) {
    let offenders: Vec<String> = collect_files(root, extensions)
        .into_iter()
        .filter_map(|path| {
            let text = fs::read_to_string(&path).ok()?;
            let matched: Vec<&str> = patterns
                .iter()
                .copied()
                .filter(|pattern| text.contains(pattern))
                .collect();
            (!matched.is_empty()).then(|| format!("{}: {matched:?}", path.display()))
        })
        .collect();

    assert!(
        offenders.is_empty(),
        "legacy desktop/RPC patterns remain:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn frontend_no_longer_calls_tauri_invoke_or_rpc_paths() {
    assert_files_do_not_contain(
        repo_root().join("src"),
        &["ts", "tsx"],
        &["invoke(", "tauri-mock", "Tauri", "/rpc"],
    );
}

#[test]
fn tests_do_not_encode_desktop_or_tauri_contracts() {
    assert_files_do_not_contain(
        repo_root().join("tests"),
        &["ts", "tsx"],
        &[
            "tauri-mock",
            "mail:notification-open",
            "desktop notifications",
        ],
    );
}

#[test]
fn dev_proxy_and_nginx_do_not_expose_rpc_route() {
    let root = repo_root();
    let vite = fs::read_to_string(root.join("vite.config.ts")).unwrap();
    let nginx = fs::read_to_string(root.join("deploy/nginx.conf")).unwrap();

    assert!(!vite.contains("\"/rpc\""));
    assert!(!nginx.contains("|rpc|"));
}

#[test]
fn github_workflows_do_not_build_desktop_packages() {
    assert_files_do_not_contain(
        repo_root().join(".github/workflows"),
        &["yml", "yaml"],
        &[
            "tauri",
            "build:windows",
            "build:macos",
            "bundle/dmg",
            "nsis",
        ],
    );
}

#[test]
fn obsolete_tauri_scripts_are_removed() {
    let root = repo_root();
    assert!(!root.join("scripts/build-tauri.mjs").exists());
    assert!(!root.join("scripts/gen_dispatch.py").exists());
    assert!(!root.join("site/gen_icons.py").exists());
}
