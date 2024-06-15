#![allow(unused_variables, unused_imports, dead_code)]

//! Serialize a Rust data structure into RDF data.

use std::io;

use rio_api::formatter::TriplesFormatter;
use rio_api::model::{NamedNode as RioNamedNode, Triple};
use rio_turtle::TurtleFormatter;
use serde::ser::{self, Serialize};

use crate::error::{Error, Result};
use crate::structure::SerializerConfig;

/// Serializer mapping configuration containing mappings aka instructions on how
/// to serialize a type. There are three possible options:
/// (1) one IRI: this denotes the name of the field containing the identifier. Further, it provides
/// the prefix to the identifier used to build the IRI.
/// (2) one Clazz: this denotes the name of the struct and to what RDF class it should be typed
/// (3) one or more Property: this denotes the name of the field, it's property IRI, and the
/// literal xsd:type (`XsdType`) it should be serialized into, if it is a literal, or (`Subject`)
/// denoting that it is a struct that needs to be serialized as a separate subject.
///
/// Example:
/// ```
/// use std::collections::HashMap;
/// use serde_rdf::{SerializerConfig, SubjectConfig, PropertyConfig};
/// let _config = SerializerConfig{
///     base_iri: "".to_string(),
///     namespaces: Default::default(),
///     subjects: HashMap::from([
///         ("Project".to_string(), SubjectConfig{
///             struct_name: "Project".to_string(),
///             rdf_type: "https://ns.dasch.swiss/repository#Project".to_string(),
///             identifier_field: "id".to_string(),
///             identifier_prefix: "https://ark.dasch.swiss/ark:/72163/1/".to_string(),
///             properties: vec!(
///                 PropertyConfig{struct_field: "name".to_string(), rdf_property: "https://ns.dasch.swiss/repository#hasName".to_string()},
///                 PropertyConfig{struct_field: "description".to_string(), rdf_property: "https://ns.dasch.swiss/repository#hasDescription".to_string()},
///                 PropertyConfig{struct_field: "shortcode".to_string(), rdf_property: "https://ns.dasch.swiss/repository#hasShortcode".to_string()},
///                 PropertyConfig{struct_field: "datasets".to_string(), rdf_property: "https://ns.dasch.swiss/repository#hasDataset".to_string()},
///             ),
///         }),
///         ("Dataset".to_string(), SubjectConfig{
///             struct_name: "Dataset".to_string(),
///             rdf_type: "https://ns.dasch.swiss/repository#Dataset".to_string(),
///             identifier_field: "id".to_string(),
///             identifier_prefix: "https://ark.dasch.swiss/ark:/72163/1/".to_string(),
///             properties: vec!(
///                 PropertyConfig{struct_field: "title".to_string(), rdf_property: "https://ns.dasch.swiss/repository#hasTitle".to_string()}
///             ),
///         })])
/// };
/// ```

#[derive(Eq, PartialEq, Debug, Clone, Hash)]
pub enum Literal<'a> {
    /// A [simple literal](https://www.w3.org/TR/rdf11-concepts/#dfn-simple-literal) without datatype or language form.
    Simple {
        /// The [lexical form](https://www.w3.org/TR/rdf11-concepts/#dfn-lexical-form).
        value: String,
    },
    /// A [language-tagged string](https://www.w3.org/TR/rdf11-concepts/#dfn-language-tagged-string)
    LanguageTaggedString {
        /// The [lexical form](https://www.w3.org/TR/rdf11-concepts/#dfn-lexical-form).
        value: String,
        /// The [language tag](https://www.w3.org/TR/rdf11-concepts/#dfn-language-tag).
        language: String,
    },
    /// A literal with an explicit datatype
    Typed {
        /// The [lexical form](https://www.w3.org/TR/rdf11-concepts/#dfn-lexical-form).
        value: String,
        /// The [datatype IRI](https://www.w3.org/TR/rdf11-concepts/#dfn-datatype-iri).
        datatype: RioNamedNode<'a>,
    },
}

impl Literal<'_> {
    /// Return the lexical form of the literal.
    pub fn value(&self) -> &str {
        match self {
            Literal::Simple { value } => value,
            Literal::LanguageTaggedString { value, .. } => value,
            Literal::Typed { value, .. } => value,
        }
    }
}

