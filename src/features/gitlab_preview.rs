use std::{
  collections::HashMap,
  sync::{Arc, RwLock},
};

use chrono::{Duration, NaiveDateTime, Utc};
use regex::Regex;
use reqwest::{header, Client};
use serde_derive::Deserialize;
use serde_derive::Serialize;
use serenity::{client::Context, model::channel::Message, utils::Colour};

const GITLAB_PROJECT_URL: &str =
  "https://lab.blackfoot.io/api/v4/projects/?simple=true&per_page=100&order_by=last_activity_at";
const GITLAB_PROJECT_MR_URL: &str = "https://lab.blackfoot.io/api/v4/projects/{}/merge_requests";

type TProjectMrCache = HashMap<i64, (Vec<MergeRequest>, NaiveDateTime)>;

lazy_static! {
  static ref REQWEST_CLIENT_GITLAB: Client = build_client();
  static ref REGEX_URL_PARSE: Regex =
    Regex::new(r#"https://lab.blackfoot.io/(([A-z-/]+)/-[A-z-/0-9]+|([A-z-/0-9]+))(#[0-9,A-z]*)?"#)
      .unwrap();
  static ref REGEX_URL_PARSE_ONLY: Regex = Regex::new(
    r#"^https://lab.blackfoot.io/(([A-z-/]+)/-[A-z-/0-9]+|([A-z-/0-9]+))(#[0-9,A-z]*)?$"#
  )
  .unwrap();
  static ref REGEX_MERGE_ID: Regex = Regex::new(r#"/merge_requests/([0-9]{1,4})"#).unwrap();
  static ref PROJECTS_MR_CACHE: Arc<RwLock<TProjectMrCache>> =
    Arc::new(RwLock::new(HashMap::new()));
}

static PROJECTS_CACHE: RwLock<Option<(Vec<Project>, NaiveDateTime)>> = RwLock::new(None);

fn build_client() -> Client {
  let mut headers = header::HeaderMap::new();
  let gitlab_token = std::env::var("GITLAB_TOKEN").expect("missing gitlab token");
  headers.insert(
    "Authorization",
    header::HeaderValue::from_str(&format!("Bearer {gitlab_token}"))
      .expect("unable to create header value"),
  );

  reqwest::Client::builder()
    .default_headers(headers)
    .build()
    .expect("unablet to build reqwest client")
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Project {
  pub id: i64,
  pub description: String,
  pub name: String,
  pub path_with_namespace: String,
  pub web_url: String,
  pub avatar_url: Option<String>,
  pub namespace: ProjectNamespace,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProjectNamespace {
  pub avatar_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct MergeRequest {
  pub iid: i64,
  pub title: String,
  pub description: String,
  pub state: String,
  pub created_at: String,
  pub author: Author,
  pub source_branch: String,
  pub merge_status: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Author {
  username: String,
}

fn check_message_should_preview(message: &str) -> Option<(String, Option<i64>)> {
  if let Some(caps) = REGEX_URL_PARSE.captures(message) {
    if let Some(cap) = caps.get(2) {
      return Some((
        String::from(cap.as_str()),
        REGEX_MERGE_ID.captures(message).map(|c| {
          c.get(1)
            .map(|number| number.as_str().parse::<i64>().unwrap())
            .unwrap()
        }),
      ));
    }
    if let Some(cap) = caps.get(3) {
      return Some((String::from(cap.as_str()), None));
    }
  }
  None
}

/// Check if the date given as passed by more than hour
fn is_outdated(updated_at: &NaiveDateTime) -> bool {
  *updated_at + Duration::hours(1) > Utc::now().naive_utc()
}

async fn gitlab_get_projects() -> Result<Vec<Project>, anyhow::Error> {
  let request = REQWEST_CLIENT_GITLAB.get(GITLAB_PROJECT_URL).send().await?;
  let projects: Vec<Project> = request.json().await.unwrap();

  let mut projects_cache = PROJECTS_CACHE.write().expect("unable to acquire the lock");
  let now = Utc::now().naive_utc();
  *projects_cache = Some((projects.clone(), now));
  Ok(projects)
}

async fn gitlab_get_project_merge_requests(id: i64) -> Result<Vec<MergeRequest>, anyhow::Error> {
  let mrs = {
    let projects_mr_cache = PROJECTS_MR_CACHE.read().expect("to read PROJECTS_MR_CACHE");
    projects_mr_cache
      .get(&id)
      .map(|(project, updated_at)| (!is_outdated(updated_at)).then_some(project.clone()))
      .unwrap_or(None)
  };
  if let Some(mrs) = mrs {
    Ok(mrs)
  } else {
    let request = REQWEST_CLIENT_GITLAB
      .get(GITLAB_PROJECT_MR_URL.replace("{}", &id.to_string()))
      .send()
      .await?;
    let mrs: Vec<MergeRequest> = request.json().await.unwrap();
    let now = Utc::now().naive_utc();
    let mut projects_mr_cache = PROJECTS_MR_CACHE
      .write()
      .expect("to write PROJECTS_MR_CACHE");
    projects_mr_cache.insert(id, (mrs.clone(), now));
    Ok(mrs)
  }
}

async fn display_preview(
  context: &Context,
  message: &Message,
  project: &Project,
  merge_request: Option<MergeRequest>,
) -> Result<Message, serenity::Error> {
  let url = REGEX_URL_PARSE
    .captures(&message.content)
    .unwrap()
    .get(0)
    .unwrap()
    .as_str();
  message
    .channel_id
    .send_message(context, |m| {
      m.add_embed(|e| {
        let embed = e
          .title(url)
          .url(url)
          .footer(|f| f.text("Gitlab Preview"))
          .color(Colour::ORANGE);
        if !project.description.is_empty() {
          embed.field("description", &project.description, false);
        }
        // We can't get the image and send it to discord because they are private
        // This is how the images was guess in previous version
        // if let Some(image) = &project.avatar_url {
        //   embed.image(format!("https://lab.blackfoot.io{}", image));
        // } else if let Some(image) = &project.namespace.avatar_url {
        //   embed.image(format!("https://lab.blackfoot.io{}", image));
        // }
        embed.image("https://lab.blackfoot.io/assets/gitlab_logo-7ae504fe4f68fdebb3c2034e36621930cd36ea87924c11ff65dbcb8ed50dca58.png");
        if let Some(merge_request) = merge_request {
          embed
            .title(&merge_request.title)
            .description(&merge_request.description)
            .field("author", &merge_request.author.username, false)
            .field("merge_status", &merge_request.merge_status, false);
        } else {
          embed.description(&project.path_with_namespace);
        }
        embed
      })
    })
    .await
}

async fn get_or_update_cache(
  project_to_find: &str,
  merge_id: &Option<i64>,
) -> Result<(Project, Option<MergeRequest>), anyhow::Error> {
  {
    let projects_cache = PROJECTS_CACHE
      .read()
      .expect("to read PROJECTS_CACHE")
      .clone();
    let should_update = projects_cache
      .as_ref()
      .map(|(_, updated_at)| is_outdated(updated_at))
      .unwrap_or(true);

    if !should_update {
      let (projects, _) = projects_cache.expect("PROJECTS_CACHE to be initialized");
      return find_project(projects, project_to_find, merge_id).await;
    };
  }
  let projects = {
    let new_projects = gitlab_get_projects().await?;
    let mut projects = PROJECTS_CACHE.write().expect("to write PROJECTS_CACHE");
    let now = Utc::now().naive_utc();
    *projects = Some((new_projects, now));
    projects.as_ref().unwrap().0.clone()
  };
  return find_project(projects, project_to_find, merge_id).await;
}

async fn find_project(
  projects: Vec<Project>,
  project_to_find: &str,
  merge_id: &Option<i64>,
) -> Result<(Project, Option<MergeRequest>), anyhow::Error> {
  let found_project_url = projects
    .iter()
    .find(|p| p.path_with_namespace.contains(project_to_find));

  if let Some(project) = found_project_url {
    let mr = if let Some(merge_id) = merge_id {
      let mrs = gitlab_get_project_merge_requests(project.id).await?;
      mrs.into_iter().find(|mr| &mr.iid == merge_id)
    } else {
      None
    };

    Ok((project.clone(), mr))
  } else {
    Err(anyhow::anyhow!("project wasn't found"))
  }
}

async fn preview_and_cleanup(
  message: &Message,
  context: &Context,
  project: Project,
  mr: Option<MergeRequest>,
) -> Result<(), anyhow::Error> {
  match display_preview(context, message, &project, mr).await {
    Ok(_) => {
      if REGEX_URL_PARSE_ONLY.is_match(&message.content) {
        if let Err(err) = message.delete(context).await {
          error!("deleting message previewed failed: {}", err);
        }
      }
    }
    Err(err) => error!("Error building gitlab prewiew failed: {}", err),
  }
  Ok(())
}

pub async fn gitlab_url_preview(message: &Message, context: &Context) -> Result<(), anyhow::Error> {
  if let Some((url, merge_id)) = check_message_should_preview(&message.content) {
    let (project, mr) = get_or_update_cache(&url, &merge_id).await?;
    preview_and_cleanup(message, context, project, mr).await?;
  }
  Ok(())
}

#[test]
fn test_check_message_should_preview() {
  let r =
    check_message_should_preview("https://lab.blackfoot.io/fdj/api/-/merge_requests/105/diffs");
  assert_eq!(r, Some((String::from("fdj/api"), Some(105))));
}

#[tokio::test]
async fn test_gitlab_mr_preview() {
  dotenv::dotenv().ok();
  // https://lab.blackfoot.io/api/v4/projects/279/merge_requests/98
  // gitlab_url_preview("https://lab.blackfoot.io/api/v4/projects/")
  //   .await
  //   .unwrap();
}
