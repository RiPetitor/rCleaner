use crate::tui::action::Action;
use crate::tui::store::Store;

pub struct Dispatcher {
    store: Store,
}

impl Dispatcher {
    pub fn new() -> Self {
        Self {
            store: Store::new(),
        }
    }

    pub fn dispatch(&mut self, action: Action) {
        self.store.update(action);
    }

    pub fn store(&self) -> &Store {
        &self.store
    }

    pub fn store_mut(&mut self) -> &mut Store {
        &mut self.store
    }
}
