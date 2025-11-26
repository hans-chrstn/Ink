use gtk4::glib;
use libpulse_binding::callbacks::ListResult;
use libpulse_binding::context::subscribe::InterestMaskSet;
use libpulse_binding::context::{Context, FlagSet as ContextFlagSet};
use libpulse_binding::mainloop::standard::Mainloop;
use libpulse_binding::volume::{ChannelVolumes, Volume};
use mlua::{Function, Lua, Result as LuaResult};
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;
use tokio::sync::mpsc;

#[derive(Debug)]
pub enum AudioError {
    ConnectionFailed(String),
    OperationFailed(String),
    PulseError(libpulse_binding::error::PAErr),
    LuaError(mlua::Error),
    Other(String),
}

unsafe impl Send for AudioError {}
unsafe impl Sync for AudioError {}

impl std::fmt::Display for AudioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioError::ConnectionFailed(msg) => write!(f, "PulseAudio connection failed: {}", msg),
            AudioError::OperationFailed(msg) => write!(f, "PulseAudio operation failed: {}", msg),
            AudioError::PulseError(e) => write!(f, "PulseAudio error: {}", e),
            AudioError::LuaError(e) => write!(f, "Lua error: {}", e),
            AudioError::Other(msg) => write!(f, "Audio error: {}", msg),
        }
    }
}

impl std::error::Error for AudioError {}

impl From<libpulse_binding::error::PAErr> for AudioError {
    fn from(err: libpulse_binding::error::PAErr) -> Self {
        AudioError::PulseError(err)
    }
}

impl From<AudioError> for mlua::Error {
    fn from(err: AudioError) -> Self {
        mlua::Error::external(err)
    }
}

pub struct PulseAudioClient {
    context: Rc<RefCell<Context>>,
    mainloop: Mainloop,
}

impl Drop for PulseAudioClient {
    fn drop(&mut self) {
        if let Ok(mut ctx) = self.context.try_borrow_mut() {
            ctx.disconnect();
        }
    }
}

impl PulseAudioClient {
    pub fn new() -> std::result::Result<Self, AudioError> {
        let mut mainloop = Mainloop::new()
            .ok_or_else(|| AudioError::ConnectionFailed("Failed to create mainloop".into()))?;

        let mut context = Context::new(&mainloop, "Ink Client")
            .ok_or_else(|| AudioError::ConnectionFailed("Failed to create context".into()))?;

        context
            .connect(None, ContextFlagSet::NOFLAGS, None)
            .map_err(|e| {
                AudioError::ConnectionFailed(format!("Failed to connect context: {}", e))
            })?;

        loop {
            mainloop.iterate(false);

            match context.get_state() {
                libpulse_binding::context::State::Ready => break,
                libpulse_binding::context::State::Failed => {
                    return Err(AudioError::ConnectionFailed(
                        "PulseAudio context failed".into(),
                    ));
                }
                libpulse_binding::context::State::Terminated => {
                    return Err(AudioError::ConnectionFailed(
                        "PulseAudio context terminated".into(),
                    ));
                }
                _ => {}
            }
        }

        Ok(Self {
            context: Rc::new(RefCell::new(context)),
            mainloop,
        })
    }

    pub fn get_volume(&mut self) -> std::result::Result<i32, AudioError> {
        let vol_percent = Rc::new(RefCell::new(0));
        let done = Rc::new(RefCell::new(false));
        let keep_alive: Rc<RefCell<Vec<Box<dyn Any>>>> = Rc::new(RefCell::new(Vec::new()));

        let context = self.context.clone();
        let ctx_cb1 = context.clone();
        let vol_res = vol_percent.clone();
        let done_res = done.clone();
        let keep_alive_res = keep_alive.clone();

        let op = context
            .borrow_mut()
            .introspect()
            .get_server_info(move |info| {
                if let Some(default_sink) = &info.default_sink_name {
                    let vol_inner = vol_res.clone();
                    let done_inner = done_res.clone();
                    let op2 = ctx_cb1.borrow_mut().introspect().get_sink_info_by_name(
                        default_sink.as_ref(),
                        move |list_res| {
                            if let ListResult::Item(si) = list_res {
                                let avg = si.volume.avg().0 as f64;
                                *vol_inner.borrow_mut() = (avg / 65536.0 * 100.0).round() as i32;
                            }
                            *done_inner.borrow_mut() = true;
                        },
                    );
                    keep_alive_res.borrow_mut().push(Box::new(op2));
                } else {
                    *done_res.borrow_mut() = true;
                }
            });

        keep_alive.borrow_mut().push(Box::new(op));

        while !*done.borrow() {
            self.mainloop.iterate(false);
        }

        let result = *vol_percent.borrow();
        Ok(result)
    }

