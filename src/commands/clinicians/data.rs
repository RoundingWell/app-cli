//! JSON:API attribute types for clinician resources.

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub(super) struct ClinicianAttributes {
    pub(super) name: String,
    pub(super) email: String,
    pub(super) enabled: bool,
    #[serde(default)]
    pub(super) npi: Option<String>,
    #[serde(default)]
    pub(super) credentials: Vec<String>,
}

/// Convenience alias: a JSON:API resource carrying clinician attributes.
pub(super) type Clinician = crate::jsonapi::Resource<ClinicianAttributes>;
