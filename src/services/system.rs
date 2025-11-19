use mlua::{Lua, Result};
use std::fs;
use std::process::Command;

pub fn register(lua: &Lua) -> Result<()> {
    let sys = lua.create_table()?;

    sys.set(
        "get_battery",
        lua.create_function(|_, ()| {
            let bat_path = if std::path::Path::new("/sys/class/power_supply/BAT0").exists() {
                "/sys/class/power_supply/BAT0"
            } else {
                "/sys/class/power_supply/BAT1"
            };

            let capacity = fs::read_to_string(format!("{}/capacity", bat_path))
                .ok()
                .and_then(|s| s.trim().parse::<i32>().ok())
                .unwrap_or(0);

            let status = fs::read_to_string(format!("{}/status", bat_path))
                .ok()
                .map(|s| s.trim().to_string())
                .unwrap_or_else(|| "Unknown".to_string());

            Ok((capacity, status))
        })?,
    )?;

    sys.set(
        "get_wifi_ssid",
        lua.create_function(|_, ()| {
            let output = Command::new("nmcli")
                .args(["-t", "-f", "active,ssid", "dev", "wifi"])
                .output()
                .ok()
                .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
                .unwrap_or_default();

            for line in output.lines() {
                if line.starts_with("yes:") {
                    return Ok(line.trim_start_matches("yes:").to_string());
                }
            }
            Ok("Disconnected".to_string())
        })?,
    )?;

    sys.set(
        "media_info",
        lua.create_function(|_, ()| {
            let title = Command::new("playerctl")
                .arg("metadata")
                .arg("title")
                .output()
                .ok()
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                .unwrap_or_default();

            let artist = Command::new("playerctl")
                .arg("metadata")
                .arg("artist")
                .output()
                .ok()
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                .unwrap_or_default();

            Ok((title, artist))
        })?,
    )?;

    lua.globals().set("System", sys)?;
    Ok(())
}
