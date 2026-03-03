#![allow(dead_code)]

mod alignment3d;
mod error;
mod horizontal;
mod parser;
mod spiral;
mod station;
mod terrain;
mod types;
mod vertical;

pub(crate) use alignment3d::evaluate_alignment_3d;
pub(crate) use error::LandXmlError;
pub(crate) use horizontal::evaluate_alignment_2d;
pub(crate) use parser::parse_landxml;
pub(crate) use types::*;

impl LandXmlDocument {
    pub fn parse(xml: &str, options: LandXmlParseOptions) -> Result<Self, LandXmlError> {
        parse_landxml(xml, options)
    }

    pub fn alignment_names(&self) -> Vec<String> {
        self.alignments.iter().map(|a| a.name.clone()).collect()
    }

    pub fn alignment_by_name(&self, name: &str) -> Option<&AlignmentRecord> {
        self.alignments.iter().find(|a| a.name == name)
    }
}
