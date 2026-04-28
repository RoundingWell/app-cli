//! JSON:API deserialization types for clinician responses.

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

#[derive(Debug, Deserialize)]
pub(super) struct ClinicianResource {
    pub(super) id: String,
    pub(super) attributes: ClinicianAttributes,
}

#[derive(Debug, Deserialize)]
pub(super) struct ClinicianListResponse {
    pub(super) data: Vec<ClinicianResource>,
}

#[derive(Debug, Deserialize)]
pub(super) struct ClinicianSingleResponse {
    pub(super) data: ClinicianResource,
}
