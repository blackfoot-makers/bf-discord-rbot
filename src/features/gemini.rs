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
  _category: String,
  _probability: String,
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
  let input = r#"[
    {
      "candidates": [
        {
          "content": {
            "role": "model",
            "parts": [
              {
                "text": "Baptiste earned the nickname \"Potato\" during his early days as a member of the"
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
                "text": " Talon organization. During a mission, Baptiste and his team were tasked with infiltrating"
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
                "text": " a heavily guarded facility. As they made their way through the complex, they encountered a group of guards who were armed with powerful weapons.\n\nIn the ensuing fire"
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
                "text": "fight, Baptiste's teammates were quickly overwhelmed and taken down. Baptiste, however, managed to hold his own, using his agility and combat skills to evade the"
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
                "text": " enemy fire. As the guards closed in on him, Baptiste realized that he needed to find a way to escape.\n\nSpotting a pile of potatoes nearby, Baptiste had an idea. He quickly grabbed a handful of potatoes and threw them at"
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
                "text": " the guards, blinding them momentarily. This gave Baptiste the opportunity to make his escape, and he managed to slip away without being seen.\n\nAfter the mission, Baptiste's teammates couldn't help but laugh at the story of how he"
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
                "text": " had used potatoes to escape. They started calling him \"Potato\" as a joke, and the nickname stuck.\n\nBaptiste initially disliked the nickname, but over time he came to embrace it. He realized that the nickname was a reminder of his resourcefulness and his ability to think on his feet. He also liked the fact"
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
                "text": " that it made him stand out from the other members of Talon.\n\nTo this day, Baptiste is still known as \"Potato\" by his teammates and associates. The nickname is a testament to his unique skills and his ability to overcome any challenge that comes his way."
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
        "promptTokenCount": 16,
        "candidatesTokenCount": 309,
        "totalTokenCount": 325
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
