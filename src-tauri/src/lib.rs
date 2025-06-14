mod did;
mod keri;
mod paths;

use cesrox::primitives::codes::basic::Basic;
// use keri_core::{
//     actor,
//     database::{redb::RedbDatabase, sled::SledEventDatabase},
//     event_message::{
//         event_msg_builder::EventMsgBuilder, signed_event_message::Notice, EventTypeTag,
//     },
//     prefix::{BasicPrefix, IndexedSignature, SelfSigningPrefix},
//     processor::{basic_processor::BasicProcessor, event_storage::EventStorage},
//     signer::{CryptoBox, KeyManager},
// };
use std::sync::Arc;
use tauri::Manager;
use tauri_plugin_fs::FsExt;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn keri_inception(app: tauri::AppHandle) -> String {
    let (root_path, events_db_path) = paths::get_paths(app);

    let new_keri = keri::new_keri_data(root_path, events_db_path);
    match new_keri {
        Ok(keri) => {
            let inception_event = keri::keri_inception_event(keri);
            format!("Ok {}", inception_event)
        }
        Err(err) => format!("Error: {}", err),
    }
}

#[tauri::command]
async fn keri_rotate(app: tauri::AppHandle) -> String {
    format!("Unimplemented")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .setup(|app| {
            // allowed the given directory
            let scope = app.fs_scope();
            scope.allow_directory("./data", false);
            // dbg!(scope.allowed());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet])
        .invoke_handler(tauri::generate_handler![keri_inception])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
