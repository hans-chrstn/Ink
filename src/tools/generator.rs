use crate::ui::registry::Registry;
use gtk4::glib::object::ObjectClass;
use gtk4::prelude::*;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

pub fn generate(path: Option<PathBuf>) -> std::io::Result<()> {
    let p = path.unwrap_or_else(|| PathBuf::from("definitions.lua"));
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut f = File::create(&p)?;
    writeln!(
        f,
        "---@meta\n---@class WidgetConfig\n---@field type string\n---@field properties? table\n---@field signals? table\n---@field children? WidgetConfig[]\n"
    )?;

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
            writeln!(f, "---@field layer? \"top\" | \"bottom\"")?;
            writeln!(f, "---@field anchors? table")?;
        }
        writeln!(f, "")?;
    }
    println!("Generated: {:?}", p);
    Ok(())
}
