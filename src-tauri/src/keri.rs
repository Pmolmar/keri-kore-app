use cesrox::primitives::codes::basic::Basic;
use keri_core::{
    actor,
    database::{redb::RedbDatabase, sled::SledEventDatabase},
    event::KeyEvent,
    event_message::{
        event_msg_builder::EventMsgBuilder, msg::TypedEvent, signed_event_message::Notice,
        EventTypeTag,
    },
    prefix::{BasicPrefix, IndexedSignature, SelfSigningPrefix},
    processor::{basic_processor::BasicProcessor, event_storage::EventStorage},
    signer::{CryptoBox, KeyManager},
};
use std::path::PathBuf;
use std::sync::Arc;
use tauri::Manager;
use tauri_plugin_fs::FsExt;

pub struct KeriData {
    key_manager: CryptoBox,
    storage: EventStorage<RedbDatabase>,
    processor: BasicProcessor<RedbDatabase>,
}

pub fn save_keri_state(keri_save: KeriData) {
    //Se tiene que guardar el key_Manager ya que contiene las claves privadas
    // El resto de datos se pueden recuperar del fichero que crean
    keri_save.key_manager.public_key();
}

pub fn load_keri_state(root_path: PathBuf, events_db_path: PathBuf) {}

pub fn new_keri_data(root_path: PathBuf, events_db_path: PathBuf) -> Result<KeriData, String> {
    let key_manager = CryptoBox::new();

    match key_manager {
        Ok(key_manager) => {
            // Initialize databases
            let db = Arc::new(SledEventDatabase::new(&root_path).unwrap());
            let events_db = Arc::new(RedbDatabase::new(&events_db_path).unwrap());

            let (processor, storage) = (
                BasicProcessor::new(events_db.clone(), db.clone(), None),
                EventStorage::new(events_db.clone(), db.clone()),
            );

            Ok(KeriData {
                key_manager,
                storage,
                processor,
            })
        }
        Err(error) => Err(format!("Error con el KeyManager {}", error)),
    }
}

fn keri_sign_event(
    event: TypedEvent<EventTypeTag, KeyEvent>,
    keri_instance: KeriData,
) -> std::string::String {
    let identifier = event.data.prefix.clone();
    let to_sign = event.encode();

    match to_sign {
        Ok(sign) => {
            let signature = keri_instance.key_manager.sign(&sign);
            match signature {
                Ok(signature) => {
                    let indexed_signature = IndexedSignature::new_both_same(
                        SelfSigningPrefix::Ed25519Sha512(signature),
                        0,
                    );

                    let signed_inception = event.sign(vec![indexed_signature], None, None);

                    match actor::process_notice(
                        Notice::Event(signed_inception),
                        &keri_instance.processor,
                    ) {
                        Ok(val) => {
                            let state = keri_instance.storage.get_state(&identifier);
                            format!("Evento de incepcion creado con estado final {:?}", state)
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

pub fn keri_inception_event(keri_instance: KeriData) -> String {
    let current_key = BasicPrefix::new(Basic::Ed25519, keri_instance.key_manager.public_key());
    let next_key = BasicPrefix::new(Basic::Ed25519, keri_instance.key_manager.next_public_key());

    let inception_event = EventMsgBuilder::new(EventTypeTag::Icp)
        .with_keys(vec![current_key])
        .with_next_keys(vec![next_key])
        .build();

    match inception_event {
        Ok(event) => keri_sign_event(event, keri_instance),
        Err(err) => {
            format!("Error preparando evento {:?}", err)
        }
    }
}

pub fn keri_rotate_keys(mut keri_instance: KeriData) -> String {
    match keri_instance.key_manager.rotate() {
        Ok(()) => format!("Rotacion correcta"),
        Err(err) => format!("Problemas en la rotacion: {}", err),
    }
}
