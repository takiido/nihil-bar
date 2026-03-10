use gtk4::{Box as GtkBox, Label, Orientation};
use gtk4::prelude::{BoxExt};
use std::fs;
use std::sync::OnceLock;

#[derive(Debug, thiserror::Error)]
pub enum BacklightError {
    #[error("no backlight device found")]
    NoDevice,
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("parse error: {0}")]
    Parse(#[from] std::num::ParseIntError),
}

const BACKLIGHT: &str = "/sys/class/backlight/";

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
    let label = Label::new(Some(
        &format!(
            "{}%", brightness
        )
    ));
    container.append(&label);

    Ok(container)
}

fn get_devices() -> Result<Vec<String>, BacklightError> {
    let mut devices: Vec<String> = Vec::new();
    for entry in fs::read_dir(BACKLIGHT)? {
        devices.push(entry?.file_name().display().to_string());
    }

    Ok(devices)
}

fn get_max_brightness(device: &str) -> Result<u32, BacklightError> {
    if let Some(&v) = MAX_BRIGHTNESS.get() {
        return Ok(v);
    }
    let v = fs::read_to_string(format!("{}{}/max_brightness", BACKLIGHT, device))?
        .trim()
        .parse::<u32>()?;
    MAX_BRIGHTNESS.set(v).ok();
    Ok(v)
}

fn get_current_brightness(device: &str) -> Result<u32, BacklightError> {
    Ok(fs::read_to_string(format!("{}{}/brightness", BACKLIGHT, device))?
        .trim()
        .parse::<u32>()?
    )
}

fn get_brightness() -> Result<u32, BacklightError> {
    let devices = get_devices()?;
    let device = devices.first().ok_or(BacklightError::NoDevice)?;
    let max_brightness = get_max_brightness(device)?;
    let current_brightness = get_current_brightness(device)?;

    Ok((current_brightness as f32 / max_brightness as f32 * 100.0).round() as u32)
}