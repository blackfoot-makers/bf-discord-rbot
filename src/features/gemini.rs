use tokio::sync::OnceCell;

use procedural_macros::command;
use yup_oauth2::{AccessToken, ServiceAccountAuthenticator};

use crate::core::commands::{CallBackParams, CallbackReturn};

static TOKEN: OnceCell<AccessToken> = OnceCell::const_new();

async fn get_token() -> AccessToken {
  let creds = yup_oauth2::read_service_account_key("blackfoot-dev-bd1f97a0d61e.json")
    .await
    .unwrap();
  let sa = ServiceAccountAuthenticator::builder(creds)
    .build()
    .await
    .unwrap();
  let scopes = &["https://www.googleapis.com/auth/cloud-platform"];

  sa.token(scopes).await.unwrap()
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
        max_output_tokens: 2048,
        temperature: 0.4,
        top_p: 1,
        top_k: 32,
      },
    }
  }
}

#[derive(Debug, Deserialize)]
struct SafetyRatings {
  category: String,
  probability: String,
}

#[derive(Debug, Deserialize)]
struct UsageMetadata {
  #[serde(rename(deserialize = "candidatesTokenCount"))]
  candidates_token_count: i64,
  #[serde(rename(deserialize = "promptTokenCount"))]
  prompt_token_count: i64,
  #[serde(rename(deserialize = "totalTokenCount"))]
  total_token_count: i64,
}

#[derive(Debug, Deserialize)]
struct Candidate {
  content: Content,
  #[serde(rename(deserialize = "finishReason"))]
  finish_reason: Option<String>,
  #[serde(rename(deserialize = "safetyRatings"))]
  safety_ratings: Vec<SafetyRatings>,
}

#[derive(Debug, Deserialize)]
struct GeminiResult {
  candidates: Vec<Candidate>,
  #[serde(rename(deserialize = "usageMetadata"))]
  usage_metadata: Option<UsageMetadata>,
}

async fn query_gemini(question: &str) -> Vec<GeminiResult> {
  const API_ENDPOINT: &str = "us-east4-aiplatform.googleapis.com";
  const PROJECT_ID: &str = "blackfoot-dev";
  const MODEL_ID: &str = "gemini-pro-vision";
  const LOCATION_ID: &str = "us-east4";

  let url = format!("https://{API_ENDPOINT}/v1beta1/projects/{PROJECT_ID}/locations/{LOCATION_ID}/publishers/google/models/{MODEL_ID}:streamGenerateContent");
  let body = GeminiBody::new(question);
  let client = reqwest::Client::new();
  let response = client
    .post(url)
    .bearer_auth(TOKEN.get_or_init(get_token).await.token().unwrap())
    .header("Content-Type", "application/json")
    .json(&body)
    .send()
    .await
    .unwrap();
  response.json().await.unwrap()
}

