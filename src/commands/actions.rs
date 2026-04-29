//! `rw actions trace <uuid>` — traces an action's patient, program, and form
//! to surface any misalignment between their workspace memberships.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use crate::cli::{ActionsArgs, ActionsCommands};
use crate::config::AppContext;
use crate::http::ApiClient;
use crate::jsonapi::{List, Single};
use crate::output::{CommandOutput, Output};

pub async fn dispatch(args: ActionsArgs, ctx: &AppContext, out: &Output) -> Result<()> {
    match args.command {
        ActionsCommands::Trace(a) => trace(ctx, &a.uuid, out).await,
    }
}

// --- JSON:API attributes ---

#[derive(Debug, Deserialize)]
struct ActionAttributes {
    #[serde(default)]
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PatientAttributes {
    #[serde(default)]
    first_name: Option<String>,
    #[serde(default)]
    last_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ProgramAttributes {
    #[serde(default)]
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct FormAttributes {
    #[serde(default)]
    name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct WorkspaceAttributes {
    slug: String,
    name: String,
}

// --- Output types ---

/// Internal helper carrying a workspace's id, slug, and name during the
/// gathering phase. Not part of the serialized output.
#[derive(Debug)]
struct WorkspaceRef {
    id: String,
    slug: String,
    name: String,
}

/// Identifying reference to a resource — `{id, name}`. Used at the top level
/// of `TraceOutput` so consumers don't have to do a second lookup to label
/// the action / patient / program / form.
#[derive(Debug, Serialize)]
pub struct ResourceRef {
    pub id: String,
    pub name: Option<String>,
}

/// Per-workspace alignment row. Serialized in the JSON output and rendered
/// as a markdown table in the plain output.
#[derive(Debug, Serialize)]
pub struct WorkspaceAlignment {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub has_patient: bool,
    pub has_program: bool,
    pub has_form: bool,
}

#[derive(Debug, Serialize)]
pub struct TraceOutput {
    pub action: ResourceRef,
    pub patient: ResourceRef,
    pub program: ResourceRef,
    /// `None` when the action has no `form` relationship — not every action
    /// involves a form, so this is expected and the `has_form` column is
    /// omitted from plain output in that case.
    pub form: Option<ResourceRef>,
    pub workspaces: Vec<WorkspaceAlignment>,
}

impl CommandOutput for TraceOutput {
    fn plain(&self) -> String {
        use tabled::builder::Builder;
        use tabled::settings::Style;

        let mut out = format!(
            "action:  {}\npatient: {}\nprogram: {}\n",
            label(&self.action),
            label(&self.patient),
            label(&self.program),
        );
        out.push_str(&format!(
            "form:    {}\n\n",
            self.form
                .as_ref()
                .map(label)
                .unwrap_or_else(|| "(none)".to_string()),
        ));

        let has_form = self.form.is_some();
        let mut builder = Builder::default();
        let mut headers = vec!["workspace", "patient", "program"];
        if has_form {
            headers.push("form");
        }
        builder.push_record(headers);
        for w in &self.workspaces {
            let ws_label = if w.slug.is_empty() {
                w.name.clone()
            } else {
                format!("{} ({})", w.name, w.slug)
            };
            let mut row = vec![
                ws_label,
                mark(w.has_patient).into(),
                mark(w.has_program).into(),
            ];
            if has_form {
                row.push(mark(w.has_form).into());
            }
            builder.push_record(row);
        }
        out.push_str(&builder.build().with(Style::markdown()).to_string());
        out.push('\n');
        out.push('\n');

        let issues = misalignments_from(&self.workspaces, has_form);
        if issues.is_empty() {
            let scope = if has_form {
                "program and form workspaces"
            } else {
                "program workspaces"
            };
            out.push_str(&format!("Alignment: patient workspaces match {}.", scope));
        } else {
            out.push_str("Alignment issues:\n");
            for m in &issues {
                out.push_str(&format!("  - {}\n", render_misalignment(m)));
            }
            if out.ends_with('\n') {
                out.pop();
            }
        }
        out
    }
}

fn label(r: &ResourceRef) -> String {
    match &r.name {
        Some(n) if !n.is_empty() => format!("{} ({})", n, r.id),
        _ => r.id.clone(),
    }
}

const CHECK: &str = "✓";
const CROSS: &str = "✗";

fn mark(present: bool) -> &'static str {
    if present {
        CHECK
    } else {
        CROSS
    }
}

/// Builds the per-workspace alignment matrix from the three resource workspace
/// lists. The result is sorted by workspace name and deduplicated by id.
fn build_workspace_alignments(
    patient: &[WorkspaceRef],
    program: &[WorkspaceRef],
    form: &[WorkspaceRef],
) -> Vec<WorkspaceAlignment> {
    let p: BTreeSet<&str> = patient.iter().map(|w| w.id.as_str()).collect();
    let pr: BTreeSet<&str> = program.iter().map(|w| w.id.as_str()).collect();
    let f: BTreeSet<&str> = form.iter().map(|w| w.id.as_str()).collect();

    let mut seen: BTreeSet<&str> = BTreeSet::new();
    let mut union: Vec<&WorkspaceRef> = Vec::new();
    for w in patient.iter().chain(program.iter()).chain(form.iter()) {
        if seen.insert(w.id.as_str()) {
            union.push(w);
        }
    }
    union.sort_by(|a, b| a.name.cmp(&b.name).then_with(|| a.id.cmp(&b.id)));

    union
        .into_iter()
        .map(|w| WorkspaceAlignment {
            id: w.id.clone(),
            slug: w.slug.clone(),
            name: w.name.clone(),
            has_patient: p.contains(w.id.as_str()),
            has_program: pr.contains(w.id.as_str()),
            has_form: f.contains(w.id.as_str()),
        })
        .collect()
}

/// Plain-output-only summary of a patient↔target misalignment. Derived from
/// the workspace alignment matrix; not part of the serialized JSON.
struct Misalignment<'a> {
    target: &'static str,
    only_on_patient: Vec<&'a WorkspaceAlignment>,
    only_on_target: Vec<&'a WorkspaceAlignment>,
}

fn misalignments_from(
    workspaces: &[WorkspaceAlignment],
    include_form: bool,
) -> Vec<Misalignment<'_>> {
    let mut issues = Vec::new();
    let program_only_patient: Vec<_> = workspaces
        .iter()
        .filter(|w| w.has_patient && !w.has_program)
        .collect();
    let program_only_target: Vec<_> = workspaces
        .iter()
        .filter(|w| !w.has_patient && w.has_program)
        .collect();
    if !program_only_patient.is_empty() || !program_only_target.is_empty() {
        issues.push(Misalignment {
            target: "program",
            only_on_patient: program_only_patient,
            only_on_target: program_only_target,
        });
    }

    if include_form {
        let form_only_patient: Vec<_> = workspaces
            .iter()
            .filter(|w| w.has_patient && !w.has_form)
            .collect();
        let form_only_target: Vec<_> = workspaces
            .iter()
            .filter(|w| !w.has_patient && w.has_form)
            .collect();
        if !form_only_patient.is_empty() || !form_only_target.is_empty() {
            issues.push(Misalignment {
                target: "form",
                only_on_patient: form_only_patient,
                only_on_target: form_only_target,
            });
        }
    }
    issues
}

fn render_misalignment(m: &Misalignment) -> String {
    let mut parts = Vec::new();
    if !m.only_on_patient.is_empty() {
        parts.push(format!(
            "only on patient: [{}]",
            alignment_labels(&m.only_on_patient).join(", ")
        ));
    }
    if !m.only_on_target.is_empty() {
        parts.push(format!(
            "only on {}: [{}]",
            m.target,
            alignment_labels(&m.only_on_target).join(", ")
        ));
    }
    format!(
        "patient workspaces do not align with {}; {}",
        m.target,
        parts.join("; ")
    )
}

fn alignment_labels(refs: &[&WorkspaceAlignment]) -> Vec<String> {
    refs.iter()
        .map(|w| {
            if w.name.is_empty() {
                w.id.clone()
            } else {
                w.name.clone()
            }
        })
        .collect()
}

// --- Public command function ---

pub async fn trace(ctx: &AppContext, uuid: &str, out: &Output) -> Result<()> {
    let api = ApiClient::new(ctx).await?;

    // 1. Fetch the action with its relationships.
    let action: Single<ActionAttributes> = api
        .get(&format!("actions/{}", uuid))
        .await
        .with_context(|| format!("failed to fetch action {}", uuid))?;
    let action_rels = &action.data.relationships;
    let patient_id = related_id(action_rels, "patient")
        .ok_or_else(|| anyhow::anyhow!("action {} has no patient relationship", uuid))?;
    let program_id = related_id(action_rels, "program")
        .ok_or_else(|| anyhow::anyhow!("action {} has no program relationship", uuid))?;
    // Form is optional — not every action involves a form.
    let form_id = related_id(action_rels, "form");

    // 2. Fetch all workspaces. Each workspace's `relationships.forms` and
    //    `relationships.programs` lists the form/program ids it contains —
    //    this is how we determine which workspaces a form or program lives in,
    //    since form and program resources don't carry relationships themselves.
    let ws_list: List<WorkspaceAttributes> = api.get("workspaces").await?;
    let workspace_index: BTreeMap<String, (String, String)> = ws_list
        .data
        .iter()
        .map(|w| {
            (
                w.id.clone(),
                (w.attributes.slug.clone(), w.attributes.name.clone()),
            )
        })
        .collect();

    // 3-5. Fetch related resources in parallel for their display names.
    let patient_path = format!("patients/{}", patient_id);
    let program_path = format!("programs/{}", program_id);
    let (patient, program, form) = tokio::try_join!(
        api.get::<Single<PatientAttributes>>(&patient_path),
        api.get::<Single<ProgramAttributes>>(&program_path),
        async {
            match form_id.as_deref() {
                Some(id) => api
                    .get::<Single<FormAttributes>>(&format!("forms/{}", id))
                    .await
                    .map(Some),
                None => Ok(None),
            }
        },
    )?;

    let patient_workspaces = workspace_refs(&patient.data.relationships, &workspace_index);
    let program_workspaces = workspaces_referencing(&ws_list.data, "programs", &program_id);
    let form_workspaces: Vec<WorkspaceRef> = match &form_id {
        Some(id) => workspaces_referencing(&ws_list.data, "forms", id),
        None => Vec::new(),
    };

    let workspaces =
        build_workspace_alignments(&patient_workspaces, &program_workspaces, &form_workspaces);

    let patient_full_name = format!(
        "{} {}",
        patient.data.attributes.first_name.unwrap_or_default(),
        patient.data.attributes.last_name.unwrap_or_default()
    )
    .trim()
    .to_string();
    let patient_name = (!patient_full_name.is_empty()).then_some(patient_full_name);

    let form_ref = form.map(|f| ResourceRef {
        id: f.data.id,
        name: f.data.attributes.name,
    });

    out.print(&TraceOutput {
        action: ResourceRef {
            id: action.data.id,
            name: action.data.attributes.name,
        },
        patient: ResourceRef {
            id: patient.data.id,
            name: patient_name,
        },
        program: ResourceRef {
            id: program.data.id,
            name: program.data.attributes.name,
        },
        form: form_ref,
        workspaces,
    });
    Ok(())
}

/// Reads `relationships.<key>.data.id` from an untyped relationships object.
fn related_id(rels: &serde_json::Value, key: &str) -> Option<String> {
    rels.get(key)?
        .get("data")?
        .get("id")?
        .as_str()
        .map(str::to_string)
}

/// Reads `relationships.workspaces.data` (an array of resource identifiers)
/// and resolves each id against the provided workspace index. Workspaces not
/// found in the index are emitted with empty slug/name.
fn workspace_refs(
    rels: &serde_json::Value,
    index: &BTreeMap<String, (String, String)>,
) -> Vec<WorkspaceRef> {
    let Some(data) = rels.get("workspaces").and_then(|w| w.get("data")) else {
        return Vec::new();
    };
    let Some(arr) = data.as_array() else {
        return Vec::new();
    };
    let mut refs: Vec<WorkspaceRef> = arr
        .iter()
        .filter_map(|v| v.get("id").and_then(|i| i.as_str()))
        .map(|id| {
            let (slug, name) = index
                .get(id)
                .cloned()
                .unwrap_or_else(|| (String::new(), String::new()));
            WorkspaceRef {
                id: id.to_string(),
                slug,
                name,
            }
        })
        .collect();
    refs.sort_by(|a, b| a.name.cmp(&b.name));
    refs
}

/// Returns the workspaces whose `relationships.<rel_name>.data` array contains
/// a resource identifier matching `target_id`. Used for forms and programs,
/// which appear as identifiers under each workspace's relationships rather
/// than carrying their own workspace memberships.
fn workspaces_referencing(
    workspaces: &[crate::jsonapi::Resource<WorkspaceAttributes>],
    rel_name: &str,
    target_id: &str,
) -> Vec<WorkspaceRef> {
    let mut refs: Vec<WorkspaceRef> = workspaces
        .iter()
        .filter(|w| {
            w.relationships
                .get(rel_name)
                .and_then(|r| r.get("data"))
                .and_then(|d| d.as_array())
                .map(|arr| {
                    arr.iter()
                        .any(|item| item.get("id").and_then(|i| i.as_str()) == Some(target_id))
                })
                .unwrap_or(false)
        })
        .map(|w| WorkspaceRef {
            id: w.id.clone(),
            slug: w.attributes.slug.clone(),
            name: w.attributes.name.clone(),
        })
        .collect();
    refs.sort_by(|a, b| a.name.cmp(&b.name));
    refs
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::Server;

    struct TestAuthGuard {
        dir: tempfile::TempDir,
    }

    impl TestAuthGuard {
        fn new() -> Self {
            use crate::auth_cache::{save_auth_cache, AuthCache};
            let dir = tempfile::TempDir::new().unwrap();
            let cache = AuthCache::Bearer {
                access_token: "test-token".to_string(),
                refresh_token: None,
                expires_at: i64::MAX,
            };
            save_auth_cache(dir.path(), "test", &cache).unwrap();
            TestAuthGuard { dir }
        }

        fn app_context(&self, base_url: &str) -> AppContext {
            use crate::cli::Stage;
            AppContext {
                config_dir: self.dir.path().to_path_buf(),
                profile: "test".to_string(),
                auth_profile: "test".to_string(),
                stage: Stage::Dev,
                auth_stage: Stage::Dev,
                base_url: base_url.to_string(),
                defaults: BTreeMap::new(),
            }
        }
    }

    fn action_response(
        action_id: &str,
        patient_id: &str,
        program_id: &str,
        form_id: &str,
    ) -> String {
        serde_json::json!({
            "data": {
                "type": "actions",
                "id": action_id,
                "attributes": { "name": "Daily Check-in" },
                "relationships": {
                    "patient": { "data": { "type": "patients", "id": patient_id } },
                    "program": { "data": { "type": "programs", "id": program_id } },
                    "form":    { "data": { "type": "forms",    "id": form_id    } }
                }
            }
        })
        .to_string()
    }

    /// Builds a workspaces list response. Each workspace can declare which
    /// program and form ids it contains via `programs` and `forms`.
    fn workspace_list(workspaces: &[WorkspaceFixture]) -> String {
        let data: Vec<serde_json::Value> = workspaces
            .iter()
            .map(|w| {
                let programs: Vec<_> = w
                    .programs
                    .iter()
                    .map(|p| serde_json::json!({ "type": "programs", "id": p }))
                    .collect();
                let forms: Vec<_> = w
                    .forms
                    .iter()
                    .map(|f| serde_json::json!({ "type": "forms", "id": f }))
                    .collect();
                serde_json::json!({
                    "type": "workspaces",
                    "id": w.id,
                    "attributes": { "slug": w.slug, "name": w.name },
                    "relationships": {
                        "programs": { "data": programs },
                        "forms":    { "data": forms }
                    }
                })
            })
            .collect();
        serde_json::json!({ "data": data }).to_string()
    }

    struct WorkspaceFixture<'a> {
        id: &'a str,
        slug: &'a str,
        name: &'a str,
        programs: Vec<&'a str>,
        forms: Vec<&'a str>,
    }

