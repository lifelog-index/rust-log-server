use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};  // Import Mutex and Arc
use chrono::Utc;
use serde::{Deserialize, Serialize};
use warp::Filter;
use dotenv::dotenv;

#[derive(Debug, Serialize, Deserialize)]
struct LogEntry {
    timestamp: String,
    system_name: String,
    username: String,
    interaction_type: String,
    value: serde_json::Value,
    is_file: Option<bool>,
}

const MAX_LOG_LINES: usize = 1000;
// const MAX_LOG_LINES: usize = 10_000;
const MAX_ARTIFACT_SIZE: usize = 5 * 1024 * 1024; // 5MB

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("PORT").unwrap_or_else(|_| "3030".to_string()).parse().unwrap_or(3030);

    // Create a Mutex-wrapped shared state for the log file
    let log_mutex = Arc::new(Mutex::new(()));

    let log_route = warp::post()
        .and(warp::path("log"))
        .and(warp::body::json())
        .and(warp::any().map(move || Arc::clone(&log_mutex))) // Share the Mutex across routes
        .and_then(handle_log);

    println!("Log server running on http://{}:{}", host, port);
    warp::serve(log_route).run((host.parse::<std::net::IpAddr>().unwrap(), port)).await;
}

async fn handle_log(entry: LogEntry, log_mutex: Arc<Mutex<()>>) -> Result<impl warp::Reply, warp::Rejection> {
    let timestamp = Utc::now().to_rfc3339();
    let mut log_entry = LogEntry {
        timestamp,
        is_file: Some(false),
        ..entry
    };

    let base_path = format!("{}/{}/", log_entry.system_name, log_entry.username);
    let log_dir = Path::new(&base_path);
    let artifact_dir = log_dir.join("artifacts");
    fs::create_dir_all(&artifact_dir).unwrap();

    if let Some(value_str) = log_entry.value.as_str() {
        if value_str.len() > MAX_ARTIFACT_SIZE {
            let interaction_type = log_entry.interaction_type.clone();

            let artifact_path = artifact_dir.join(format!("artifact_{}_{}.txt", interaction_type, Utc::now().timestamp()));
            fs::write(&artifact_path, value_str).unwrap();
            log_entry.value = serde_json::json!(artifact_path.to_string_lossy());
            log_entry.is_file = Some(true);
        }
    } else if log_entry.value.is_object() {
        let serialized = serde_json::to_string(&log_entry.value).unwrap();
        let interaction_type = log_entry.interaction_type.clone();
        if serialized.len() > MAX_ARTIFACT_SIZE {
            let artifact_path = artifact_dir.join(format!("artifact_{}_{}.json", interaction_type, Utc::now().timestamp()));
            fs::write(&artifact_path, serialized).unwrap();
            log_entry.value = serde_json::json!(artifact_path.to_string_lossy());
            log_entry.is_file = Some(true);
        }
    }

    // Lock the Mutex to ensure safe access to the log file
    let _lock = log_mutex.lock().unwrap();

    let log_file_path = rotate_log_file(&log_dir).unwrap();
    let mut log_file = OpenOptions::new().append(true).open(&log_file_path).unwrap();
    let serialized_log = serde_json::to_string(&log_entry).unwrap();
    writeln!(log_file, "{}", serialized_log).unwrap();

    Ok(warp::reply::json(&serde_json::json!({
        "status": "success",
        "message": "Log recorded."
    })))
}

fn rotate_log_file(log_dir: &Path) -> Result<PathBuf, std::io::Error> {
    let mut log_file_path = log_dir.join("log.log");
    let mut file_index = 0;

    while log_file_path.exists() {
        let line_count = count_lines(&log_file_path)?;
        if line_count < MAX_LOG_LINES {
            break;
        }
        file_index += 1;
        log_file_path = log_dir.join(format!("log_{}.log", file_index));
    }

    if !log_file_path.exists() {
        fs::create_dir_all(log_dir)?;
        File::create(&log_file_path)?;
    }

    Ok(log_file_path)
}

fn count_lines(file_path: &Path) -> Result<usize, std::io::Error> {
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    Ok(reader.lines().count())
}
