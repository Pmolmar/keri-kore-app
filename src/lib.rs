mod db;

use cesrox::primitives::codes::basic::Basic;
use keri_core::{
    actor,
    // database::{redb::RedbDatabase, sled::SledEventDatabase},
    event_message::{
        event_msg_builder::EventMsgBuilder, signed_event_message::Notice, EventTypeTag,
    },
    prefix::{BasicPrefix, IndexedSignature, SelfSigningPrefix},
    processor::{basic_processor::BasicProcessor, event_storage::EventStorage},
    signer::{CryptoBox, KeyManager},
};
use std::sync::Arc;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
async fn keri_inception() -> String {
    let key_manager = CryptoBox::new();

    match key_manager {
        Ok(key_manager) => {
            //Crea Claves
            let current_key = BasicPrefix::new(Basic::Ed25519, key_manager.public_key());
            let next_key = BasicPrefix::new(Basic::Ed25519, key_manager.next_public_key());

            //Crea DB de eventos
            //Cambiar a usar BBDD sqlite
            // let root = Builder::new().prefix("test-db").tempdir().unwrap();
            // let db = Arc::new(SledEventDatabase::new(root.path()).unwrap());
            // let events_db_path = NamedTempFile::new().unwrap();
            // let events_db = Arc::new(RedbDatabase::new(events_db_path.path()).unwrap());

            // Initialize SQLx database connection
            let events_db = Arc::new(db::SqlxEventDatabase::new("sqlite::db"));
            let db = Arc::new(events_db.clone());

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
        .invoke_handler(tauri::generate_handler![greet])
        .invoke_handler(tauri::generate_handler![keri_inception])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
