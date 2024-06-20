use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
enum Role {
    System,
    User,
    Assistant,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Message {
    role: Role,
    content: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Choice {
    message: Message,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ApiResponse {
    choices: Vec<Choice>,
    id: String,
    object: String,
    created: i64,
    model: String,
    usage: ApiUsage,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct ApiUsage {
    prompt_tokens: i32,
    completion_tokens: i32,
    total_tokens: i32,
}

fn read_text_data(filename: &str) -> String {
    fs::read_to_string(filename).expect("Failed to read the file")
}

fn update_json_data(response: &ApiResponse) {
    let existing_data: Vec<ApiResponse> = fs::read_to_string("response.json")
        .map(|content| serde_json::from_str(&content).unwrap_or_else(|_| vec![]))
        .unwrap_or_else(|_| vec![]);

    let mut updated_data = existing_data.clone();
    updated_data.push(response.clone());

    fs::write(
        "response.json",
        serde_json::to_string_pretty(&updated_data).unwrap(),
    )
    .expect("Unable to write to file");
}

fn generate_response(article: &str, user_content: &str) -> Result<ApiResponse, reqwest::Error> {
    let api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not found");
    let client = Client::new();

    let messages = vec![
        Message {
            role: Role::System,
            content: "you are a useful assistant".to_string(),
        },
        Message {
            role: Role::User,
            content: format!("Based on this article: {}, {}", article, user_content),
        },
    ];

    let request_payload = { messages };

    let response = client
        .post("https://api.openai.com/v1/engines/gpt-3.5-turbo-0613/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request_payload)
        .send()?
        .json::<ApiResponse>()
        .map_err(|e| {
            eprintln!("Failed to deserialize: {}", e);
            e
        })?;

    Ok(response)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let mut filename = String::new();
    println!("Enter the name of the file to read content: ");
    io::stdin().read_line(&mut filename)?;
    let article = read_text_data(filename.trim());

    loop {
        let mut user_query = String::new();
        println!("Enter your query here for GPT to answer: ");
        io::stdin().read_line(&mut user_query)?;

        match generate_response(&article, user_query.trim()) {
            Ok(response) => {
                update_json_data(&response);
                println!("{:#?}", response);
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }

        let mut count = String::new();
        println!("0 for continue, any other number for exit: ");
        io::stdin().read_line(&mut count)?;
        if count.trim() != "0" {
            break;
        }
    }

    println!("Thank you!");
    Ok(())
}

use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::io;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
enum Role {
    System,
    User,
    Assistant,
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    role: Role,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct FunctionParams {
    type_: String,
    properties: FunctionProperties,
}

#[derive(Debug, Serialize, Deserialize)]
struct FunctionProperties {
    article_text: FunctionArticleText,
}

#[derive(Debug, Serialize, Deserialize)]
struct FunctionArticleText {
    type_: String,
    description: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Function {
    name: String,
    description: String,
    parameters: FunctionParams,
}

#[derive(Debug, Serialize, Deserialize)]
struct MessageDetails {
    role: Role,
    content: String,
    function_call: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct MotorheadChoice {
    message: MessageDetails,
}

#[derive(Debug, Serialize, Deserialize)]
struct MotorheadResponse {
    choices: Vec<MotorheadChoice>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ApiUsage {
    prompt_tokens: i32,
    completion_tokens: i32,
    total_tokens: i32,
}

#[derive(Debug, Serialize, Deserialize)]
struct ApiResponse {
    data: MotorheadResponse,
    id: String,
    object: String,
    created: i64,
    model: String,
    usage: ApiUsage,
}

#[derive(Debug, Serialize, Deserialize)]
struct RequestPayload {
    messages: Vec<Message>,
    functions: Vec<Function>,
}

fn read_text_data(filename: &str) -> String {
    fs::read_to_string(filename).expect("Failed to read the file")
}

fn update_json_data(response: &Value) {
    let existing_data: Vec<Value> = fs::read_to_string("response.json")
        .map(|content| serde_json::from_str(&content).unwrap_or_else(|_| vec![]))
        .unwrap_or_else(|_| vec![]);

    let mut updated_data = existing_data.clone();
    updated_data.push(response.clone());

    fs::write(
        "response.json",
        serde_json::to_string_pretty(&updated_data).unwrap(),
    )
    .expect("Unable to write to file");
}

fn generate_response(
    article: &str,
    user_content: &str,
    previous_messages: Option<Vec<Message>>,
) -> Result<ApiResponse, reqwest::Error> {
    let api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not found");

    let client = Client::new();

    let mut messages = match previous_messages {
        Some(prev_msgs) => prev_msgs,
        None => vec![
            Message {
                role: Role::System,
                content: "you are a useful assistant".to_string(),
            },
            Message {
                role: Role::User,
                content: format!("here is an article input: {}, please return information based on the article when asked", article),
            },
        ],
    };

    messages.push(Message {
        role: Role::User,
        content: user_content.to_string(),
    });

    let functions = vec![Function {
        name: "get_article_summary".to_string(),
        description: "Returns a summary of the article text".to_string(),
        parameters: FunctionParams {
            type_: "object".to_string(),
            properties: FunctionProperties {
                article_text: FunctionArticleText {
                    type_: "string".to_string(),
                    description: "Content of the article".to_string(),
                },
            },
        },
    }];

    let request_payload = RequestPayload {
        messages: messages,
        functions: functions,
    };

    let response = client
        .post("https://api.openai.com/v1/engines/gpt-3.5-turbo-0613/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request_payload)
        .send()?
        .json::<ApiResponse>()
        .map_err(|e| {
            eprintln!("Failed to deserialize: {}", e);
            e
        })?;

    Ok(response)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let mut filename = String::new();
    println!("Enter the name of the file to read content: ");
    io::stdin().read_line(&mut filename)?;
    filename = filename.trim().to_string();
    let article = read_text_data(&filename);

    loop {
        let mut user_query = String::new();
        println!("Enter your query here for GPT to answer: ");
        io::stdin().read_line(&mut user_query)?;
        user_query = user_query.trim().to_string();

        match generate_response(&article, &user_query, None) {
            Ok(response) => {
                update_json_data(&serde_json::to_value(&response.data).unwrap());
                println!("{:#?}", response.data);
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }

        let mut count = String::new();
        println!("0 for continue, any other number for exit: ");
        io::stdin().read_line(&mut count)?;
        if count.trim() != "0" {
            break;
        }
    }

    println!("Thank you!");
    Ok(())
}
