use serde_derive::Deserialize;
use serde_derive::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Root {
  pub version: String,
  pub incident: Incident,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Incident {
  pub incident_id: String,
  pub scoping_project_id: String,
  pub scoping_project_number: i64,
  pub url: String,
  pub started_at: i64,
  pub ended_at: i64,
  pub state: String,
  pub summary: String,
  pub resource_type_display_name: String,
  pub resource_display_name: String,
}
