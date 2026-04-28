//! Generic [JSON:API](https://jsonapi.org/) deserialization envelopes.
//!
//! Use these in place of per-resource `*Resource` / `*ListResponse` structs:
//!
//! ```ignore
//! let single: Single<TeamAttributes> = api.get("teams/abc").await?;
//! let list:   List<TeamAttributes>   = api.get("teams").await?;
//! ```
//!
//! Relationships default to `()` (ignored). Pass an explicit type when a
//! command needs them.

use serde::Deserialize;

/// A single JSON:API resource object: `{ "type", "id", "attributes", "relationships" }`.
#[derive(Debug, Deserialize)]
#[serde(bound(deserialize = "A: serde::Deserialize<'de>, R: Default + serde::Deserialize<'de>"))]
pub struct Resource<A, R = ()> {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub attributes: A,
    #[serde(default)]
    pub relationships: R,
}

/// A JSON:API top-level document: `{ "data": ... }`.
#[derive(Debug, Deserialize)]
pub struct Document<T> {
    pub data: T,
}

/// Single-resource response: `{ "data": { ... } }`.
pub type Single<A, R = ()> = Document<Resource<A, R>>;

/// Multi-resource response: `{ "data": [ ... ] }`.
pub type List<A, R = ()> = Document<Vec<Resource<A, R>>>;

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    struct TeamAttrs {
        name: String,
        abbr: String,
    }

    #[test]
    fn test_single_deserializes() {
        let json = r#"{
            "data": {
                "type": "teams",
                "id": "uuid-1",
                "attributes": { "name": "Nursing", "abbr": "NUR" }
            }
        }"#;
        let doc: Single<TeamAttrs> = serde_json::from_str(json).unwrap();
        assert_eq!(doc.data.id, "uuid-1");
        assert_eq!(doc.data.kind, "teams");
        assert_eq!(doc.data.attributes.name, "Nursing");
        assert_eq!(doc.data.attributes.abbr, "NUR");
    }

    #[test]
    fn test_list_deserializes() {
        let json = r#"{
            "data": [
                { "type": "teams", "id": "a", "attributes": { "name": "A", "abbr": "A" } },
                { "type": "teams", "id": "b", "attributes": { "name": "B", "abbr": "B" } }
            ]
        }"#;
        let doc: List<TeamAttrs> = serde_json::from_str(json).unwrap();
        assert_eq!(doc.data.len(), 2);
        assert_eq!(doc.data[0].id, "a");
        assert_eq!(doc.data[1].id, "b");
    }

    #[test]
    fn test_relationships_default_to_unit_when_absent() {
        let json = r#"{
            "data": {
                "type": "teams",
                "id": "uuid-1",
                "attributes": { "name": "Nursing", "abbr": "NUR" }
            }
        }"#;
        // No `relationships` key — `()` default lets it deserialize cleanly.
        let _: Single<TeamAttrs> = serde_json::from_str(json).unwrap();
    }

    #[test]
    fn test_typed_relationships() {
        #[derive(Debug, Deserialize, Default)]
        struct ClinicianRels {
            #[serde(default)]
            team: Option<TeamRef>,
        }
        #[derive(Debug, Deserialize, Default)]
        struct TeamRef {
            data: TeamRefData,
        }
        #[derive(Debug, Deserialize, Default)]
        struct TeamRefData {
            id: String,
        }

        let json = r#"{
            "data": {
                "type": "clinicians",
                "id": "c1",
                "attributes": { "name": "x", "abbr": "x" },
                "relationships": {
                    "team": { "data": { "id": "team-uuid" } }
                }
            }
        }"#;
        let doc: Single<TeamAttrs, ClinicianRels> = serde_json::from_str(json).unwrap();
        assert_eq!(doc.data.relationships.team.unwrap().data.id, "team-uuid");
    }

    #[test]
    fn test_empty_list() {
        let json = r#"{ "data": [] }"#;
        let doc: List<TeamAttrs> = serde_json::from_str(json).unwrap();
        assert!(doc.data.is_empty());
    }

    #[test]
    fn test_missing_attributes_errors() {
        let json = r#"{ "data": { "type": "teams", "id": "x" } }"#;
        let result: Result<Single<TeamAttrs>, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }
}