#[command]
pub async fn question(params: CallBackParams) -> CallbackReturn {
  let question = params.args[1..].join(" ");
  let res = query_gemini(&question).await;
  let response_text = res
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
  let res = query_gemini("what is the result of 2 + 3 * 29").await;
  let response_text: String = res
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
  let input = r#"[
  {
    "candidates": [
      {
        "content": {
          "role": "model",
          "parts": [
            {
              "text": "Sure, here is a test:\n\n**Question 1:**\n\nWhat is"
            }
          ]
        },
        "safetyRatings": [
          {
            "category": "HARM_CATEGORY_HARASSMENT",
            "probability": "NEGLIGIBLE"
          },
          {
            "category": "HARM_CATEGORY_HATE_SPEECH",
            "probability": "NEGLIGIBLE"
          },
          {
            "category": "HARM_CATEGORY_SEXUALLY_EXPLICIT",
            "probability": "NEGLIGIBLE"
          },
          {
            "category": "HARM_CATEGORY_DANGEROUS_CONTENT",
            "probability": "NEGLIGIBLE"
          }
        ]
      }
    ]
  },
  {
    "candidates": [
      {
        "content": {
          "role": "model",
          "parts": [
            {
              "text": " the capital of France?\n\nA. London\nB. Paris\nC."
            }
          ]
        },
        "safetyRatings": [
          {
            "category": "HARM_CATEGORY_HARASSMENT",
            "probability": "NEGLIGIBLE"
          },
          {
            "category": "HARM_CATEGORY_HATE_SPEECH",
            "probability": "NEGLIGIBLE"
          },
          {
            "category": "HARM_CATEGORY_SEXUALLY_EXPLICIT",
            "probability": "NEGLIGIBLE"
          },
          {
            "category": "HARM_CATEGORY_DANGEROUS_CONTENT",
            "probability": "NEGLIGIBLE"
          }
        ]
      }
    ]
  },
  {
    "candidates": [
      {
        "content": {
          "role": "model",
          "parts": [
            {
              "text": " Rome\nD. Berlin\n\n**Question 2:**\n\nWhat is the largest ocean in the world?\n\nA. Pacific Ocean\nB. Atlantic Ocean\n"
            }
          ]
        },
        "safetyRatings": [
          {
            "category": "HARM_CATEGORY_HARASSMENT",
            "probability": "NEGLIGIBLE"
          },
          {
            "category": "HARM_CATEGORY_HATE_SPEECH",
            "probability": "NEGLIGIBLE"
          },
          {
            "category": "HARM_CATEGORY_SEXUALLY_EXPLICIT",
            "probability": "NEGLIGIBLE"
          },
          {
            "category": "HARM_CATEGORY_DANGEROUS_CONTENT",
            "probability": "NEGLIGIBLE"
          }
        ]
      }
    ]
  },
  {
    "candidates": [
      {
        "content": {
          "role": "model",
          "parts": [
            {
              "text": "C. Indian Ocean\nD. Arctic Ocean\n\n**Question 3:**\n\nWhat is the most common element in the universe?\n\nA. Hydrogen\nB"
            }
          ]
        },
        "safetyRatings": [
          {
            "category": "HARM_CATEGORY_HARASSMENT",
            "probability": "NEGLIGIBLE"
          },
          {
            "category": "HARM_CATEGORY_HATE_SPEECH",
            "probability": "NEGLIGIBLE"
          },
          {
            "category": "HARM_CATEGORY_SEXUALLY_EXPLICIT",
            "probability": "NEGLIGIBLE"
          },
          {
            "category": "HARM_CATEGORY_DANGEROUS_CONTENT",
            "probability": "NEGLIGIBLE"
          }
        ]
      }
    ]
  },
  {
    "candidates": [
      {
        "content": {
          "role": "model",
          "parts": [
            {
              "text": ". Helium\nC. Carbon\nD. Oxygen\n\n**Question 4:**\n\nWhat is the name of the largest planet in our solar system?\n\nA. Jupiter\nB. Saturn\nC. Uranus\nD. Neptune\n\n**"
            }
          ]
        },
        "safetyRatings": [
          {
            "category": "HARM_CATEGORY_HARASSMENT",
            "probability": "NEGLIGIBLE"
          },
          {
            "category": "HARM_CATEGORY_HATE_SPEECH",
            "probability": "NEGLIGIBLE"
          },
          {
            "category": "HARM_CATEGORY_SEXUALLY_EXPLICIT",
            "probability": "NEGLIGIBLE"
          },
          {
            "category": "HARM_CATEGORY_DANGEROUS_CONTENT",
            "probability": "NEGLIGIBLE"
          }
        ]
      }
    ]
  },
  {
    "candidates": [
      {
        "content": {
          "role": "model",
          "parts": [
            {
              "text": "Question 5:**\n\nWhat is the name of the star at the center of our solar system?\n\nA. Sun\nB. Moon\nC. Mars\nD. Venus\n\n**Answers:**\n\n1. B. Paris\n2"
            }
          ]
        },
        "safetyRatings": [
          {
            "category": "HARM_CATEGORY_HARASSMENT",
            "probability": "NEGLIGIBLE"
          },
          {
            "category": "HARM_CATEGORY_HATE_SPEECH",
            "probability": "NEGLIGIBLE"
          },
          {
            "category": "HARM_CATEGORY_SEXUALLY_EXPLICIT",
            "probability": "NEGLIGIBLE"
          },
          {
            "category": "HARM_CATEGORY_DANGEROUS_CONTENT",
            "probability": "NEGLIGIBLE"
          }
        ]
      }
    ]
  },
  {
    "candidates": [
      {
        "content": {
          "role": "model",
          "parts": [
            {
              "text": ". A. Pacific Ocean\n3. A. Hydrogen\n4. A. Jupiter\n5. A. Sun\n\nI hope you enjoyed this test!"
            }
          ]
        },
        "finishReason": "STOP",
        "safetyRatings": [
          {
            "category": "HARM_CATEGORY_HARASSMENT",
            "probability": "NEGLIGIBLE"
          },
          {
            "category": "HARM_CATEGORY_HATE_SPEECH",
            "probability": "NEGLIGIBLE"
          },
          {
            "category": "HARM_CATEGORY_SEXUALLY_EXPLICIT",
            "probability": "NEGLIGIBLE"
          },
          {
            "category": "HARM_CATEGORY_DANGEROUS_CONTENT",
            "probability": "NEGLIGIBLE"
          }
        ]
      }
    ],
    "usageMetadata": {
      "promptTokenCount": 1,
      "candidatesTokenCount": 223,
      "totalTokenCount": 224
    }
  }
]"#;

  let deserialized: Vec<GeminiResult> = serde_json::from_str(input).unwrap();
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
