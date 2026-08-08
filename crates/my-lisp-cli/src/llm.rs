use serde::{Deserialize, Serialize};
use std::env;

#[derive(Serialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Deserialize)]
struct ResponseMessage {
    content: String,
}

pub fn call_llm(system_prompt: &str, user_text: &str) -> Result<String, String> {
    let api_key = env::var("OPENAI_API_KEY")
        .map_err(|_| "OPENAI_API_KEY environment variable not set".to_string())?;

    let client = reqwest::blocking::Client::new();
    let req_body = ChatRequest {
        model: "gpt-4o".to_string(), // or any capable model
        messages: vec![
            Message {
                role: "system".to_string(),
                content: system_prompt.to_string(),
            },
            Message {
                role: "user".to_string(),
                content: user_text.to_string(),
            },
        ],
        temperature: 0.0, // We want deterministic syntax
    };

    let res = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&req_body)
        .send()
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    if !res.status().is_success() {
        return Err(format!("API error: {}", res.status()));
    }

    let chat_res: ChatResponse = res
        .json()
        .map_err(|e| format!("Failed to parse JSON response: {}", e))?;

    let content = chat_res
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .unwrap_or_default();

    // Remove markdown code blocks if the LLM adds them
    let content = content.replace("```lisp\n", "");
    let content = content.replace("```\n", "");
    let content = content.replace("```", "");
    
    Ok(content.trim().to_string())
}

pub fn generate_rule(text: &str) -> Result<String, String> {
    let system = "You are a Lisp symbolic knowledge compiler. \
                  Translate the user's natural language statement into a list of my-lisp rules. \
                  A rule is a list where the first element is the head, and the rest are body conditions. \
                  A fact has no body. \
                  Use `(var x)` for logic variables. \
                  Output ONLY valid Lisp s-expressions. No markdown, no explanations. \
                  Example input: All men are mortal. Socrates is a man. \
                  Example output: ((mortal (var x)) (man (var x))) ((man socrates))";
    call_llm(system, text)
}

pub fn generate_query(text: &str) -> Result<String, String> {
    let system = "You are a Lisp symbolic query compiler. \
                  Translate the user's natural language question into a SINGLE my-lisp goal expression. \
                  Output ONLY a valid Lisp s-expression. No markdown, no explanations. \
                  Example input: Is Socrates mortal? \
                  Example output: (mortal socrates)";
    call_llm(system, text)
}
