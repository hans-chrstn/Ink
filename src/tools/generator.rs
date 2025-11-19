use crate::ui::registry::Registry;
use gtk4::glib::object::ObjectClass;
use gtk4::prelude::*;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

pub fn generate(target_dir: Option<PathBuf>) -> std::io::Result<()> {
    let dir = match target_dir {
        Some(d) => d,
        None => {
            let home = std::env::var("HOME").expect("Could not find HOME directory");
            PathBuf::from(home).join(".config").join("ink")
        }
    };

    if !dir.exists() {
        fs::create_dir_all(&dir)?;
    }

    generate_definitions(&dir.join("definitions.lua"))?;
    generate_luarc(&dir.join(".luarc.json"))?;
    generate_main(&dir.join("main.lua"))?;
    generate_config(&dir.join("config.lua"))?;

    Ok(())
}

fn generate_definitions(path: &Path) -> std::io::Result<()> {
    let mut f = File::create(path)?;
    writeln!(f, "---@meta")?;
    writeln!(
        f,
        "-- Auto-generated definitions for Ink. Do not edit manually."
    )?;
    writeln!(f, "")?;

    writeln!(f, "---@class WidgetConfig")?;
    writeln!(f, "---@field type string")?;
    writeln!(f, "---@field properties? table")?;
    writeln!(f, "---@field signals? table")?;
    writeln!(f, "---@field children? WidgetConfig[]")?;
    writeln!(f, "")?;

    for t in Registry::get_all_types() {
        let name = t.name();
        let props_name = format!("{}Props", name);

        writeln!(f, "---@class {}", props_name)?;
        if let Some(oc) = ObjectClass::from_type(t) {
            for p in oc.list_properties() {
                writeln!(
                    f,
                    "---@field {}? any -- {}",
                    p.name().replace('-', "_"),
                    p.value_type().name()
                )?;
            }
        }
        writeln!(f, "")?;

        writeln!(f, "---@class {}Config : WidgetConfig", name)?;
        writeln!(f, "---@field type \"{}\"", name)?;
        writeln!(f, "---@field properties? {}", props_name)?;

        if name == "GtkApplicationWindow" {
            writeln!(f, "---@field window_mode? \"layer_shell\" | \"normal\"")?;
            writeln!(
                f,
                "---@field layer? \"top\" | \"bottom\" | \"overlay\" | \"background\""
            )?;
            writeln!(
                f,
                "---@field anchors? {{ top: boolean, bottom: boolean, left: boolean, right: boolean }}"
            )?;
            writeln!(
                f,
                "---@field keyboard_mode? \"none\" | \"exclusive\" | \"on_demand\""
            )?;
        }
        writeln!(f, "")?;
    }
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
    let content = r#"local cfg = require("config")

---@type WindowConfig
return {
	type = "GtkApplicationWindow",

  -- css_path = "path-to-your-css.css",
	css = [[
        button {
            background-color: gray;
            color: black;
            border-radius: 12px;
        }
        label {
            font-size: 20px;
            color: black;
        }

        .my-window {
          background-color: white;
          border-radius: 20px;
          margin: 5px;
        }

        .bar {
          padding: 5px;
        }
    ]],
	window_mode = "layer_shell",
	layer = "top",
	anchors = { top = true, left = true, right = true, bottom = false },
	margins = { top = 10, left = 10, right = 10 },
	auto_exclusive_zone = true,
	properties = {
		title = "My Ink Bar",
		default_height = 40,
		css_classes = { "my-window" },
	},
	children = {
		{
			type = "GtkBox",
			properties = {
				orientation = "horizontal",
				spacing = cfg.spacing,
				hexpand = true,
        css_classes = { "bar" },
			},
			children = {
				{ type = "GtkLabel", properties = { label = "<b>Ink</b> System", use_markup = true } },
				{ 
          type = "GtkButton", 
          properties = { label = "Click Me" },
					signals = { clicked = function() print("Button was clicked!") end },
				},
				{
					type = "GtkButton",
					properties = { label = "Exit" },
					signals = { clicked = function() print("Exit clicked") end },
				},
			},
		},
	},
}"#;
    fs::write(path, content)?;
    Ok(())
}

fn generate_config(path: &Path) -> std::io::Result<()> {
    if path.exists() {
        return Ok(());
    }
    let content = r##"return {
    spacing = 12,
    primary_color = "#blue"
}"##;
    fs::write(path, content)?;
    Ok(())
}
