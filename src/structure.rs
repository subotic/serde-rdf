#![allow(unused_variables, unused_imports, dead_code)]

use std::collections::HashMap;

use rio_api::model::NamedNode;

pub enum Term {
    Literal(String),
    Subject(String),
}

/// A subject holds additional information for the serializer
/// to further configure how a specific rust struct should be serialized.
#[derive(Debug)]
pub struct SubjectConfig {
    pub struct_name: String,
    pub rdf_type: String,
    pub identifier_field: String,
    pub identifier_prefix: String,
    pub properties: Vec<PropertyConfig>,
}

#[derive(Debug)]
pub struct PropertyConfig {
    pub struct_field: String,
    pub rdf_property: String,
}

/// Serializer configuration containing mappings / instructions on how to
/// serialize rust structs into RDF. The config contains one ore more
/// `Subject`s.
#[derive(Debug)]
pub struct SerializerConfig {
    pub base_iri: String,
    pub namespaces: HashMap<String, String>,
    pub subjects: HashMap<String, SubjectConfig>,
}

#[derive(Debug)]
pub struct SubjectBuilder {
    struct_name: String,
    rdf_type: String,
    identifier_field: String,
    identifier_prefix: String,
    properties: Vec<PropertyConfig>,
}
