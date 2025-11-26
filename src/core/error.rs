use gtk4::{DialogFlags, MessageType, prelude::*};
use std::error::Error as StdError;
use std::fmt;

use mlua;

use crate::interop::converter::ConversionError;
use crate::services::audio::AudioError;
use crate::services::system::SystemError;
use crate::services::tray_api::TrayApiError;

#[derive(Debug)]
pub enum AppError {
    LuaError(mlua::Error),
    IoError(std::io::Error),
    ConversionError(ConversionError),
    TrayApiError(TrayApiError),
    SystemError(SystemError),
    AudioError(AudioError),
    AppSetupError(String),
    GtkError(String),
    Other(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::LuaError(e) => write!(f, "Lua error: {}", e),
            AppError::IoError(e) => write!(f, "I/O error: {}", e),
            AppError::ConversionError(e) => write!(f, "Conversion error: {}", e),
            AppError::TrayApiError(e) => write!(f, "Tray API error: {}", e),
            AppError::SystemError(e) => write!(f, "System error: {}", e),
            AppError::AudioError(e) => write!(f, "Audio error: {}", e),
            AppError::AppSetupError(e) => write!(f, "Application setup error: {}", e),
            AppError::GtkError(e) => write!(f, "GTK error: {}", e),
            AppError::Other(e) => write!(f, "An unexpected error occurred: {}", e),
        }
    }
}

impl StdError for AppError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            AppError::LuaError(e) => Some(e),
            AppError::IoError(e) => Some(e),
            AppError::ConversionError(e) => Some(e),
            AppError::TrayApiError(e) => Some(e),
            AppError::SystemError(e) => Some(e),
            AppError::AudioError(e) => Some(e),
            _ => None,
        }
    }
}

impl From<mlua::Error> for AppError {
    fn from(err: mlua::Error) -> Self {
        AppError::LuaError(err)
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::IoError(err)
    }
}

impl From<ConversionError> for AppError {
    fn from(err: ConversionError) -> Self {
        AppError::ConversionError(err)
    }
}

impl From<TrayApiError> for AppError {
    fn from(err: TrayApiError) -> Self {
        AppError::TrayApiError(err)
    }
}

impl From<SystemError> for AppError {
    fn from(err: SystemError) -> Self {
        AppError::SystemError(err)
    }
}

impl From<AudioError> for AppError {
    fn from(err: AudioError) -> Self {
        AppError::AudioError(err)
    }
}

impl From<AppError> for mlua::Error {
    fn from(err: AppError) -> Self {
        mlua::Error::external(err)
    }
}
pub fn handle_error(app: &gtk4::Application, title: &str, error: &AppError) {
    let dialog = gtk4::MessageDialog::new(
        None::<&gtk4::Window>,
        DialogFlags::MODAL,
        MessageType::Error,
        gtk4::ButtonsType::Close,
        title,
    );
    dialog.set_secondary_text(Some(&error.to_string()));
    dialog.set_application(Some(app));
    dialog.connect_response(|dialog, _| {
        dialog.close();
    });
    dialog.show();
}
