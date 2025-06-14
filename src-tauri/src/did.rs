// use ssi_dids::DIDKey;
use crate::keri::KeriData;
use keri_core::signer::KeyManager;
use ssi_dids::{
    document::{
        self,
        representation::{self, json_ld},
    },
    resolution::{self, Error, Output},
    DIDBuf, DIDMethod, DIDMethodResolver, DIDURLBuf, Document,
};
use static_iref::iri_ref;

pub struct DIDKeri {
    pub keri_data: KeriData,
}

impl DIDKeri {
    pub fn generate() {}
}

impl DIDMethod for DIDKeri {
    const DID_METHOD_NAME: &'static str = "keri";
}

impl DIDMethodResolver for DIDKeri {
    fn method_name(&self) -> &str {
        Self::DID_METHOD_NAME
    }
    async fn resolve_method_representation<'a>(
        &'a self,
        id: &'a str,
        options: resolution::Options,
    ) -> Result<Output<Vec<u8>>, Error> {
        let did = DIDBuf::from_string(format!("did:key:{id}")).unwrap();

        //TODO: Anadir clave publica (recuperar de Keri)
        let public_key = self.keri_data.key_manager.public_key();

        let vm_didurl = DIDURLBuf::from_string(format!("{did}#{id}")).unwrap();

        let mut doc = Document::new(did.to_owned());

        //TODO: Anadir metodos de verificacion
        // doc.verification_method.push();
        // doc.verification_relationships
        //     .authentication
        //     .push(ValueOrReference::Reference(vm_didurl.clone().into()));
        // doc.verification_relationships
        //     .assertion_method
        //     .push(ValueOrReference::Reference(vm_didurl.into()));

        let mut json_ld_context = Vec::new();
        //Este es el tipo de clave pública que usa Keri
        json_ld_context.push(json_ld::ContextEntry::IriRef(
            iri_ref!("https://w3id.org/security/suites/ed25519-2020/v1").to_owned(),
        ));

        let content_type = options.accept.unwrap_or(representation::MediaType::JsonLd);
        let represented = doc.into_representation(representation::Options::from_media_type(
            content_type,
            move || json_ld::Options {
                context: json_ld::Context::array(json_ld::DIDContext::V1, json_ld_context),
            },
        ));

        Ok(Output::new(
            represented.to_bytes(),
            document::Metadata::default(),
            resolution::Metadata::from_content_type(Some(content_type.to_string())),
        ))
    }
}
