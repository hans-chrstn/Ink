use crate::scripting::traits::ScriptValue;
use gtk4::Widget;
use gtk4::prelude::*;

pub trait WidgetContainer<T: ScriptValue> {
    fn add_child(&self, child: &Widget, data: &T);
}

pub trait WidgetBehavior<T: ScriptValue>: Send + Sync {
    fn apply(&self, widget: &Widget, data: &T);
}

pub struct LeafStrategy;
impl<T: ScriptValue> WidgetContainer<T> for LeafStrategy {
    fn add_child(&self, _: &Widget, _: &T) {
        eprintln!("Warn: Cannot add child to this widget (it might be a leaf).");
    }
}

macro_rules! impl_set_child {
    ($($t:ty),*) => {
        $(
            impl<T: ScriptValue> WidgetContainer<T> for $t {
                fn add_child(&self, child: &Widget, _: &T) {
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

impl<T: ScriptValue> WidgetContainer<T> for gtk4::CheckButton {
    fn add_child(&self, child: &Widget, _: &T) {
        self.set_property("child", child);
    }
}

impl<T: ScriptValue> WidgetContainer<T> for gtk4::MenuButton {
    fn add_child(&self, child: &Widget, _: &T) {
        self.set_property("child", child);
    }
}

impl<T: ScriptValue> WidgetContainer<T> for gtk4::LockButton {
    fn add_child(&self, _child: &Widget, _: &T) {
        eprintln!("Warn: LockButton does not support adding arbitrary children.");
    }
}

impl<T: ScriptValue> WidgetContainer<T> for gtk4::ListBox {
    fn add_child(&self, child: &gtk4::Widget, _: &T) {
        self.append(child);
    }
}

impl<T: ScriptValue> WidgetContainer<T> for gtk4::FlowBox {
    fn add_child(&self, child: &gtk4::Widget, _: &T) {
        self.insert(child, -1);
    }
}

impl<T: ScriptValue> WidgetContainer<T> for gtk4::Box {
    fn add_child(&self, child: &Widget, _: &T) {
        self.append(child);
    }
}

impl<T: ScriptValue> WidgetContainer<T> for gtk4::CenterBox {
    fn add_child(&self, child: &Widget, data: &T) {
        if let Some("start") = data
            .get_property("type_pos")
            .and_then(|v| v.as_string())
            .as_deref()
        {
            self.set_start_widget(Some(child));
        } else if let Some("end") = data
            .get_property("type_pos")
            .and_then(|v| v.as_string())
            .as_deref()
        {
            self.set_end_widget(Some(child));
        } else {
            self.set_center_widget(Some(child));
        }
    }
}

impl<T: ScriptValue> WidgetContainer<T> for gtk4::HeaderBar {
    fn add_child(&self, child: &Widget, _: &T) {
        self.pack_end(child);
    }
}

impl<T: ScriptValue> WidgetContainer<T> for gtk4::Stack {
    fn add_child(&self, child: &Widget, data: &T) {
        let name = data.get_property("name").and_then(|v| v.as_string());
        let title = data.get_property("title").and_then(|v| v.as_string());
        self.add_titled(child, name.as_deref(), title.as_deref().unwrap_or("Page"));
    }
}

impl<T: ScriptValue> WidgetContainer<T> for gtk4::ActionBar {
    fn add_child(&self, child: &Widget, _: &T) {
        self.pack_start(child);
    }
}

impl<T: ScriptValue> WidgetContainer<T> for gtk4::Notebook {
    fn add_child(&self, child: &Widget, data: &T) {
        let label_text = data.get_property("tab_label").and_then(|v| v.as_string());
        let label = gtk4::Label::new(label_text.as_deref());
        self.append_page(child, Some(&label));
    }
}

impl<T: ScriptValue> WidgetContainer<T> for gtk4::Paned {
    fn add_child(&self, child: &Widget, _: &T) {
        if self.start_child().is_none() {
            self.set_start_child(Some(child));
        } else {
            self.set_end_child(Some(child));
        }
    }
}
