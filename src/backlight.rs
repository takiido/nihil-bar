use gtk4::{Box as GtkBox, Label, Orientation};
use gtk4::prelude::{BoxExt};
use std::fs;

const BACKLIGHT: &str = "/sys/class/backlight/";

pub fn create_widget() -> Result<GtkBox, Box<dyn std::error::Error>> {
    let brightness = get_brightness()?;
    let container = GtkBox::new(Orientation::Horizontal, 0);
    let label = Label::new(Some(brightness.to_string().as_str()));
    container.append(&label);

    Ok(container)
}

fn get_devices() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut devices: Vec<String> = Vec::new();
    for entry in fs::read_dir(BACKLIGHT)? {
        devices.push(entry?.file_name().into_string().unwrap());
    }

    Ok(devices)
}

fn get_max_brightness(device: &str) -> Result<u32, Box<dyn std::error::Error>> {
    Ok(fs::read_to_string(format!("{}{}/max_brightness", BACKLIGHT, device))?
        .trim()
        .parse::<u32>()?
    )
}

fn get_current_brightness(device: &str) -> Result<u32, Box<dyn std::error::Error>> {
    Ok(fs::read_to_string(format!("{}{}/brightness", BACKLIGHT, device))?
        .trim()
        .parse::<u32>()?
    )
}

fn get_brightness() -> Result<u32, Box<dyn std::error::Error>> {
    let devices = get_devices()?;
    let device = devices.first().ok_or("No backlight device found")?;
    let max_brightness = get_max_brightness(device)?;
    let current_brightness = get_current_brightness(device)?;

    Ok((current_brightness as f32 / max_brightness as f32 * 100.0).round() as u32)
}