use crate::ui::registry::Registry;
use gtk4::glib::object::ObjectClass;
use gtk4::prelude::*;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use tera::{Context, Tera};
pub fn generate(target_dir: Option<PathBuf>) -> std::io::Result<()> {
    let dir = match target_dir {
        Some(d) => d,
        None => {
            let home = std::env::var("HOME").map_err(|e| {
                io::Error::other(
                    format!("Could not find HOME directory: {}", e),
                )
            })?;
            PathBuf::from(home).join(".config").join("ink")
        }
    };
    if !dir.exists() {
        fs::create_dir_all(&dir)?;
    }
    generate_definitions(&dir.join("definitions.lua"))?;
    generate_luarc(&dir.join(".luarc.json"))?;
    generate_main(&dir.join("init.lua"))?;
    generate_config(&dir.join("config.lua"))?;
    Ok(())
}
fn generate_definitions(path: &Path) -> std::io::Result<()> {
            let mut tera = Tera::new("src/tools/templates/**/*").map_err(|e| {
                io::Error::other(
                    format!("Failed to create Tera instance: {}", e),
                )
            })?;    tera.autoescape_on(vec![]);

    let mut context = Context::new();
    let mut widgets_data = Vec::new();

    for t in Registry::get_all_types() {
        let name = t.name().to_string();
        let mut properties = Vec::new();

        if let Some(oc) = ObjectClass::from_type(t) {
            for p in oc.list_properties() {
                properties.push(serde_json::json!({
                    "name": p.name().replace('-', "_"),
                    "type": p.value_type().name(),
                }));
            }
        }

        widgets_data.push(serde_json::json!({
            "name": name,
            "properties": properties,
        }));
    }

    context.insert("widgets", &widgets_data);

    let rendered = tera.render("definitions.lua.tera", &context).map_err(|e| {
        io::Error::other(
            format!("Failed to render definitions.lua.tera: {}", e),
        )
    })?;
    fs::write(path, rendered)?;
    Ok(())
}
fn generate_luarc(path: &Path) -> std::io::Result<()> {
    let content = r#"{
    "workspace": {
        "library": [
            "definitions.lua"
        ],
        "checkThirdParty": false
    },
    "diagnostics": {
        "globals": [
            "exec",
            "spawn",
            "fetch"
        ]
    }
}"#;
    fs::write(path, content)?;
    Ok(())
}
fn generate_main(path: &Path) -> std::io::Result<()> {
    if path.exists() {
        return Ok(());
    }
    let tera = Tera::new("src/tools/templates/**/*").map_err(|e| {
        io::Error::other(
            format!("Failed to create Tera instance: {}", e),
        )
    })?;
    let context = Context::new();
    let rendered = tera.render("ink.lua.tera", &context).map_err(|e| {
        io::Error::other(
            format!("Failed to render ink.lua.tera: {}", e),
        )
    })?;
    fs::write(path, rendered)?;
    Ok(())
}
fn generate_config(path: &Path) -> std::io::Result<()> {
    if path.exists() {
        return Ok(());
    }
    let tera = Tera::new("src/tools/templates/**/*").map_err(|e| {
        io::Error::other(
            format!("Failed to create Tera instance: {}", e),
        )
    })?;
    let context = Context::new();
    let rendered = tera.render("config.lua.tera", &context).map_err(|e| {
        io::Error::other(
            format!("Failed to render config.lua.tera: {}", e),
        )
    })?;
    fs::write(path, rendered)?;
    Ok(())
}