    pub fn set_volume(&mut self, percent: i32) -> std::result::Result<(), AudioError> {
        let context = self.context.clone();
        let target_vol = ((percent as f64 / 100.0) * 65536.0) as u32;
        let done = Rc::new(RefCell::new(false));
        let keep_alive: Rc<RefCell<Vec<Box<dyn Any>>>> = Rc::new(RefCell::new(Vec::new()));
        let ctx_cb1 = context.clone();
        let done_res = done.clone();
        let keep_alive_res = keep_alive.clone();

        let op = context
            .borrow_mut()
            .introspect()
            .get_server_info(move |info| {
                if let Some(default_sink) = &info.default_sink_name {
                    let sink_name = default_sink.to_string();
                    let ctx_cb2 = ctx_cb1.clone();
                    let done_inner = done_res.clone();
                    let keep_alive_inner = keep_alive_res.clone();
                    let op2 = ctx_cb1.borrow_mut().introspect().get_sink_info_by_name(
                        &sink_name,
                        move |list_res| {
                            if let ListResult::Item(sink) = list_res {
                                let mut new_cv = ChannelVolumes::default();
                                new_cv.set(sink.volume.len(), Volume(target_vol));
                                let op3 = ctx_cb2
                                    .borrow_mut()
                                    .introspect()
                                    .set_sink_volume_by_index(sink.index, &new_cv, None);
                                keep_alive_inner.borrow_mut().push(Box::new(op3));
                            }
                            *done_inner.borrow_mut() = true;
                        },
                    );
                    keep_alive_res.borrow_mut().push(Box::new(op2));
                } else {
                    *done_res.borrow_mut() = true;
                }
            });

        keep_alive.borrow_mut().push(Box::new(op));

        while !*done.borrow() {
            self.mainloop.iterate(false);
        }

        for _ in 0..5 {
            self.mainloop.iterate(false);
        }

        Ok(())
    }
}

pub fn register(lua: Rc<Lua>) -> LuaResult<()> {
    let audio = lua.create_table()?;

    audio.set(
        "get_volume",
        lua.create_async_function(|_lua, ()| async move {
            let result = tokio::task::spawn_blocking(move || {
                let mut client = PulseAudioClient::new()?;
                client.get_volume()
            })
            .await
            .map_err(|e| {
                mlua::Error::external(format!("Failed to spawn blocking task: {}", e))
            })??;

            Ok(result)
        })?,
    )?;

    audio.set(
        "set_volume",
        lua.create_async_function(|_, percent: i32| async move {
            tokio::task::spawn_blocking(move || {
                let mut client = PulseAudioClient::new()?;
                client.set_volume(percent)
            })
            .await
            .map_err(|e| {
                mlua::Error::external(format!("Failed to spawn blocking task: {}", e))
            })??;

            Ok(())
        })?,
    )?;

    {
        let lua_watch = lua.clone();
        audio.set(
            "watch",
            lua.create_function(move |_, callback: Function| {
                let cb_key = lua_watch.create_registry_value(callback)?;
                let (sender, mut receiver) = mpsc::unbounded_channel();
                std::thread::spawn(move || {
                    let mut handler = match PulseAudioClient::new() {
                        Ok(client) => client,
                        Err(e) => {
                            eprintln!("Audio Error: Failed to connect to Pulse/Pipewire: {}", e);
                            return;
                        }
                    };
                    let context = handler.context.clone();
                    context
                        .borrow_mut()
                        .set_subscribe_callback(Some(Box::new(move |_, _, _| {
                            let _ = sender.send(());
                        })));
                    context
                        .borrow_mut()
                        .subscribe(InterestMaskSet::SINK, |_| {});

                    if let Err(e) = handler.mainloop.run() {
                        eprintln!("Error running PulseAudio mainloop: {:?}", e);
                    }
                });

                let lua_inner = lua_watch.clone();
                glib::MainContext::default().spawn_local(async move {
                    while receiver.recv().await.is_some() {
                        if let LuaResult::Ok(func) = lua_inner.registry_value::<Function>(&cb_key) {
                            let _ = func.call::<()>(());
                        }
                    }
                });
                Ok(())
            })?,
        )?;
    }
    lua.globals().set("Audio", audio)?;
    Ok(())
}