#[derive(Debug)]
struct Loc {
    id: String,
    type_name: String,
}
/// Need a structure inside the serializer to hold the components of triples as they are
/// gathered:
/// - one IRI field holding the IRI of the subject
/// - one field with Vec holding tuples with the predicate and literal.
///
/// The struct that we want to serialize, needs to be prepared:
/// - those fields of a struct that contain a Vec of literals need to be flattened `serde(flatten)`
/// - those fields of a struct that contain a Vec of structs should **not** be flattened
///  
pub struct Serializer<'a, W: io::Write> {
    stack: Vec<Loc>,
    last_subject: &'a str,
    last_key: &'a str,
    last_literal: Option<Literal<'a>>,
    output: String,
    mapping: SerializerConfig,
    formatter: TurtleFormatter<W>,
}

impl<'a, W> Serializer<'a, W>
where
    W: io::Write,
{
    fn new(mapping: SerializerConfig, writer: W) -> Serializer<'a, W> {
        Serializer::with_formatter(mapping, TurtleFormatter::new(writer))
    }

    fn with_formatter(
        mapping: SerializerConfig,
        formatter: TurtleFormatter<W>,
    ) -> Serializer<'a, W> {
        Serializer {
            stack: Vec::new(),
            last_subject: "",
            last_key: "",
            last_literal: None,
            output: String::new(),
            mapping,
            formatter,
        }
    }
}

/// Serialize the given value as an RDF string.
///
/// # Errors
///
/// Serialization fails if the type cannot be represented as RDF.
pub fn to_string<T>(value: &T, config: SerializerConfig) -> Result<String>
where
    T: ?Sized + Serialize,
{
    let mut serializer = Serializer::new(config, Vec::default());
    value.serialize(&mut serializer)?;
    let bytes = serializer.formatter.finish()?;

    // SAFETY: The `Formatter` never emits invalid UTF-8.
    Ok(unsafe { String::from_utf8_unchecked(bytes) })
}

