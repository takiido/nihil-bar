use std::rc::Rc;
use std::thread;
use std::cell::RefCell;
use std::process::Command;
use pipewire::types::ObjectType;
use pipewire::{main_loop::MainLoopRc, context::ContextRc};
use gtk4::prelude::{WidgetExt, RangeExt, ButtonExt};
use gtk4::{Box as GtkBox, Label, Orientation, Scale};

#[derive(Debug, thiserror::Error)]
pub enum AudioError {
    #[error("no audio device found")]
    NoDevice,
    #[error("io error: {0}")]
    Io(#[from] pipewire::Error),
    #[error("parse error: {0}")]
    Parse(#[from] std::num::ParseIntError),
    #[error("failed to set volume")]
    SetVolume,
}

pub fn create_widget() -> Result<GtkBox, AudioError> {
    let (volume, muted) = get_volume().unwrap_or((50, false));

    let container = GtkBox::new(Orientation::Horizontal, 0);

    let mute_btn = gtk4::Button::new();
    update_icon(&mute_btn, volume, muted); // set initial icon
    mute_btn.set_parent(&container);

    let slider = Scale::with_range(Orientation::Horizontal, 0.0, 100.0, 1.0);
    slider.set_css_classes(&["slider"]);
    slider.set_value(volume as f64);
    slider.set_parent(&container);

    let label = Label::new(Some(&format!("{}%", volume)));
    label.set_parent(&container);

    // Mute button click
    let mute_btn_clone = mute_btn.clone();
    mute_btn.connect_clicked(move |_| {
        let (vol, muted) = get_volume().unwrap_or((50, false));
        set_mute(!muted).expect("Failed to toggle mute");
        let (new_vol, new_muted) = get_volume().unwrap_or((50, false));
        update_icon(&mute_btn_clone, new_vol, new_muted);
    });

    let label_clone = label.clone();
    slider.connect_value_changed(move |scale| {
        let vol = scale.value() as u32;
        label_clone.set_text(&format!("{}%", vol));
        set_volume(vol).expect("Failed to set volume");
    });

    let (sender, receiver) = async_channel::unbounded::<(u32, bool)>();
    thread::spawn(move || {
        watch_default_dev(sender).expect("Failed to watch default device");
    });

    let slider_clone = slider.clone();
    let label_clone2 = label.clone();
    let mute_btn_clone2 = mute_btn.clone();
    glib::MainContext::default().spawn_local(async move {
        while let Ok((vol, muted)) = receiver.recv().await {
            slider_clone.set_value(vol as f64);
            label_clone2.set_text(&format!("{}%", vol));
            update_icon(&mute_btn_clone2, vol, muted);
        }
    });

    Ok(container)
}

fn watch_default_dev(sender: async_channel::Sender<(u32, bool)>) -> Result<(), AudioError> {
    let mainloop = MainLoopRc::new(None)?;
    let context = ContextRc::new(&mainloop, None)?;
    let core = context.connect_rc(None)?;
    let registry = core.get_registry_rc()?;
    let registry_clone = registry.clone();

    let alive: Rc<RefCell<Vec<Box<dyn std::any::Any>>>> = Rc::new(RefCell::new(Vec::new()));
    let alive_clone = alive.clone();

    let _listener = registry
        .add_listener_local()
        .global(move |global| {
            if global.type_ != ObjectType::Metadata {
                return;
            }

            let props = global.props.as_ref().map(|p| p.as_ref());
            if let Some(p) = props {
                let name = p.get("metadata.name").unwrap_or("");
                if name != "default" {
                    return;
                }
            }

            let metadata: pipewire::metadata::Metadata = registry_clone.bind(global).unwrap();
            let sender_clone = sender.clone();
            let meta_listener = metadata
                .add_listener_local()
                .property(move |_subject, key, _type, value| {
                    match key {
                        Some("default.audio.sink") | Some("default.audio.source") => {
                            if let Ok(volume) = get_volume() {
                                sender_clone.send_blocking(volume).expect("Failed to send volume");
                            }
                        }
                        _ => {}
                    }
                    0
                })
                .register();

            alive_clone.borrow_mut().push(Box::new(meta_listener));
            alive_clone.borrow_mut().push(Box::new(metadata));
        })
        .register();

    mainloop.run();

    Ok(())
}

fn get_volume() -> Result<(u32, bool), AudioError> {
    let output = Command::new("wpctl")
        .args(["get-volume", "@DEFAULT_SINK@"])
        .output()
        .map_err(|_| AudioError::SetVolume)?;

    if !output.status.success() {
        return Err(AudioError::SetVolume);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let muted = stdout.contains("[MUTED]");
    let volume = stdout
        .trim()
        .split_whitespace()
        .nth(1)
        .unwrap_or("0")
        .parse::<f32>()
        .unwrap_or(0.0);

    Ok(((volume * 100.0) as u32, muted))
}

fn set_volume(volume: u32) -> Result<(), AudioError> {
    let output = Command::new("wpctl")
        .args(["set-volume", "@DEFAULT_SINK@", &format!("{}%", volume)])
        .output()
        .map_err(|_| AudioError::SetVolume)?;

    if !output.status.success() {
        return Err(AudioError::SetVolume);
    }

    Ok(())
}

fn set_mute(mute: bool) -> Result<(), AudioError> {
    let output = Command::new("wpctl")
        .args(["set-mute", "@DEFAULT_SINK@", &format!("{}", mute)])
        .output()
        .map_err(|_| AudioError::SetVolume)?;

    if !output.status.success() {
        return Err(AudioError::SetVolume);
    }

    Ok(())
}

fn update_icon(button: &gtk4::Button, volume: u32,  muted: bool) {
    let icon;
    if muted {
        icon = "";
    } else {
        match volume {
            0..=20 =>  icon = "",
            21..=74 =>  icon = "",
            75..=100 =>  icon = "",
            _ => icon = ""
        }
    }
    button.set_label(icon);
}
