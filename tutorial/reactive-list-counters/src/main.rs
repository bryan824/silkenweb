use silkenweb::{
    element_list::OrderedElementList,
    elements::{button, div, Button},
    mount,
    signal::Signal,
    Builder,
};
use silkenweb_tutorial_common::render_counter;

fn main() {
    let list = Signal::new(OrderedElementList::new(div()));

    mount(
        "app",
        div()
            .child(pop_button(&list))
            .text(list.read().map(|list| format!("{}", list.len())))
            .child(push_button(&list))
            .child(list.read()),
    );
}

fn push_button(list: &Signal<OrderedElementList<usize>>) -> Button {
    let push_elem = list.write();
    button()
        .on_click(move |_, _| {
            push_elem
                .mutate(move |list| list.insert(list.len(), render_counter(&Signal::new(0)).into()))
        })
        .text("+")
        .build()
}

fn pop_button(list: &Signal<OrderedElementList<usize>>) -> Button {
    let pop_elem = list.write();
    button()
        .on_click(move |_, _| {
            pop_elem.mutate(move |list| {
                if !list.is_empty() {
                    list.remove(&(list.len() - 1))
                }
            })
        })
        .text("-")
        .build()
}
