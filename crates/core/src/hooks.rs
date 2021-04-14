// pub mod list_state;
pub mod memo;
pub mod reference;
pub mod state;

use std::{
    cell::RefCell,
    rc::{self, Rc},
};

use wasm_bindgen::{prelude::Closure, JsCast, JsValue};

use crate::{window, Builder, Element, ElementData};

#[derive(Default)]
pub struct Scope<T>(T);

impl<T: Scopeable> Scope<T> {
    pub fn new(value: T) -> Self {
        Self(value)
    }

    pub fn with<ElemBuilder, Elem, Gen>(&self, generate: Gen) -> Elem
    where
        ElemBuilder: Builder<Target = Elem>,
        Elem: Into<Element>,
        Element: Into<Elem>,
        Gen: 'static + Fn(&T::Item) -> ElemBuilder,
    {
        let element = self.0.generate(|scoped| generate(scoped).build().into());

        // TODO: What happens when we nest `with` calls on the same element. e.g.
        // parent.with(|x| child.with(|x| ...))? Replacing the generator should work Ok.
        self.0
            .link_to_parent(Rc::downgrade(&element.0), move |scoped| {
                generate(scoped).build().into()
            });

        element.into()
    }
}

pub trait Scopeable {
    type Item;

    fn generate<Gen: Fn(&Self::Item) -> Element>(&self, f: Gen) -> Element;

    fn link_to_parent<F>(&self, parent: rc::Weak<RefCell<ElementData>>, mk_elem: F)
    where
        F: 'static + Fn(&Self::Item) -> Element;
}

trait Update {
    fn parent(&self) -> &rc::Weak<RefCell<ElementData>>;

    fn apply(&self);
}

trait Effect {
    fn apply(&self);
}

fn queue_update(x: impl 'static + Update) {
    let len = {
        PENDING_UPDATES.with(|update_queue| {
            let mut update_queue = update_queue.borrow_mut();

            update_queue.push(Box::new(x));
            update_queue.len()
        })
    };

    if len == 1 {
        request_process_updates();
    }
}

fn request_process_updates() {
    ON_ANIMATION_FRAME.with(|process_updates| {
        window()
            .request_animation_frame(process_updates.as_ref().unchecked_ref())
            .unwrap()
    });
}

fn process_updates() {
    PENDING_UPDATES.with(|update_queue| {
        let mut update_queue = update_queue.take();

        if update_queue.len() != 1 {
            let mut updates_by_depth: Vec<_> = update_queue
                .into_iter()
                .filter_map(|u| u.parent().upgrade().map(|p| (p.borrow().dom_depth(), u)))
                .collect();

            updates_by_depth.sort_unstable_by_key(|(key, _)| *key);

            update_queue = updates_by_depth
                .into_iter()
                .map(|(_, value)| value)
                .collect();
        }

        for update in update_queue {
            update.apply();
        }
    });

    PENDING_EFFECTS.with(|effect_queue| {
        for effect in effect_queue.take() {
            effect.apply();
        }
    });
}

thread_local!(
    static PENDING_UPDATES: RefCell<Vec<Box<dyn Update>>> = RefCell::new(Vec::new());
    static PENDING_EFFECTS: RefCell<Vec<Box<dyn Effect>>> = RefCell::new(Vec::new());
    static ON_ANIMATION_FRAME: Closure<dyn FnMut(JsValue)> =
        Closure::wrap(Box::new(move |_time_stamp: JsValue| {
            process_updates();
        }));
);