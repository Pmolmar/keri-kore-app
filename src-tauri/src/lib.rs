use cesrox::primitives::codes::basic::Basic;
use keri_core::{
    actor,
    database::{redb::RedbDatabase, sled::SledEventDatabase},
    event_message::{
        event_msg_builder::EventMsgBuilder, signed_event_message::Notice, EventTypeTag,
    },
    prefix::{BasicPrefix, IndexedSignature, SelfSigningPrefix},
    processor::{basic_processor::BasicProcessor, event_storage::EventStorage},
    signer::{CryptoBox, KeyManager},
};
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
    let key_manager = CryptoBox::new();

    match key_manager {
        Ok(key_manager) => {
            //Crea Claves
            let current_key = BasicPrefix::new(Basic::Ed25519, key_manager.public_key());
            let next_key = BasicPrefix::new(Basic::Ed25519, key_manager.next_public_key());

            // Get platform-specific app directory
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

            // Initialize databases
            let db = Arc::new(SledEventDatabase::new(&root_path).unwrap());
            let events_db = Arc::new(RedbDatabase::new(&events_db_path).unwrap());

            let (processor, storage) = (
                BasicProcessor::new(events_db.clone(), db.clone(), None),
                EventStorage::new(events_db.clone(), db.clone()),
            );

            let inception_event = EventMsgBuilder::new(EventTypeTag::Icp)
                .with_keys(vec![current_key])
                .with_next_keys(vec![next_key])
                .build();

            match inception_event {
                Ok(event) => {
                    // format!(
                    //     "Evento de incepcion satisfactorio con id {:?}",
                    //     event.data.prefix.clone()
                    // );
                    let identifier = event.data.prefix.clone();
                    let to_sign = event.encode();

                    match to_sign {
                        Ok(sign) => {
                            let signature = key_manager.sign(&sign);
                            match signature {
                                Ok(signature) => {
                                    let indexed_signature = IndexedSignature::new_both_same(
                                        SelfSigningPrefix::Ed25519Sha512(signature),
                                        0,
                                    );

                                    let signed_inception =
                                        event.sign(vec![indexed_signature], None, None);

                                    match actor::process_notice(
                                        Notice::Event(signed_inception),
                                        &processor,
                                    ) {
                                        Ok(val) => {
                                            let state = storage.get_state(&identifier);
                                            format!(
                                                "Evento de incepcion creado con estado final {:?}",
                                                state
                                            )
                                        }
                                        Err(error) => {
                                            format!("Error procesando evento {:?}", error)
                                        }
                                    }
                                }
                                Err(error) => {
                                    format!("Error con firma {:?}", error)
                                }
                            }
                        }
                        Err(error) => {
                            format!("Error preparando firma {:?}", error)
                        }
                    }
                }
                Err(error) => {
                    format!("Error con el evento de creacion {:?}", error)
                }
            }

            // format!("Todo Ok con clave {:?}", current_key)
        }
        Err(error) => {
            format!("Error con el KeyManager {}", error)
        }
    }
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
