#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
use rocket::serde::{json::Json, Deserialize, Serialize};
use rocket::http::Status;
use std::collections::HashMap;
use std::fs::{OpenOptions};
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize)]
struct Config {
    basedir: String,
    clients: HashMap<String, ClientConfig>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ClientConfig {
    id: String,
    log_path: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(crate = "rocket::serde")]
struct Request {
    client_id: String,
    message: String,
}

fn read_config() -> Result<Config, Box<dyn std::error::Error>> {
    // Construct the absolute path to config.json
    let cwd = std::env::current_dir()?;
    let config_path = cwd.join("config.json");
    
    // Print the absolute path for validation - commented out
    //println!("Attempting to read config from: {}", config_path.display());

    let config_content = std::fs::read_to_string(&config_path)?;
    
    // Print the contents of config.json for validation - commented out
    //println!("Contents of config.json:\n{}", config_content);

    let config: Config = serde_json::from_str(&config_content)?;
    Ok(config)
}

async fn log_message(message: &str, log_path: &PathBuf) -> Result<(), std::io::Error> {
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(log_path)?;

    writeln!(file, "{}", message)
}

#[post("/", format = "json", data = "<request>")]
async fn index(request: Json<Request>) -> Result<Json<&'static str>, (Status, String)> {
    // Retrieve and print the current working directory  - commented out
    
    let cwd = std::env::current_dir().expect("Failed to retrieve current working directory");
    //println!("Current working directory: {:?}", cwd);

    // Print a list of files in the CWD - commented out
    //println!("Files in the current directory:");
    match std::fs::read_dir(&cwd) {
        Ok(entries) => {
            for entry in entries {
                if let Ok(entry) = entry {
                    println!("- {}", entry.path().display());
                }
            }
        },
        Err(e) => println!("Failed to list files in the current directory: {}", e),
    }

    let config = match read_config() {
        Ok(config) => config,
        Err(_) => return Err((Status::InternalServerError, "Failed to read config".into())),
    };

    let client_config = match config.clients.get(&request.client_id) {
        Some(client) => client,
        None => return Err((Status::BadRequest, "Invalid client ID".into())),
    };

    let log_path = PathBuf::from(&config.basedir)
        .join("clients")
        .join(&client_config.id)
        .join(&client_config.log_path);

    match log_message(&request.message, &log_path).await {
        Ok(_) => Ok(Json("Logged")),
        Err(_) => Err((Status::InternalServerError, "Failed to log message".into())),
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index])
}
