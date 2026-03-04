use dioxus::prelude::*;
use neuronet::App;

fn main() {
    console_error_panic_hook::set_once();
    launch(App);
}
