use dioxus::{logger::tracing, prelude::*};
use neuronet::App;
use tracing::Level;

fn main() {
    console_error_panic_hook::set_once();
    dioxus::logger::init(Level::INFO).expect("failed to init logger");
    launch(App);
}
