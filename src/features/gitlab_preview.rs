use std::{
  collections::HashMap,
  sync::{Arc, RwLock},
};

use chrono::{Duration, NaiveDateTime, Utc};
use regex::Regex;
use reqwest::{header, Client, Error};
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
    Regex::new(r#"https://lab\.blackfoot\.io/(([A-z-/]+)/-[A-z-/1-9]+$|([A-z-/1-9]+$))"#).unwrap();
  static ref REGEX_URL_PARSE_START: Regex =
    Regex::new(r#"^https://lab\.blackfoot\.io/(([A-z-/]+)/-[A-z-/1-9]+$|([A-z-/1-9]+$))"#).unwrap();
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

async fn gitlab_get_projects() -> Result<Vec<Project>, Error> {
  let request = REQWEST_CLIENT_GITLAB.get(GITLAB_PROJECT_URL).send().await?;
  let projects: Vec<Project> = request.json().await.unwrap();

  let mut projects_cache = PROJECTS_CACHE.write().expect("unable to acquire the lock");
  let now = Utc::now().naive_utc();
  *projects_cache = Some((projects.clone(), now));
  Ok(projects)
}

async fn gitlab_get_project_merge_requests(id: i64) -> Result<Vec<MergeRequest>, Error> {
  let request = REQWEST_CLIENT_GITLAB
    .get(GITLAB_PROJECT_MR_URL.replace("{}", &id.to_string()))
    .send()
    .await?;
  let mrs: Vec<MergeRequest> = request.json().await.unwrap();

  let mut projects_mr_cache = PROJECTS_MR_CACHE
    .write()
    .expect("unable to acquire the lock");
  let now = Utc::now().naive_utc();
  projects_mr_cache.insert(id, (mrs.clone(), now));
  Ok(mrs)
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
          .title(&url)
          .url(&url)
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
        embed.image("https://assets.stickpng.com/images/5847f997cef1014c0b5e48c1.png");
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

pub async fn gitlab_url_preview(message: &Message, context: &Context) -> Result<(), Error> {
  let (project_to_find, merge_id) =
    if let Some((url, merge_id)) = check_message_should_preview(&message.content) {
      (url, merge_id)
    } else {
      return Ok(());
    };

  let should_update;
  let mut projects = None;
  {
    let projects_cache = PROJECTS_CACHE.read().expect("unable to acquire the lock");
    should_update = if let Some((cache_projects, updated_at)) = projects_cache.as_ref() {
      let now = Utc::now().naive_utc();
      if *updated_at + Duration::hours(1) > now {
        projects = Some(cache_projects.clone());
        false
      } else {
        true
      }
    } else {
      true
    };
  }
  if should_update {
    projects = Some(gitlab_get_projects().await?);
  };

  let projects = projects.expect("projects should have been initialized");
  let found_project_url = projects
    .iter()
    .find(|p| p.path_with_namespace.contains(&project_to_find));

  if let Some(project) = found_project_url {
    let mut merge_request = None;
    let should_update;
    if let Some(merge_id) = merge_id {
      {
        let mrs = PROJECTS_MR_CACHE
          .read()
          .expect("unable to acquire the lock");
        should_update = if let Some((_, updated_at)) = mrs.get(&project.id) {
          let now = Utc::now().naive_utc();
          *updated_at + Duration::hours(1) <= now
        } else {
          true
        };
      }
      let mut merge_requests = if should_update {
        gitlab_get_project_merge_requests(project.id)
          .await?
          .into_iter()
      } else {
        let mrs = PROJECTS_MR_CACHE
          .read()
          .expect("unable to acquire the lock");
        mrs.get(&project.id).unwrap().0.clone().into_iter()
      };
      merge_request = merge_requests.find(|mr| mr.iid == merge_id);
    }
    if display_preview(context, message, project, merge_request)
      .await
      .is_ok()
      && REGEX_URL_PARSE_START.is_match(&message.content)
    {
      if let Err(err) = message.delete(context).await {
        error!("deleting message previewed failed: {}", err);
      }
    }
  };
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
