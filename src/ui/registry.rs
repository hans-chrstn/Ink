use crate::ui::traits::{LeafStrategy, WidgetContainer};
use gtk4::glib::Type;
use gtk4::prelude::*;
use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

type StrategyFactory = Box<dyn Fn(&gtk4::Widget) -> Box<dyn WidgetContainer> + Send + Sync>;

pub struct Registry {
    types: HashMap<String, Type>,
    strategies: HashMap<String, StrategyFactory>,
}

impl Registry {
    fn global() -> &'static RwLock<Registry> {
        static INSTANCE: OnceLock<RwLock<Registry>> = OnceLock::new();
        INSTANCE.get_or_init(|| {
            RwLock::new(Registry {
                types: HashMap::new(),
                strategies: HashMap::new(),
            })
        })
    }

    pub fn register_leaf<T: IsA<gtk4::Widget> + StaticType>() {
        Self::register_impl::<T>(Box::new(|_| Box::new(LeafStrategy)));
    }

    pub fn register_container<T: IsA<gtk4::Widget> + StaticType + WidgetContainer + Clone>() {
        Self::register_impl::<T>(Box::new(|w| {
            if let Some(obj) = w.downcast_ref::<T>() {
                Box::new(obj.clone())
            } else {
                Box::new(LeafStrategy)
            }
        }));
    }

    fn register_impl<T: StaticType>(factory: StrategyFactory) {
        let mut lock = Self::global().write().unwrap();
        let t = T::static_type();
        let name = t.name().to_string();
        lock.types.insert(name.clone(), t);
        lock.strategies.insert(name, factory);
    }

    pub fn get_type(name: &str) -> Option<Type> {
        Self::global().read().unwrap().types.get(name).copied()
    }

    pub fn get_strategy(name: &str, w: &gtk4::Widget) -> Box<dyn WidgetContainer> {
        let lock = Self::global().read().unwrap();
        if let Some(f) = lock.strategies.get(name) {
            f(w)
        } else {
            Box::new(LeafStrategy)
        }
    }

    pub fn get_all_types() -> Vec<Type> {
        Self::global()
            .read()
            .unwrap()
            .types
            .values()
            .copied()
            .collect()
    }
}