impl<'a, W> ser::Serializer for &mut Serializer<'a, W>
where
    W: io::Write,
{
    // The output type produced by this `Serializer` during successful
    // serialization. Most serializers that produce text or binary output should
    // set `Ok = ()` and serialize into an `io::Write` or buffer contained
    // within the `Serializer` instance, as happens here. Serializers that build
    // in-memory data structures may be simplified by using `Ok` to propagate
    // the data structure around.
    type Ok = ();

    // The error type when some error occurs during serialization.
    type Error = Error;

    // Associated types for keeping track of additional state while serializing
    // compound data structures like sequences and maps. In this case no
    // additional state is required beyond what is already stored in the
    // Serializer struct.
    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    // Here we go with the simple methods. The following 12 methods receive one
    // of the primitive types of the data model and map it to JSON by appending
    // into the output string.
    fn serialize_bool(self, v: bool) -> Result<()> {
        use crate::ser::Literal::Typed;

        if v {
            self.last_literal = Some(Typed {
                value: "true".to_owned(),
                datatype: RioNamedNode { iri: "xsd:boolean" },
            });
        } else {
            self.last_literal = Some(Typed {
                value: "false".to_owned(),
                datatype: RioNamedNode { iri: "xsd:boolean" },
            });
        }
        Ok(())
    }

    // JSON does not distinguish between different sizes of integers, so all
    // signed integers will be serialized the same and all unsigned integers
    // will be serialized the same. Other formats, especially compact binary
    // formats, may need independent logic for the different sizes.
    fn serialize_i8(self, v: i8) -> Result<Self::Ok> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok> {
        self.serialize_i64(i64::from(v))
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok> {
        self.serialize_i64(i64::from(v))
    }

    // Not particularly efficient but this is example code anyway. A more
    // performant approach would be to use the `itoa` crate.
    fn serialize_i64(self, v: i64) -> Result<Self::Ok> {
        self.output += &v.to_string();
        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok> {
        self.serialize_u64(u64::from(v))
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok> {
        self.output += &v.to_string();
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok> {
        self.serialize_f64(f64::from(v))
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok> {
        self.output += &v.to_string();
        Ok(())
    }

    // Serialize a char as a single-character string. Other formats may
    // represent this differently.
    fn serialize_char(self, v: char) -> Result<Self::Ok> {
        self.serialize_str(&v.to_string())
    }

    // This only works for strings that don't require escape sequences but you
    // get the idea. For example it would emit invalid JSON if the input string
    // contains a '"' character.
    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        println!("serialize_str");

        use crate::ser::Literal::Typed;

        self.last_literal = Some(Typed {
            value: v.to_owned(),
            datatype: RioNamedNode { iri: "xsd:string" },
        });

        Ok(())
    }

    // Serialize a byte array as an array of bytes. Could also use a base64
    // string here. Binary formats will typically represent byte arrays more
    // compactly.
    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        use serde::ser::SerializeSeq;
        let mut seq = self.serialize_seq(Some(v.len()))?;
        for byte in v {
            seq.serialize_element(byte)?;
        }
        seq.end()
    }

    // An absent optional is represented as the JSON `null`.
    fn serialize_none(self) -> Result<Self::Ok> {
        self.serialize_unit()
    }

    // A present optional is represented as just the contained value. Note that
    // this is a lossy representation. For example the values `Some(())` and
    // `None` both serialize as just `null`. Unfortunately this is typically
    // what people expect when working with JSON. Other formats are encouraged
    // to behave more intelligently if possible.
    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    // In Serde, unit means an anonymous value containing no data. Map this to
    // JSON as `null`.
    fn serialize_unit(self) -> Result<Self::Ok> {
        self.output += "null";
        Ok(())
    }

    // Unit struct means a named value containing no data. Again, since there is
    // no data, map this to JSON as `null`. There is no need to serialize the
    // name in most formats.
    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok> {
        self.serialize_unit()
    }

    // When serializing a unit variant (or any other kind of variant), formats
    // can choose whether to keep track of it by index or by name. Binary
    // formats typically use the index of the variant and human-readable formats
    // typically use the name.
    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(variant)
    }

    // As is done here, serializers are encouraged to treat newtype structs as
    // insignificant wrappers around the data they contain.
    fn serialize_newtype_struct<T>(self, name: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    // Note that newtype variant (and all of the other variant serialization
    // methods) refer exclusively to the "externally tagged" enum
    // representation.
    //
    // Serialize this to JSON in externally tagged form as `{ NAME: VALUE }`.
    fn serialize_newtype_variant<T>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        self.output += "{";
        variant.serialize(&mut *self)?;
        self.output += ":";
        value.serialize(&mut *self)?;
        self.output += "}";
        Ok(())
    }

    // Now we get to the serialization of compound types.
    //
    // The start of the sequence, each value, and the end are three separate
    // method calls. This one is responsible only for serializing the start,
    // which in JSON is `[`.
    //
    // The length of the sequence may or may not be known ahead of time. This
    // doesn't make a difference in JSON because the length is not represented
    // explicitly in the serialized form. Some serializers may only be able to
    // support sequences for which the length is known up front.
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        self.output += "[";
        Ok(self)
    }

    // Tuples look just like sequences in JSON. Some formats may be able to
    // represent tuples more efficiently by omitting the length, since tuple
    // means that the corresponding `Deserialize implementation will know the
    // length without needing to look at the serialized data.
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(len))
    }

    // Tuple structs look just like sequences in JSON.
    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(Some(len))
    }

    // Tuple variants are represented in JSON as `{ NAME: [DATA...] }`. Again
    // this method is only responsible for the externally tagged representation.
    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        self.output += "{";
        variant.serialize(&mut *self)?;
        self.output += ":[";
        Ok(self)
    }

    // Maps are represented in JSON as `{ K: V, K: V, ... }`.
    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        self.output += "{";
        Ok(self)
    }

    // Structs represent subjects, where the name is the "type".
    fn serialize_struct(self, name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        println!("serialize_struct");
        println!("name: {}", name);
        self.last_subject = name;
        Ok(self)
    }

    // Struct variants are represented in JSON as `{ NAME: { K: V, ... } }`.
    // This is the externally tagged representation.
    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        self.output += "{";
        variant.serialize(&mut *self)?;
        self.output += ":{";
        Ok(self)
    }
}

// The following 7 impls deal with the serialization of compound types like
// sequences and maps. Serialization of such types is begun by a Serializer
// method and followed by zero or more calls to serialize individual elements of
// the compound type and one call to end the compound type.
//
// This impl is SerializeSeq so these methods are called after `serialize_seq`
// is called on the Serializer.
impl<'a, W: io::Write> ser::SerializeSeq for &mut Serializer<'a, W> {
    // Must match the `Ok` type of the serializer.
    type Ok = ();
    // Must match the `Error` type of the serializer.
    type Error = Error;

    // Serialize a single element of the sequence.
    fn serialize_element<T>(&mut self, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        if !self.output.ends_with('[') {
            self.output += ",";
        }
        value.serialize(&mut **self)
    }

    // Close the sequence.
    fn end(self) -> Result<Self::Ok> {
        self.output += "]";
        Ok(())
    }
}

// Same thing but for tuples.
impl<'a, W: io::Write> ser::SerializeTuple for &mut Serializer<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        if !self.output.ends_with('[') {
            self.output += ",";
        }
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        self.output += "]";
        Ok(())
    }
}

// Same thing but for tuple structs.
impl<'a, W: io::Write> ser::SerializeTupleStruct for &mut Serializer<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        if !self.output.ends_with('[') {
            self.output += ",";
        }
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        self.output += "]";
        Ok(())
    }
}

