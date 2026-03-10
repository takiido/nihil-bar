mod clock;
mod backlight;

use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow};
use gtk4_layer_shell::{Layer, LayerShell, Edge};

fn main() {
    let app = Application::builder()
        .application_id("com.example.layer-shell-app")
        .build();

    app.connect_activate(create_window);
    app.run();
}

fn create_window(app: &Application) {
    let window = ApplicationWindow::builder()
        .application(app)
        .build();

    window.init_layer_shell();
    window.set_namespace(Some("necroshell"));
    window.set_layer(Layer::Overlay);
    window.set_anchor(Edge::Top, true);
    window.set_anchor(Edge::Left, true);
    window.set_anchor(Edge::Right, true);
    window.auto_exclusive_zone_enable();

    add_widgets(&window);
    
    window.present();
}

fn add_widgets(window: &ApplicationWindow) {
    let wrapper = gtk4::Box::new(gtk4::Orientation::Horizontal, 0);

    let clock = clock::create_widget();
    clock.set_parent(&wrapper);

    /*
     * Optional widgets.
     * These widgets will be created based on the system shell is installed on
     */
    match backlight::create_widget() {
        Ok(w) => w.set_parent(&wrapper),
        Err(e) => eprintln!("Backlight widget is not supported on this system: {}", e),
    }

    window.set_child(Some(&wrapper));
}