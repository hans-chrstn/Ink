use gtk4::{prelude::*, DialogFlags, MessageType};
pub fn handle_error(app: &gtk4::Application, title: &str, message: &str) {
    let dialog = gtk4::MessageDialog::new(
        None::<&gtk4::Window>,
        DialogFlags::MODAL,
        MessageType::Error,
        gtk4::ButtonsType::Close,
        title,
    );
    dialog.set_secondary_text(Some(message));
    dialog.set_application(Some(app));
    dialog.connect_response(|dialog, _| {
        dialog.close();
    });
    dialog.show();
}
