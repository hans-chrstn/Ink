use mlua::{Lua, Result};
use std::process::Command;
use std::thread;

pub fn init(lua: &Lua) -> Result<()> {
    let globals = lua.globals();

    let exec = lua.create_function(|_, cmd: String| {
        let output = Command::new("sh")
            .arg("-c")
            .arg(cmd)
            .output()
            .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr).to_string();
            return Ok(format!("Error: {}", err));
        }

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(stdout.trim().to_string())
    })?;
    globals.set("exec", exec)?;

    let spawn = lua.create_function(|_, cmd: String| {
        thread::spawn(move || {
            let _ = Command::new("sh").arg("-c").arg(cmd).spawn();
        });
        Ok(())
    })?;
    globals.set("spawn", spawn)?;

    let fetch = lua.create_function(|_, url: String| {
        let resp = reqwest::blocking::get(url)
            .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?
            .text()
            .map_err(|e| mlua::Error::RuntimeError(e.to_string()))?;
        Ok(resp)
    })?;
    globals.set("fetch", fetch)?;

    Ok(())
}
