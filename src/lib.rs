//! A parser for [Java Classfiles](https://docs.oracle.com/javase/specs/jvms/se10/html/jvms-4.html)

use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

#[macro_use]
extern crate nom;

#[macro_use]
extern crate bitflags;

pub mod attribute_info;
pub mod constant_info;
pub mod field_info;
pub mod method_info;

pub mod parser;
pub mod types;

pub mod constant_pool;
pub mod descriptor;

pub use parser::class_parser;
pub use parser::class_parser_opt;
use parser::ParseData;
pub use types::*;

mod util;

/// Attempt to parse a class file given a class file given a path to a class file
/// (without .class extension)
///
/// ```rust
/// match classfile_parser::parse_class("./java-assets/compiled-classes/BasicClass") {
///     Ok(class_file) => {
///         println!("version {:?}", class_file.version);
///     }
///     Err(ex) => panic!("Failed to parse: {}", ex),
/// };
/// ```
pub fn parse_class(class_name: &str) -> Result<ClassFile, String> {
    let class_file_name = &format!("{}.class", class_name);
    let path = Path::new(class_file_name);
    let display = path.display();

    let mut file = match File::open(&path) {
        Err(why) => {
            return Err(format!("Unable to open {}: {}", display, &why.to_string()));
        }
        Ok(file) => file,
    };

    let mut class_bytes = Vec::new();
    if let Err(why) = file.read_to_end(&mut class_bytes) {
        return Err(format!("Unable to read {}: {}", display, &why.to_string()));
    }

    let parsed_class = class_parser(ParseData::new(&class_bytes));
    match parsed_class {
        Ok((_, c)) => Ok(c),
        _ => Err("Failed to parse class?".to_string()),
    }
}
