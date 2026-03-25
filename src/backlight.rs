use std::fs;
use std::sync::OnceLock;
use std::process::Command;
use std::os::unix::io::AsRawFd;
use udev::{MonitorBuilder};
use gtk4::prelude::{RangeExt, WidgetExt};
use gtk4::{Box as GtkBox, Label, Orientation, Scale};

static MAX_BRIGHTNESS : OnceLock<u32> = OnceLock::new();

/// Creates a GTK widget to display the current screen brightness as a percentage.
///
/// # Returns
///
/// * `Ok(GtkBox)` - A horizontal `GtkBox` containing a label displaying the brightness percentage.
/// * `Err(BacklightError)` - Returns an error if fetching the brightness fails.
///
/// # Errors
///
/// This function will return a `BacklightError` if the `get_brightness` function
/// fails to retrieve the current brightness level.
pub fn create_widget() -> Result<GtkBox, BacklightError> {
    let brightness = get_brightness()?;
    let container = GtkBox::new(Orientation::Horizontal, 0);
    let slider = Scale::with_range(
        Orientation::Horizontal,
        0.0,
        100.0,
        1.0
    );
    slider.set_css_classes(&["slider"]);
    slider.connect_value_changed(move |scale| {
        let brightness = (scale.value()) as u32;
        set_brightness(brightness).expect("Failed to set brightness");
    });
    slider.set_value(brightness as f64);
    slider.set_parent(&container);
    let label = Label::new(Some(
        &format!(
            "{}%", brightness
        )
    ));
    label.set_parent(&container);

    watch_brightness(label)?;

    Ok(container)
}