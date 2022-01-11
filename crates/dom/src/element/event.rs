use wasm_bindgen::{prelude::Closure, JsCast, JsValue, UnwrapThrowExt};

pub struct EventCallback {
    target: web_sys::Element,
    name: &'static str,
    callback: Closure<dyn FnMut(JsValue)>,
}

impl EventCallback {
    pub fn new(
        target: web_sys::Element,
        name: &'static str,
        f: impl FnMut(JsValue) + 'static,
    ) -> Self {
        let callback = Closure::wrap(Box::new(f) as Box<dyn FnMut(JsValue)>);
        target
            .add_event_listener_with_callback(name, callback.as_ref().unchecked_ref())
            .unwrap_throw();

        Self {
            target,
            name,
            callback,
        }
    }
}

impl Drop for EventCallback {
    fn drop(&mut self) {
        self.target
            .remove_event_listener_with_callback(
                self.name,
                self.callback.as_ref().as_ref().unchecked_ref(),
            )
            .unwrap_throw();
    }
}
