use mlua::{Lua, Result};
use std::process::Command;

pub fn register(lua: &Lua) -> Result<()> {
    let audio = lua.create_table()?;

    audio.set(
        "get_volume",
        lua.create_function(|_, ()| {
            let output = if let Ok(out) = Command::new("wpctl")
                .arg("get-volume")
                .arg("@DEFAULT_AUDIO_SINK@")
                .output()
            {
                String::from_utf8_lossy(&out.stdout).to_string()
            } else if let Ok(out) = Command::new("pactl")
                .arg("get-sink-volume")
                .arg("@DEFAULT_SINK@")
                .output()
            {
                String::from_utf8_lossy(&out.stdout).to_string()
            } else {
                return Ok(0);
            };

            if let Some(idx) = output.find("Volume:") {
                let slice = &output[idx + 7..];
                let clean: String = slice
                    .chars()
                    .filter(|c| c.is_numeric() || *c == '.')
                    .collect();
                let vol: f64 = clean.parse().unwrap_or(0.0);
                return Ok((vol * 100.0) as i32);
            }

            Ok(0)
        })?,
    )?;

    audio.set(
        "set_volume",
        lua.create_function(|_, percent: i32| {
            let vol_str = format!("{:.2}", percent as f64 / 100.0);
            let _ = Command::new("wpctl")
                .arg("set-volume")
                .arg("@DEFAULT_AUDIO_SINK@")
                .arg(vol_str)
                .spawn();
            Ok(())
        })?,
    )?;

    lua.globals().set("Audio", audio)?;
    Ok(())
}
