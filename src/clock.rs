use gtk4::{glib, Box, Label, Orientation};
use gtk4::prelude::WidgetExt;

pub fn create_widget() -> Box {
    let container = Box::new(Orientation::Horizontal, 0);
    let label  = Label::new(None);
    label.set_parent(&container);
    schedule_clock(label);

    container
}

fn update_clock(label: &Label) {
    let now = glib::DateTime::now_local().unwrap();
    let text = now.format("%-I:%M%p").unwrap();
    label.set_text(text.as_str());
}

fn schedule_clock(label: Label) {
    let now = glib::DateTime::now_local().unwrap();

    let secs_remaining = 59 - now.second() as u64;
    let us_remaining = 1_000_000 - now.microsecond() as u64;
    let ms_until_next_minute = secs_remaining * 1000 + us_remaining / 1000;

    update_clock(&label);

    glib::timeout_add_local_once(
        std::time::Duration::from_millis(ms_until_next_minute),
        move || {
            update_clock(&label);
            glib::timeout_add_seconds_local(60, move || {
                update_clock(&label);
                glib::ControlFlow::Continue
            });
        }
    );
}