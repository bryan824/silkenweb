// TODO: Need to think carefully about a minimal list container that
// filter/sort/etc can be built on top of.

use std::{cell::RefCell, collections::BTreeMap, mem, rc::Rc};

use web_sys as dom;

use super::state::{ReadSignal, Signal};
use crate::{DomElement, Element, ElementBuilder};

// TODO: Is this neccessary
type SharedItem<T> = Rc<T>;

struct StoredItem<T> {
    item: SharedItem<T>,
    updater: ReadSignal<()>,
}

// TODO: Rename
struct Storage {
    root: ElementBuilder,
    items: BTreeMap<usize, Element>,
}

impl Storage {
    // TODO: Add an `entry()` method
    pub fn insert(&mut self, key: usize, element: Element) {
        // TODO: Add a test to make sure a reactive element gives us the correct
        // dom_element.
        let dom_element = element.dom_element();

        if let Some(existing_elem) = self.items.insert(key, element) {
            self.root.remove_child(&existing_elem.dom_element());
        }

        // TODO: Put in the correct position in the list
        self.root.append_child(&dom_element);
    }

    pub fn remove(&mut self, key: usize) {
        if let Some(element) = self.items.remove(&key) {
            self.root.remove_child(&element.dom_element());
        }
    }
}

// TODO: Parameterize on key type
// TODO: Parameterize on storage type
pub struct ElementList<T> {
    storage: Rc<RefCell<Storage>>,
    generate_child: Rc<dyn Fn(&T) -> Element>,
    items: BTreeMap<usize, StoredItem<T>>,
    filter: Box<dyn Fn(&T) -> ReadSignal<bool>>,
}

impl<T: 'static> ElementList<T> {
    // TODO: Assert builders children empty.
    // How would we set attributes? Could take a Builder type and build it.
    pub fn new<GenerateChild>(
        root: ElementBuilder,
        generate_child: GenerateChild,
        initial: impl Iterator<Item = (usize, T)>,
    ) -> Self
    where
        GenerateChild: 'static + Fn(&T) -> Element,
    {
        let mut new = Self {
            storage: Rc::new(RefCell::new(Storage {
                root,
                items: BTreeMap::new(),
            })),
            generate_child: Rc::new(generate_child),
            items: BTreeMap::new(),
            filter: Box::new(|_| Signal::new(true).read()),
        };

        for (key, elem) in initial {
            new.insert(key, elem);
        }

        new
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn insert(&mut self, key: usize, item: T) {
        let item = Rc::new(item);
        let updater = self.updater(key, &item);

        self.items.insert(key, StoredItem { item, updater });
    }

    pub fn pop(&mut self) {
        if let Some((&key, _)) = self.items.iter().next_back() {
            self.items.remove(&key);
            self.storage.borrow_mut().remove(key);
        }
    }

    pub fn remove(&mut self, key: usize) {
        if self.items.remove(&key).is_some() {
            self.storage.borrow_mut().remove(key)
        }
    }

    pub fn filter(&mut self, f: impl 'static + Fn(&T) -> ReadSignal<bool>) {
        let old_items = mem::take(&mut self.items);
        self.filter = Box::new(f);

        for (key, StoredItem { item, updater }) in old_items {
            mem::drop(updater);
            let updater = self.updater(key, &item);
            self.items.insert(key, StoredItem { item, updater });
        }
    }

    fn updater(&self, key: usize, item: &Rc<T>) -> ReadSignal<()> {
        (self.filter)(&item).map({
            let storage = self.storage.clone();
            let item = item.clone();
            let generate_child = self.generate_child.clone();

            move |&visible| {
                if visible {
                    storage.borrow_mut().insert(key, generate_child(&item));
                } else {
                    storage.borrow_mut().remove(key);
                }
            }
        })
    }
}

impl<T> DomElement for ElementList<T> {
    type Target = dom::Element;

    fn dom_element(&self) -> Self::Target {
        self.storage.borrow_mut().root.dom_element()
    }
}
