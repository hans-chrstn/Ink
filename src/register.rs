use gtk4::prelude::StaticType;
macro_rules! register_widgets {
    ($($widget_type:ty),* $(,)?) => {
        $(
            <$widget_type>::static_type();
        )*
    };
}

pub fn register_widgets() {
    register_widgets! {
        gtk4::AboutDialog,
        gtk4::AppChooserDialog,
        gtk4::ApplicationWindow,
        gtk4::ColorChooserDialog,
        gtk4::Dialog,
        gtk4::FileChooserDialog,
        gtk4::FontChooserDialog,
        gtk4::MessageDialog,
        gtk4::Window,

        gtk4::ActionBar,
        gtk4::AspectFrame,
        gtk4::Box,
        gtk4::CenterBox,
        gtk4::Expander,
        gtk4::Fixed,
        gtk4::Frame,
        gtk4::Grid,
        gtk4::HeaderBar,
        gtk4::Notebook,
        gtk4::Overlay,
        gtk4::Paned,
        gtk4::ScrolledWindow,
        gtk4::Separator,
        gtk4::Stack,
        gtk4::StackSidebar,
        gtk4::StackSwitcher,

        gtk4::AppChooserButton,
        gtk4::Button,
        gtk4::CheckButton,
        gtk4::ColorButton,
        gtk4::DropDown,
        gtk4::Entry,
        gtk4::FontButton,
        gtk4::LinkButton,
        gtk4::MenuButton,
        gtk4::PasswordEntry,
        gtk4::Scale,
        gtk4::SearchBar,
        gtk4::SearchEntry,
        gtk4::SpinButton,
        gtk4::Switch,
        gtk4::ToggleButton,
        gtk4::VolumeButton,

        gtk4::Calendar,
        gtk4::DrawingArea,
        gtk4::Image,
        gtk4::InfoBar,
        gtk4::Label,
        gtk4::LevelBar,
        gtk4::ListView,
        gtk4::Picture,
        gtk4::ProgressBar,
        gtk4::Spinner,
        gtk4::Statusbar,
        gtk4::TextView,
    }
}
