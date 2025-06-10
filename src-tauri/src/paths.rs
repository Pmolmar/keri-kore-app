use std::path::PathBuf;
use tauri::Manager;

pub fn get_paths(app: tauri::AppHandle) -> (PathBuf, PathBuf) {
    let app_dir = app
        .path()
        .app_data_dir()
        .expect("Failed to get app directory");
    println!("App data directory: {:?}", app_dir);
    // Create paths that work on both mobile and desktop
    let root_path = app_dir.join("test-db");
    let events_db_path = app_dir.join("events.db");

    // Create directory if needed
    std::fs::create_dir_all(&app_dir).expect("Failed to create directory");

    (root_path, events_db_path)
}
