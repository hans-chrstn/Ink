use std::collections::HashMap;
use std::process::Command;
use std::thread;

pub fn exec(cmd: &str) -> Result<String, String> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(format!("Error: {}", err));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn spawn(cmd: String) {
    thread::spawn(move || {
        let mut command = Command::new("sh");
        command.arg("-c").arg(cmd);

        match command.spawn() {
            Ok(mut child) => {
                let _ = child.wait();
            }
            Err(e) => {
                eprintln!("Failed to spawn command: {}", e);
            }
        }
    });
}

pub fn fetch(
    method: &str,
    uri: &str,
    headers: Option<HashMap<String, String>>,
    body: Option<String>,
) -> Result<String, String> {
    let client = reqwest::blocking::Client::new();
    let mut request = match method.to_lowercase().as_str() {
        "post" => client.post(uri),
        "put" => client.put(uri),
        "delete" => client.delete(uri),
        _ => client.get(uri),
    };

    if let Some(h) = headers {
        for (key, val) in h {
            request = request.header(&key, &val);
        }
    }

    if let Some(b) = body {
        request = request.body(b);
    }

    request
        .send()
        .map_err(|e| e.to_string())?
        .text()
        .map_err(|e| e.to_string())
}

pub async fn exec_async(cmd: String) -> Result<String, String> {
    let output = tokio::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .await
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(format!("Error: {}", err));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub async fn fetch_async(
    method: String,
    uri: String,
    headers: Option<HashMap<String, String>>,
    body: Option<String>,
) -> Result<String, String> {
    let client = reqwest::Client::new();
    let mut request = match method.to_lowercase().as_str() {
        "post" => client.post(&uri),
        "put" => client.put(&uri),
        "delete" => client.delete(&uri),
        _ => client.get(&uri),
    };

    if let Some(h) = headers {
        for (key, val) in h {
            request = request.header(&key, &val);
        }
    }

    if let Some(b) = body {
        request = request.body(b);
    }

    request
        .send()
        .await
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())
}
