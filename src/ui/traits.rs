use gtk4::Widget;
use gtk4::prelude::*;

pub trait WidgetContainer {
    fn add_child(&self, child: &Widget);
}

pub struct LeafStrategy;
impl WidgetContainer for LeafStrategy {
    fn add_child(&self, _: &Widget) {
        eprintln!(
            "Warn: Cannot add child to this widget (it might be a leaf or unsupported container)."
        );
    }
}

macro_rules! impl_set_child {
    ($($t:ty),*) => {
        $(
            impl WidgetContainer for $t {
                fn add_child(&self, child: &Widget) {
                    self.set_child(Some(child));
                }
            }
        )*
    };
}

impl_set_child!(
    gtk4::Window,
    gtk4::ApplicationWindow,
    gtk4::Dialog,
    gtk4::AboutDialog,
    gtk4::AppChooserDialog,
    gtk4::ColorChooserDialog,
    gtk4::FileChooserDialog,
    gtk4::FontChooserDialog,
    gtk4::MessageDialog,
    gtk4::Frame,
    gtk4::AspectFrame,
    gtk4::ScrolledWindow,
    gtk4::Overlay,
    gtk4::Expander,
    gtk4::Viewport,
    gtk4::Popover,
    gtk4::Button,
    gtk4::ToggleButton,
    gtk4::LinkButton,
    gtk4::Revealer,
    gtk4::WindowHandle
);

impl WidgetContainer for gtk4::ListBox {
    fn add_child(&self, child: &gtk4::Widget) {
        self.append(child);
    }
}

impl WidgetContainer for gtk4::FlowBox {
    fn add_child(&self, child: &gtk4::Widget) {
        self.insert(child, -1); // -1 means append
    }
}

impl WidgetContainer for gtk4::MenuButton {
    fn add_child(&self, child: &Widget) {
        self.set_property("child", child);
    }
}

impl WidgetContainer for gtk4::Box {
    fn add_child(&self, child: &Widget) {
        self.append(child);
    }
}

impl WidgetContainer for gtk4::CenterBox {
    fn add_child(&self, child: &Widget) {
        self.set_center_widget(Some(child));
    }
}

impl WidgetContainer for gtk4::HeaderBar {
    fn add_child(&self, child: &Widget) {
        self.pack_end(child);
    }
}

impl WidgetContainer for gtk4::Stack {
    fn add_child(&self, child: &Widget) {
        self.add_child(child);
    }
}

impl WidgetContainer for gtk4::ActionBar {
    fn add_child(&self, child: &Widget) {
        self.pack_start(child);
    }
}

impl WidgetContainer for gtk4::Notebook {
    fn add_child(&self, child: &Widget) {
        self.append_page(child, None::<&Widget>);
    }
}

impl WidgetContainer for gtk4::Paned {
    fn add_child(&self, child: &Widget) {
        if self.start_child().is_none() {
            self.set_start_child(Some(child));
        } else {
            self.set_end_child(Some(child));
        }
    }
}