    fn patient_response(id: &str, first: &str, last: &str, workspace_ids: &[&str]) -> String {
        let ws_data: Vec<_> = workspace_ids
            .iter()
            .map(|w| serde_json::json!({ "type": "workspaces", "id": w }))
            .collect();
        serde_json::json!({
            "data": {
                "type": "patients",
                "id": id,
                "attributes": { "first_name": first, "last_name": last },
                "relationships": {
                    "workspaces": { "data": ws_data }
                }
            }
        })
        .to_string()
    }

    /// Forms and programs don't carry relationships — only attributes.
    fn bare_resource(kind: &str, id: &str, attrs: serde_json::Value) -> String {
        serde_json::json!({
            "data": {
                "type": kind,
                "id": id,
                "attributes": attrs
            }
        })
        .to_string()
    }

    #[tokio::test]
    async fn test_trace_aligned_workspaces_reports_no_issues() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;
        let action_id = "11111111-1111-1111-1111-111111111111";
        let patient_id = "22222222-2222-2222-2222-222222222222";
        let program_id = "33333333-3333-3333-3333-333333333333";
        let form_id = "44444444-4444-4444-4444-444444444444";
        let ws = "ws000000-0000-0000-0000-000000000001";

        server
            .mock("GET", format!("/actions/{}", action_id).as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(action_response(action_id, patient_id, program_id, form_id))
            .create_async()
            .await;
        server
            .mock("GET", "/workspaces")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(workspace_list(&[WorkspaceFixture {
                id: ws,
                slug: "main",
                name: "Main",
                programs: vec![program_id],
                forms: vec![form_id],
            }]))
            .create_async()
            .await;
        server
            .mock("GET", format!("/patients/{}", patient_id).as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(patient_response(patient_id, "Jane", "Doe", &[ws]))
            .create_async()
            .await;
        server
            .mock("GET", format!("/programs/{}", program_id).as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(bare_resource(
                "programs",
                program_id,
                serde_json::json!({ "name": "Diabetes Care" }),
            ))
            .create_async()
            .await;
        server
            .mock("GET", format!("/forms/{}", form_id).as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(bare_resource(
                "forms",
                form_id,
                serde_json::json!({ "name": "Discharge Survey" }),
            ))
            .create_async()
            .await;

        let out = Output { json: false };
        trace(&_auth.app_context(&server.url()), action_id, &out)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_trace_misaligned_workspaces_reports_issues() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;
        let action_id = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa";
        let patient_id = "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb";
        let program_id = "cccccccc-cccc-cccc-cccc-cccccccccccc";
        let form_id = "dddddddd-dddd-dddd-dddd-dddddddddddd";
        let ws_a = "ws000000-0000-0000-0000-00000000000a";
        let ws_b = "ws000000-0000-0000-0000-00000000000b";

        server
            .mock("GET", format!("/actions/{}", action_id).as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(action_response(action_id, patient_id, program_id, form_id))
            .create_async()
            .await;
        // Patient is in ws_a; program lives in ws_b; form lives in ws_a.
        server
            .mock("GET", "/workspaces")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(workspace_list(&[
                WorkspaceFixture {
                    id: ws_a,
                    slug: "alpha",
                    name: "Alpha",
                    programs: vec![],
                    forms: vec![form_id],
                },
                WorkspaceFixture {
                    id: ws_b,
                    slug: "beta",
                    name: "Beta",
                    programs: vec![program_id],
                    forms: vec![],
                },
            ]))
            .create_async()
            .await;
        server
            .mock("GET", format!("/patients/{}", patient_id).as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(patient_response(patient_id, "John", "Smith", &[ws_a]))
            .create_async()
            .await;
        server
            .mock("GET", format!("/programs/{}", program_id).as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(bare_resource(
                "programs",
                program_id,
                serde_json::json!({ "name": "Hypertension" }),
            ))
            .create_async()
            .await;
        server
            .mock("GET", format!("/forms/{}", form_id).as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(bare_resource(
                "forms",
                form_id,
                serde_json::json!({ "name": "Intake Form" }),
            ))
            .create_async()
            .await;

        let out = Output { json: false };
        let result = trace(&_auth.app_context(&server.url()), action_id, &out).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_trace_action_not_found_errors() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;
        let action_id = "deadbeef-dead-beef-dead-beefdeadbeef";

        server
            .mock("GET", format!("/actions/{}", action_id).as_str())
            .with_status(404)
            .with_body("not found")
            .create_async()
            .await;

        let out = Output { json: false };
        let err = trace(&_auth.app_context(&server.url()), action_id, &out)
            .await
            .unwrap_err();
        let msg = format!("{:#}", err);
        assert!(msg.contains("failed to fetch action"));
    }

    #[tokio::test]
    async fn test_trace_action_missing_relationship_errors() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;
        let action_id = "11111111-2222-3333-4444-555555555555";

        let body = serde_json::json!({
            "data": {
                "type": "actions",
                "id": action_id,
                "attributes": {},
                "relationships": {
                    "patient": { "data": { "type": "patients", "id": "p" } }
                }
            }
        })
        .to_string();
        server
            .mock("GET", format!("/actions/{}", action_id).as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(body)
            .create_async()
            .await;

        let out = Output { json: false };
        let err = trace(&_auth.app_context(&server.url()), action_id, &out)
            .await
            .unwrap_err();
        assert!(err.to_string().contains("no program relationship"));
    }

    #[tokio::test]
    async fn test_trace_action_without_form_succeeds() {
        let _auth = TestAuthGuard::new();
        let mut server = Server::new_async().await;
        let action_id = "feedface-feed-face-feed-facefeedface";
        let patient_id = "feedbabe-feed-babe-feed-babefeedbabe";
        let program_id = "facefeed-face-feed-face-feedfacefeed";
        let ws = "ws000000-0000-0000-0000-000000000099";

        let action_body = serde_json::json!({
            "data": {
                "type": "actions",
                "id": action_id,
                "attributes": { "name": "Reminder" },
                "relationships": {
                    "patient": { "data": { "type": "patients", "id": patient_id } },
                    "program": { "data": { "type": "programs", "id": program_id } }
                }
            }
        })
        .to_string();
        server
            .mock("GET", format!("/actions/{}", action_id).as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(action_body)
            .create_async()
            .await;
        server
            .mock("GET", "/workspaces")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(workspace_list(&[WorkspaceFixture {
                id: ws,
                slug: "main",
                name: "Main",
                programs: vec![program_id],
                forms: vec![],
            }]))
            .create_async()
            .await;
        server
            .mock("GET", format!("/patients/{}", patient_id).as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(patient_response(patient_id, "Form", "Less", &[ws]))
            .create_async()
            .await;
        server
            .mock("GET", format!("/programs/{}", program_id).as_str())
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(bare_resource(
                "programs",
                program_id,
                serde_json::json!({ "name": "Wellness" }),
            ))
            .create_async()
            .await;
        // No /forms/... mock — the trace should not call it.

        let out = Output { json: false };
        trace(&_auth.app_context(&server.url()), action_id, &out)
            .await
            .unwrap();
    }

    // --- pure-helper tests ---

    fn ws(id: &str, name: &str) -> WorkspaceRef {
        WorkspaceRef {
            id: id.to_string(),
            slug: name.to_lowercase(),
            name: name.to_string(),
        }
    }

    fn align(
        id: &str,
        name: &str,
        has_patient: bool,
        has_program: bool,
        has_form: bool,
    ) -> WorkspaceAlignment {
        WorkspaceAlignment {
            id: id.to_string(),
            slug: name.to_lowercase(),
            name: name.to_string(),
            has_patient,
            has_program,
            has_form,
        }
    }

    #[test]
    fn test_misalignments_from_returns_empty_when_aligned() {
        let ws = vec![align("a", "Alpha", true, true, true)];
        assert!(misalignments_from(&ws, true).is_empty());
    }

    #[test]
    fn test_misalignments_from_flags_program_and_form() {
        let ws = vec![
            align("a", "Alpha", true, false, false),
            align("b", "Beta", false, true, true),
        ];
        let issues = misalignments_from(&ws, true);
        assert_eq!(issues.len(), 2);
        assert_eq!(issues[0].target, "program");
        assert_eq!(issues[0].only_on_patient.len(), 1);
        assert_eq!(issues[0].only_on_patient[0].name, "Alpha");
        assert_eq!(issues[0].only_on_target.len(), 1);
        assert_eq!(issues[0].only_on_target[0].name, "Beta");
        assert_eq!(issues[1].target, "form");
    }

    #[test]
    fn test_misalignments_from_skips_form_when_disabled() {
        // Patient↔form disagreement should be ignored when form is absent.
        let ws = vec![align("a", "Alpha", true, true, false)];
        let issues = misalignments_from(&ws, false);
        assert!(issues.is_empty());
    }

    #[test]
    fn test_render_misalignment_includes_both_sides() {
        let alpha = align("a", "Alpha", true, false, false);
        let beta = align("b", "Beta", false, true, false);
        let m = Misalignment {
            target: "program",
            only_on_patient: vec![&alpha],
            only_on_target: vec![&beta],
        };
        let s = render_misalignment(&m);
        assert!(s.contains("do not align with program"));
        assert!(s.contains("only on patient: [Alpha]"));
        assert!(s.contains("only on program: [Beta]"));
    }

    #[test]
    fn test_related_id_extracts_id_from_relationship() {
        let rels = serde_json::json!({
            "patient": { "data": { "type": "patients", "id": "p-1" } }
        });
        assert_eq!(related_id(&rels, "patient").as_deref(), Some("p-1"));
        assert_eq!(related_id(&rels, "program"), None);
    }

    #[test]
    fn test_workspace_refs_resolves_via_index() {
        let mut idx = BTreeMap::new();
        idx.insert("a".to_string(), ("alpha".to_string(), "Alpha".to_string()));
        idx.insert("b".to_string(), ("beta".to_string(), "Beta".to_string()));

        let rels = serde_json::json!({
            "workspaces": { "data": [
                { "type": "workspaces", "id": "b" },
                { "type": "workspaces", "id": "a" }
            ] }
        });
        let refs = workspace_refs(&rels, &idx);
        assert_eq!(refs.len(), 2);
        assert_eq!(refs[0].id, "a");
        assert_eq!(refs[0].slug, "alpha");
        assert_eq!(refs[1].id, "b");
    }

    #[test]
    fn test_workspace_refs_empty_when_relationship_absent() {
        let rels = serde_json::json!({});
        let idx = BTreeMap::new();
        assert!(workspace_refs(&rels, &idx).is_empty());
    }

    #[test]
    fn test_workspaces_referencing_filters_by_relationship_data() {
        use crate::jsonapi::Resource;

        fn workspace(
            id: &str,
            slug: &str,
            name: &str,
            rels: serde_json::Value,
        ) -> Resource<WorkspaceAttributes> {
            serde_json::from_value(serde_json::json!({
                "type": "workspaces",
                "id": id,
                "attributes": { "slug": slug, "name": name },
                "relationships": rels,
            }))
            .unwrap()
        }

        let workspaces = vec![
            workspace(
                "w-a",
                "alpha",
                "Alpha",
                serde_json::json!({
                    "programs": { "data": [{ "type": "programs", "id": "p-1" }] },
                    "forms": { "data": [] }
                }),
            ),
            workspace(
                "w-b",
                "beta",
                "Beta",
                serde_json::json!({
                    "programs": { "data": [{ "type": "programs", "id": "p-2" }] },
                    "forms": { "data": [{ "type": "forms", "id": "f-1" }] }
                }),
            ),
        ];

        let p1 = workspaces_referencing(&workspaces, "programs", "p-1");
        assert_eq!(p1.len(), 1);
        assert_eq!(p1[0].id, "w-a");

        let f1 = workspaces_referencing(&workspaces, "forms", "f-1");
        assert_eq!(f1.len(), 1);
        assert_eq!(f1[0].id, "w-b");

        let none = workspaces_referencing(&workspaces, "programs", "p-missing");
        assert!(none.is_empty());
    }

    #[test]
    fn test_build_workspace_alignments_unions_and_sorts_by_name() {
        let p = vec![ws("a", "Alpha"), ws("c", "Gamma")];
        let pr = vec![ws("b", "Beta"), ws("c", "Gamma")];
        let f = vec![ws("a", "Alpha")];

        let rows = build_workspace_alignments(&p, &pr, &f);
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].name, "Alpha");
        assert!(rows[0].has_patient);
        assert!(!rows[0].has_program);
        assert!(rows[0].has_form);
        assert_eq!(rows[1].name, "Beta");
        assert!(!rows[1].has_patient);
        assert!(rows[1].has_program);
        assert!(!rows[1].has_form);
        assert_eq!(rows[2].name, "Gamma");
        assert!(rows[2].has_patient);
        assert!(rows[2].has_program);
        assert!(!rows[2].has_form);
    }

    #[test]
    fn test_build_workspace_alignments_empty_when_no_workspaces() {
        assert!(build_workspace_alignments(&[], &[], &[]).is_empty());
    }

    fn rref(id: &str, name: &str) -> ResourceRef {
        ResourceRef {
            id: id.to_string(),
            name: Some(name.to_string()),
        }
    }

    #[test]
    fn test_trace_output_plain_aligned() {
        let output = TraceOutput {
            action: rref("act-1", "Daily"),
            patient: rref("p-1", "Jane Doe"),
            program: rref("pr-1", "Diabetes"),
            form: Some(rref("f-1", "Survey")),
            workspaces: vec![align("w-1", "Main", true, true, true)],
        };
        let s = output.plain();
        assert!(s.contains("action:  Daily (act-1)"));
        assert!(s.contains("patient: Jane Doe (p-1)"));
        assert!(s.contains("program: Diabetes (pr-1)"));
        assert!(s.contains("form:    Survey (f-1)"));
        assert!(s.contains("Main (main)"));
        assert!(s.contains(CHECK));
        assert!(s.contains("Alignment: patient workspaces match"));
    }

    #[test]
    fn test_trace_output_plain_with_issues() {
        let output = TraceOutput {
            action: ResourceRef {
                id: "act-2".to_string(),
                name: None,
            },
            patient: rref("p-2", "John"),
            program: rref("pr-2", "Heart"),
            form: Some(rref("f-2", "Intake")),
            workspaces: vec![
                align("w-1", "Alpha", true, false, true),
                align("w-2", "Beta", false, true, false),
            ],
        };
        let s = output.plain();
        assert!(s.contains("Alpha (alpha)"));
        assert!(s.contains("Beta (beta)"));
        assert!(s.contains(CHECK));
        assert!(s.contains(CROSS));
        assert!(s.contains("Alignment issues:"));
        assert!(s.contains("only on patient: [Alpha]"));
        assert!(s.contains("only on program: [Beta]"));
    }

    #[test]
    fn test_trace_output_plain_no_form() {
        let output = TraceOutput {
            action: rref("act-3", "Reminder"),
            patient: rref("p-3", "Jane"),
            program: rref("pr-3", "Wellness"),
            form: None,
            workspaces: vec![align("w-1", "Main", true, true, false)],
        };
        let s = output.plain();
        assert!(s.contains("form:    (none)"));
        // Form column should be omitted from the matrix header and rows.
        assert!(!s.contains("| form |"));
        assert!(s.contains("Alignment: patient workspaces match program workspaces"));
    }

    #[test]
    fn test_trace_output_json_omits_skipped_fields() {
        let output = TraceOutput {
            action: rref("act-1", "Daily"),
            patient: rref("p-1", "Jane"),
            program: rref("pr-1", "Diabetes"),
            form: None,
            workspaces: vec![align("w-1", "Main", true, true, false)],
        };
        let json = serde_json::to_value(&output).unwrap();
        assert_eq!(json["action"]["id"], "act-1");
        assert_eq!(json["action"]["name"], "Daily");
        assert!(json["form"].is_null());
        assert_eq!(json["workspaces"][0]["has_patient"], true);
        assert_eq!(json["workspaces"][0]["has_form"], false);
        // Top-level should only have these keys.
        let obj = json.as_object().unwrap();
        let keys: BTreeSet<&str> = obj.keys().map(String::as_str).collect();
        let expected: BTreeSet<&str> =
            ["action", "patient", "program", "form", "workspaces"].into();
        assert_eq!(keys, expected);
    }
}
