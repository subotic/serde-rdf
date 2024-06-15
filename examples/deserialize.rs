#![allow(unused_variables, unused_imports, dead_code)]

use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Project {
    id: Iri,
    name: String,
    description: LangString,
    shortcode: String,
    datasets: Vec<Dataset>,
}

#[derive(Debug, Deserialize)]
struct Dataset {
    id: Iri,
    name: String,
}

type Iri = String;

#[derive(Debug, Deserialize)]
pub struct LangString(pub HashMap<IsoCode, String>);

#[derive(Debug, Default, Deserialize, Hash, Eq, PartialEq)]
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
    let project_ttl = r#"
       <https://ark.dasch.swiss/ark:/72163/1/081C> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <https://ns.dasch.swiss/repository#Project> ;
            <https://ns.dasch.swiss/repository#hasName> "Hôtel de Musique Bern"^^xsd:string ;
            <https://ns.dasch.swiss/repository#hasDescription> "The database documents the events that took place in the Hôtel de Musique in Bern between 1766 and 1905. The repertoire was constituted by different kinds of spectacles like theatre plays, operas, ballets, concerts, dance parties, acrobatic performances, conferences or magicians. The list reconstructs the lifely and colourful theatre culture of Bern in the 19th Century."@en ;

            <https://ns.dasch.swiss/repository#hasShortcode> "081C"^^xsd:string ;
            <https://ns.dasch.swiss/repository#hasDataset> <dataset-0> ;
    "#;

    let _project: Project = serde_rdf::from_str(project_ttl).unwrap();
}
