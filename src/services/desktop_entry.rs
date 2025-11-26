use std::fs;
use std::path::Path;

#[derive(Debug, Default)]
pub struct DesktopEntry {
    pub name: Option<String>,
    pub icon: Option<String>,
}

impl DesktopEntry {
    pub fn parse_from_file<P: AsRef<Path>>(path: P) -> Option<Self> {
        let content = fs::read_to_string(path).ok()?;
        Self::parse_from_string(&content)
    }

    pub fn parse_from_string(content: &str) -> Option<Self> {
        let mut name: Option<String> = None;
        let mut icon: Option<String> = None;

        let mut in_desktop_entry_section = false;

        for line in content.lines() {
            let line = line.trim();
            if line.starts_with('[') && line.ends_with(']') {
                in_desktop_entry_section = line == "[Desktop Entry]";
                continue;
            }

            if !in_desktop_entry_section || line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();

                if key == "Name" || key.starts_with("Name[") {
                    if name.is_none() || key == "Name" {
                        name = Some(value.to_string());
                    }
                } else if key == "Icon" {
                    icon = Some(value.to_string());
                }
            }
        }

        if name.is_some() || icon.is_some() {
            Some(DesktopEntry { name, icon })
        } else {
            None
        }
    }
}
