pub mod parser;
pub mod ssml_constants;
pub mod xml_writer;

use color_eyre::Result;

/// Parses a String into the Unique Text to SSML Format. Useful for taking a string
/// and making some sweet, sweet SSML.
pub fn parse_string(to_parse: String) -> Result<String> {
    parser::parse_as_ssml(&to_parse)
}

/// Parses a String into the Unique Text to SSML Format. Useful for taking a string
/// and making some sweet, sweet SSML.
pub fn parse_str(to_parse: &str) -> Result<String> {
    parser::parse_as_ssml(to_parse)
}
