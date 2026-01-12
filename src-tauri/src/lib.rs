// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Serialize)]
struct BrowserProfile {
    browser_name: String,
    profile_name: String,
    root_directory: String,
}

#[tauri::command]
fn get_all_profiles() -> Vec<BrowserProfile> {
    let targets = vec![
        ("Firefox", "Mozilla/Firefox", "Firefox", ".mozilla/firefox"),
        ("Floorp", "Floorp", "Floorp", ".floorp"),
        ("Zen", "Zen", "Zen Browser", ".zen"),
        ("LibreWolf", "LibreWolf", "LibreWolf", ".librewolf"),
    ];

    let mut all_profiles = Vec::new();

    for (label, win, mac, lin) in targets {
        if let Some(base_dir) = get_base_dir(win, mac, lin) {
            let ini_path = base_dir.join("profiles.ini");
            if ini_path.exists() {
                if let Ok(content) = fs::read_to_string(&ini_path) {
                    parse_ini_content(&content, &base_dir, label, &mut all_profiles);
                }
            }
        }
    }

    all_profiles
}

fn get_base_dir(win: &str, mac: &str, lin: &str) -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        std::env::var("APPDATA")
            .ok()
            .map(|p| PathBuf::from(p).join(win))
    }
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var("HOME").ok()?;
        Some(
            PathBuf::from(home)
                .join("Library/Application Support")
                .join(mac),
        )
    }
    #[cfg(target_os = "linux")]
    {
        std::env::var("HOME")
            .ok()
            .map(|p| PathBuf::from(p).join(lin))
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        None
    }
}

fn parse_ini_content(
    content: &str,
    base_dir: &Path,
    label: &str,
    results: &mut Vec<BrowserProfile>,
) {
    let mut current_name = String::new();
    let mut current_path = String::new();
    let mut is_relative = false;

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("[Profile") {
            push_to_results(
                results,
                &current_name,
                &current_path,
                is_relative,
                base_dir,
                label,
            );
            current_name = String::new();
            current_path = String::new();
            is_relative = false;
        } else if line.starts_with("Name=") {
            current_name = line.replace("Name=", "");
        } else if line.starts_with("Path=") {
            current_path = line.replace("Path=", "");
        } else if line.starts_with("IsRelative=") {
            is_relative = line.contains('1');
        }
    }
    push_to_results(
        results,
        &current_name,
        &current_path,
        is_relative,
        base_dir,
        label,
    );
}

fn push_to_results(
    list: &mut Vec<BrowserProfile>,
    name: &str,
    path: &str,
    rel: bool,
    base: &Path,
    label: &str,
) {
    if !name.is_empty() && !path.is_empty() {
        let root_dir = if rel {
            base.join(path).to_string_lossy().to_string()
        } else {
            path.to_string()
        };

        list.push(BrowserProfile {
            browser_name: label.to_string(),
            profile_name: name.to_string(),
            root_directory: root_dir,
        });
    }
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, get_all_profiles])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
