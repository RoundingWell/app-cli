//! Output types for clinician commands. Each implements `CommandOutput` for
//! the human-readable form; `Serialize` carries the JSON form.

use serde::Serialize;

use crate::output::CommandOutput;

#[derive(Debug, Serialize)]
pub struct PrepareOutput {
    #[serde(rename = "id")]
    pub clinician_id: String,
    #[serde(rename = "name")]
    pub clinician_name: String,
    pub is_staff: bool,
    pub role_id: String,
    pub role_name: String,
    pub team_id: String,
    pub team_name: String,
    pub hidden: bool,
    pub workspace_ids: Vec<String>,
}

impl CommandOutput for PrepareOutput {
    fn plain(&self) -> String {
        let kind = if self.is_staff { "staff" } else { "employee" };
        let ws = self.workspace_ids.join(", ");
        format!(
            "{} ({}) prepared as {}: role={}, team={}, hidden={}, workspaces=[{}]",
            self.clinician_name,
            self.clinician_id,
            kind,
            self.role_name,
            self.team_name,
            self.hidden,
            ws
        )
    }
}

#[derive(Debug, Serialize)]
pub struct GrantOutput {
    pub clinician_id: String,
    pub clinician_name: String,
    pub role_id: String,
    pub role_name: String,
}

impl CommandOutput for GrantOutput {
    fn plain(&self) -> String {
        format!(
            "{} ({}) granted '{}' role",
            self.clinician_name, self.clinician_id, self.role_name
        )
    }
}

#[derive(Debug, Serialize)]
pub struct AssignTeamOutput {
    pub clinician_id: String,
    pub clinician_name: String,
    pub team_id: String,
    pub team_name: String,
}

impl CommandOutput for AssignTeamOutput {
    fn plain(&self) -> String {
        format!(
            "{} ({}) assigned to '{}' team",
            self.clinician_name, self.clinician_id, self.team_name
        )
    }
}

#[derive(Debug, Serialize)]
pub struct ClinicianOutput {
    pub id: String,
    pub name: String,
    pub email: String,
    pub enabled: bool,
}

impl CommandOutput for ClinicianOutput {
    fn plain(&self) -> String {
        let status = if self.enabled { "enabled" } else { "disabled" };
        format!("{} ({}) is now {}", self.name, self.id, status)
    }
}

#[derive(Debug, Serialize)]
pub struct ClinicianUpdateOutput {
    pub id: String,
    pub name: String,
    pub email: String,
    pub enabled: bool,
    pub npi: Option<String>,
    pub credentials: Vec<String>,
    #[serde(skip)]
    pub updated_field: String,
}

impl CommandOutput for ClinicianUpdateOutput {
    fn plain(&self) -> String {
        format!("{} ({}) updated {}", self.name, self.id, self.updated_field)
    }
}

#[derive(Debug, Serialize)]
pub struct ClinicianRegisterOutput {
    pub id: String,
    pub name: String,
    pub email: String,
}

impl CommandOutput for ClinicianRegisterOutput {
    fn plain(&self) -> String {
        format!("{} ({}) registered", self.name, self.id)
    }
}

#[derive(Debug, Serialize)]
pub struct ClinicianShowOutput {
    pub id: String,
    pub name: String,
    pub email: String,
    pub enabled: bool,
    pub npi: Option<String>,
    pub credentials: Vec<String>,
}

impl CommandOutput for ClinicianShowOutput {
    fn plain(&self) -> String {
        [
            format!("id:          {}", self.id),
            format!("name:        {}", self.name),
            format!("email:       {}", self.email),
            format!("enabled:     {}", self.enabled),
            format!("npi:         {}", self.npi.as_deref().unwrap_or("")),
            format!("credentials: {}", self.credentials.join(", ")),
        ]
        .join("\n")
    }
}
