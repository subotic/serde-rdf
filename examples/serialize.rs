#![allow(dead_code)]

use std::collections::HashMap;

use serde::*;
use serde_rdf::{SerializerConfig, SubjectConfig};

#[derive(Debug, Serialize)]
struct Project {
    id: Iri,
    name: String,
    description: LangString,
    shortcode: String,
    datasets: Vec<Dataset>,
}

#[derive(Debug, Serialize)]
struct Dataset {
    id: Iri,
}

type Iri = String;

#[derive(Debug, Serialize)]
pub struct LangString(pub HashMap<IsoCode, String>);

#[derive(Debug, Default, Serialize, PartialEq, Eq, Hash)]
pub enum IsoCode {
    #[default]
    DE, // German
    EN, // English
    FR, // French
    IT, // Italian
    ES, // Spanish
    PT, // Portuguese
    NL, // Dutch
    PL, // Polish
    RU, // Russian
    JA, // Japanese
    ZH, // Chinese
    AR, // Arabic
    FA, // Persian
}

fn main() {
    let mut name = HashMap::<IsoCode, String>::new();
    name.insert(IsoCode::EN, "Hôtel de Musique Bern".to_string());

    let dataset = Dataset {
        id: "dataset-0".to_string(),
    };

    let config = SerializerConfig {
        base_iri: "".to_string(),
        namespaces: Default::default(),
        subjects: HashMap::from([(
            "Dataset".to_string(),
            SubjectConfig {
                struct_name: "Dataset".to_string(),
                rdf_type: "https://example.org/ns#Test".to_string(),
                identifier_field: "id".to_string(),
                identifier_prefix: "https://ark.dasch.swiss/ark:/72163/1/".to_string(),
                properties: Vec::new(),
            },
        )]),
    };

    let project_ttl = serde_rdf::to_string(&dataset, config).unwrap();

    dbg!(project_ttl);
}
