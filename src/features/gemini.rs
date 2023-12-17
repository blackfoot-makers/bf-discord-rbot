use tokio::sync::OnceCell;

use procedural_macros::command;
use yup_oauth2::{
  authenticator::Authenticator, hyper::client, hyper_rustls, ServiceAccountAuthenticator,
};

use crate::core::commands::{CallBackParams, CallbackReturn};

type SA = Authenticator<hyper_rustls::HttpsConnector<client::HttpConnector>>;

static SERVICE_ACCOUNT: OnceCell<SA> = OnceCell::const_new();
const SCOPES: [&str; 1] = ["https://www.googleapis.com/auth/cloud-platform"];

async fn get_token() -> SA {
  let key_path = std::env::var("GOOGLE_APPLICATION_CREDENTIALS")
    .unwrap_or_else(|_| "blackfoot-dev-bd1f97a0d61e.json".to_string());
  let creds = yup_oauth2::read_service_account_key(key_path)
    .await
    .unwrap();
  ServiceAccountAuthenticator::builder(creds)
    .build()
    .await
    .unwrap()
}

#[derive(Debug, Serialize, Deserialize)]
struct Part {
  text: String,
}
#[derive(Debug, Serialize, Deserialize)]
struct Content {
  parts: Vec<Part>,
  role: String,
}
#[derive(Debug, Serialize)]
struct GenerationConfig {
  #[serde(rename(serialize = "maxOutputTokens"))]
  max_output_tokens: i64,
  temperature: f64,
  #[serde(rename(serialize = "topK"))]
  top_k: i64,
  #[serde(rename(serialize = "topP"))]
  top_p: i64,
}
#[derive(Debug, Serialize)]
struct GeminiBody {
  contents: Vec<Content>,
  generation_config: GenerationConfig,
}

impl GeminiBody {
  fn new(input: &str) -> Self {
    Self {
      contents: vec![Content {
        role: "user".to_string(),
        parts: vec![Part {
          text: input.to_string(),
        }],
      }],
      generation_config: GenerationConfig {
        // This is to avoid multiple discord messages
        max_output_tokens: 1900,
        temperature: 0.4,
        top_p: 1,
        top_k: 32,
      },
    }
  }
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
struct SafetyRatings {
  category: String,
  probability: String,
}

#[derive(Debug, Deserialize)]
struct UsageMetadata {
  #[serde(rename(deserialize = "candidatesTokenCount"))]
  _candidates_token_count: i64,
  #[serde(rename(deserialize = "promptTokenCount"))]
  _prompt_token_count: i64,
  #[serde(rename(deserialize = "totalTokenCount"))]
  _total_token_count: i64,
}

#[derive(Debug, Deserialize)]
struct Candidate {
  content: Content,
  #[serde(rename(deserialize = "finishReason"))]
  _finish_reason: Option<String>,
  #[serde(rename(deserialize = "safetyRatings"))]
  _safety_ratings: Vec<SafetyRatings>,
}

#[derive(Debug, Deserialize)]
struct GeminiResult {
  candidates: Vec<Candidate>,
  #[serde(rename(deserialize = "usageMetadata"))]
  _usage_metadata: Option<UsageMetadata>,
}

async fn query_gemini(question: &str) -> Result<Vec<GeminiResult>, &str> {
  const API_ENDPOINT: &str = "us-east4-aiplatform.googleapis.com";
  const PROJECT_ID: &str = "blackfoot-dev";
  const MODEL_ID: &str = "gemini-pro";
  const LOCATION_ID: &str = "us-east4";

  let url = format!("https://{API_ENDPOINT}/v1beta1/projects/{PROJECT_ID}/locations/{LOCATION_ID}/publishers/google/models/{MODEL_ID}:streamGenerateContent");
  let body = GeminiBody::new(question);
  let client = reqwest::Client::new();
  let token = SERVICE_ACCOUNT
    .get_or_init(get_token)
    .await
    .token(&SCOPES)
    .await
    .expect("service account token to call google services");
  let token = token.token().unwrap();
  let response = client
    .post(url)
    .bearer_auth(token)
    .header("Content-Type", "application/json")
    .json(&body)
    .send()
    .await
    .unwrap();

  let res_text = response.text().await.unwrap();
  let Ok(res) = serde_json::from_str(&res_text) else {
    eprintln!("unable to deserialize gemini response: \n{res_text}");
    return Err("failed to query gemini");
  };
  Ok(res)
}

#[command]
pub async fn question(params: CallBackParams) -> CallbackReturn {
  let question = params.args[1..].join(" ");
  let res = query_gemini(&question).await;
  let response_text = res?
    .into_iter()
    .flat_map(|r| {
      r.candidates
        .into_iter()
        .flat_map(|c| c.content.parts.into_iter().map(|p| p.text))
    })
    .collect();
  Ok(Some(response_text))
}

#[tokio::test]
async fn test_call_gemini() {
  let res =
    query_gemini("Can you tell  the story of why Baptiste is nicknamed \"potatoe aim\" ?").await;
  let response_text: String = res
    .unwrap()
    .into_iter()
    .flat_map(|r| {
      r.candidates
        .into_iter()
        .flat_map(|c| c.content.parts.into_iter().map(|p| p.text))
    })
    .collect();
  println!("{:#?}", response_text);
}

#[test]
fn test_gemini_deserialize() {
  use std::fs;

  for test_files in ["gemini-1.json", "gemini-2.json"] {
    let input = fs::read_to_string(format!("fixtures/tests/{test_files}")).unwrap();

    let deserialized: Vec<GeminiResult> = serde_json::from_str(&input).unwrap();
    let response_text: String = deserialized
      .into_iter()
      .flat_map(|r| {
        r.candidates
          .into_iter()
          .flat_map(|c| c.content.parts.into_iter().map(|p| p.text))
      })
      .collect();
    println!("{response_text}");
  }
}
