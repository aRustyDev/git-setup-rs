//! Output formatting for git-setup-rs.
//!
//! This module provides different output formatters for profile data.

pub mod csv;
pub mod json;
pub mod table;
pub mod yaml;

pub use csv::CsvFormatter;
pub use json::{JsonFormatter, OutputFormatter};
pub use table::TableFormatter;
pub use yaml::YamlFormatter;
