use gio::prelude::*;
use gio::{File, FileMonitor, FileMonitorEvent};
use mlua::{Function, Lua, Result, UserData, UserDataMethods};
use std::fs;
use std::rc::Rc;

struct FileWatcher {
    monitor: FileMonitor,
}

impl Drop for FileWatcher {
    fn drop(&mut self) {
        self.monitor.cancel();
    }
}

impl UserData for FileWatcher {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("disconnect", |_, this, ()| {
            this.monitor.cancel();
            Ok(())
        });
    }
}

pub fn register(lua: Rc<Lua>) -> Result<()> {
    let fs_table = lua.create_table()?;

    fs_table.set(
        "read_file",
        lua.create_function(|_, path: String| {
            fs::read_to_string(path).map_err(mlua::Error::external)
        })?,
    )?;
    fs_table.set(
        "write_file",
        lua.create_function(|_, (path, content): (String, String)| {
            fs::write(path, content).map_err(mlua::Error::external)
        })?,
    )?;
    fs_table.set(
        "exists",
        lua.create_function(|_, path: String| Ok(std::path::Path::new(&path).exists()))?,
    )?;

    fs_table.set(
        "watch",
        lua.create_function({
            let lua_rc = lua.clone();
            move |_, (path, callback): (String, Function)| {
                let file = File::for_path(&path);
                let monitor = file
                    .monitor(
                        gio::FileMonitorFlags::WATCH_MOVES,
                        None::<&gio::Cancellable>,
                    )
                    .map_err(mlua::Error::external)?;

                let callback_key = lua_rc
                    .create_registry_value(callback)
                    .map_err(mlua::Error::external)?;

                monitor.connect_changed({
                    let lua = lua_rc.clone();
                    move |_, _file, _other_file, event_type| {
                        if let Ok(callback) = lua.registry_value::<Function>(&callback_key) {
                            let event_str = match event_type {
                                FileMonitorEvent::Changed => "changed",
                                FileMonitorEvent::ChangesDoneHint => "changes_done_hint",
                                FileMonitorEvent::Deleted => "deleted",
                                FileMonitorEvent::Created => "created",
                                FileMonitorEvent::AttributeChanged => "attribute_changed",
                                FileMonitorEvent::PreUnmount => "pre_unmount",
                                FileMonitorEvent::Unmounted => "unmounted",
                                FileMonitorEvent::Moved => "moved",
                                _ => "unknown",
                            };
                            if let Err(e) = callback.call::<()>(event_str) {
                                eprintln!("Error in file watcher callback: {}", e);
                            }
                        }
                    }
                });

                Ok(FileWatcher { monitor })
            }
        })?,
    )?;

    lua.globals().set("Files", fs_table)?;
    Ok(())
}