// Tuple variants are a little different. Refer back to the
// `serialize_tuple_variant` method above:
//
//    self.output += "{";
//    variant.serialize(&mut *self)?;
//    self.output += ":[";
//
// So the `end` method in this impl is responsible for closing both the `]` and
// the `}`.
impl<'a, W: io::Write> ser::SerializeTupleVariant for &mut Serializer<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        if !self.output.ends_with('[') {
            self.output += ",";
        }
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        self.output += "]}";
        Ok(())
    }
}

// Some `Serialize` types are not able to hold a key and value in memory at the
// same time so `SerializeMap` implementations are required to support
// `serialize_key` and `serialize_value` individually.
//
// There is a third optional method on the `SerializeMap` trait. The
// `serialize_entry` method allows serializers to optimize for the case where
// key and value are both available simultaneously. In JSON it doesn't make a
// difference so the default behavior for `serialize_entry` is fine.
impl<'a, W: io::Write> ser::SerializeMap for &mut Serializer<'a, W> {
    type Ok = ();
    type Error = Error;

    // The Serde data model allows map keys to be any serializable type. JSON
    // only allows string keys so the implementation below will produce invalid
    // JSON if the key serializes as something other than a string.
    //
    // A real JSON serializer would need to validate that map keys are strings.
    // This can be done by using a different Serializer to serialize the key
    // (instead of `&mut **self`) and having that other serializer only
    // implement `serialize_str` and return an error on any other data type.
    fn serialize_key<T>(&mut self, key: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        if !self.output.ends_with('{') {
            self.output += ",";
        }
        key.serialize(&mut **self)
    }

    // It doesn't make a difference whether the colon is printed at the end of
    // `serialize_key` or at the beginning of `serialize_value`. In this case
    // the code is a bit simpler having it here.
    fn serialize_value<T>(&mut self, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        self.output += ":";
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        self.output += "}";
        Ok(())
    }
}

// Structs are like maps in which the keys are constrained to be compile-time
// constant strings.
impl<'a, W: io::Write> ser::SerializeStruct for &mut Serializer<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        println!("serialize_struct -> serialize_field");

        value.serialize(&mut **self)?;

        let subject = self.mapping.subjects.get(self.last_subject).unwrap();

        if subject.identifier_field == key {
            println!(
                "serialize_struct -> serialize_field -> identifier_field: {}",
                key
            );
        }

        let value = match self.last_literal.take() {
            Some(v) => v,
            None => {
                return Err(Error::Message(format!(
                    "serialize_struct -> serialize_field -> no value found for key: {}",
                    key
                )))
            }
        };

        self.formatter.format(&Triple {
            subject: RioNamedNode {
                iri: format!("{}{}", &subject.identifier_prefix, value.value()).as_str(),
            }
            .into(),
            predicate: RioNamedNode {
                iri: "http://www.w3.org/1999/02/22-rdf-syntax-ns#type",
            },
            object: RioNamedNode {
                iri: subject.rdf_type.as_str(),
            }
            .into(),
        })?;
        Ok(())
    }

    fn end(self) -> Result<()> {
        println!("serialize_struct -> end");
        Ok(())
    }
}

// Similar to `SerializeTupleVariant`, here the `end` method is responsible for
// closing both of the curly braces opened by `serialize_struct_variant`.
impl<'a, W: io::Write> ser::SerializeStructVariant for &mut Serializer<'a, W> {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Serialize,
    {
        if !self.output.ends_with('{') {
            self.output += ",";
        }
        key.serialize(&mut **self)?;
        self.output += ":";
        value.serialize(&mut **self)
    }

    fn end(self) -> Result<()> {
        self.output += "}}";
        Ok(())
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use serde::Serialize;

    use crate::{to_string, SerializerConfig, SubjectConfig};

    #[test]
    fn test_simple_struct() {
        #[derive(Serialize)]
        struct Test {
            id: String,
        }

        let config = SerializerConfig {
            base_iri: "".to_string(),
            namespaces: Default::default(),
            subjects: HashMap::from([(
                "Test".to_string(),
                SubjectConfig {
                    struct_name: "Test".to_string(),
                    rdf_type: "https://example.org/ns#Test".to_string(),
                    identifier_field: "id".to_string(),
                    identifier_prefix: "https://ark.dasch.swiss/ark:/72163/1/".to_string(),
                    properties: Vec::new(),
                },
            )]),
        };

        let test = Test {
            id: "my-id".to_string(),
        };
        let expected = "<https://ark.dasch.swiss/ark:/72163/1/my-id> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <https://example.org/ns#Test> .\n";
        assert_eq!(to_string(&test, config).unwrap(), expected);
    }
}
