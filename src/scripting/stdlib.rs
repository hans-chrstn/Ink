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
        let _ = Command::new("sh").arg("-c").arg(cmd).spawn();
    });
}

pub fn fetch(url: &str) -> Result<String, String> {
    reqwest::blocking::get(url)
        .map_err(|e| e.to_string())?
        .text()
        .map_err(|e| e.to_string())
}
