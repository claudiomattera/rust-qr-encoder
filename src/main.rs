use gtk::prelude::*;
use gio::prelude::*;

use gtk::{
    Application, ApplicationWindow, Builder, ComboBoxText, Entry,  FileChooserAction,
    FileChooserDialogBuilder, FileFilter, Image, Label, ResponseType, SpinButton, ToolButton
};

use glib::Bytes;

use gdk::ModifierType;

use gdk_pixbuf::{Colorspace, Pixbuf};

use std::fs::File;
use std::io::Write;

use log::*;

use clap::{app_from_crate, crate_authors, crate_description, crate_name, crate_version};
use clap::{Arg, ArgMatches};

use qrcode::{QrCode, EcLevel};

use image::Rgb;

fn main() {
    let arguments = parse_arguments();
    setup_logging(arguments.occurrences_of("verbosity"));

    let application = Application::new(
        Some("it.mattera.claudio.qr-encoder"),
        Default::default(),
    ).expect("failed to initialize GTK application");

    application.connect_activate(|app| {
        build_ui(app);
    });

    // Run with empty command-line arguments
    application.run(&[]);
}

fn build_ui(application: &gtk::Application) {
    let glade_src = include_str!("window.glade");
    let builder = Builder::new_from_string(glade_src);

    let window: ApplicationWindow = builder
        .get_object("window")
        .expect("Couldn't get window");
    window.set_application(Some(application));

    let text_entry: Entry = builder
        .get_object("text_entry")
        .expect("Couldn't get text_entry");
    let qr_image: Image = builder
        .get_object("qr_image").
        expect("Couldn't get qr_image");
    let error_correction_combobox: ComboBoxText = builder
        .get_object("error_correction_combobox").
        expect("Couldn't get error_correction_combobox");
    let size_spinbutton: SpinButton = builder
        .get_object("size_spinbutton").
        expect("Couldn't get size_spinbutton");
    let error_label: Label = builder
        .get_object("error_label").
        expect("Couldn't get error_label");
    let save_button: ToolButton = builder
        .get_object("save_button").
        expect("Couldn't get save_button");

    window.set_title("QR Encoder");

    debug!("Add text entry callback");
    {
        let text_entry = text_entry.clone();
        let error_correction_combobox = error_correction_combobox.clone();
        let size_spinbutton = size_spinbutton.clone();
        let error_label = error_label.clone();
        let qr_image = qr_image.clone();

        text_entry.connect_changed(move |text_entry| {
            match on_text_changed(&text_entry, &error_correction_combobox, &size_spinbutton, &error_label, &qr_image) {
                Ok(_) => (),
                Err(error) => {
                    let message = format!("Error: {}", error);
                    error!("Error: {}", error);
                    error_label.set_text(&message);
                }
            }
        });
    }

    debug!("Add error correction level combobox callback");
    {
        let text_entry = text_entry.clone();
        let error_correction_combobox = error_correction_combobox.clone();
        let size_spinbutton = size_spinbutton.clone();
        let error_label = error_label.clone();
        let qr_image = qr_image.clone();

        error_correction_combobox.connect_changed(move |error_correction_combobox| {
            match on_text_changed(&text_entry, &error_correction_combobox, &size_spinbutton, &error_label, &qr_image) {
                Ok(_) => (),
                Err(error) => {
                    let message = format!("Error: {}", error);
                    error!("Error: {}", error);
                    error_label.set_text(&message);
                }
            }
        });
    }

    debug!("Add size spinbutton callback");
    {
        let text_entry = text_entry.clone();
        let error_correction_combobox = error_correction_combobox.clone();
        let size_spinbutton = size_spinbutton.clone();
        let error_label = error_label.clone();
        let qr_image = qr_image.clone();

        size_spinbutton.connect_changed(move |size_spinbutton| {
            match on_text_changed(&text_entry, &error_correction_combobox, &size_spinbutton, &error_label, &qr_image) {
                Ok(_) => (),
                Err(error) => {
                    let message = format!("Error: {}", error);
                    error!("Error: {}", error);
                    error_label.set_text(&message);
                }
            }
        });
    }

    debug!("Add save button callback");
    {
        let qr_image = qr_image.clone();
        let error_label = error_label.clone();

        save_button.connect_clicked(move |_| {
            match on_save_button_clicked(&qr_image) {
                Ok(_) => (),
                Err(error) => {
                    let message = format!("Error: {}", error);
                    error!("Error: {}", error);
                    error_label.set_text(&message);
                }
            }
        });
    }

    debug!("Add shortcut bindings");
    {
        let application = application.clone();

        let qr_image = qr_image.clone();
        let error_label = error_label.clone();

        window.connect_key_press_event(move |_window, gdk| {
            match gdk.get_keyval() {
                key if key == 's' as u32 && gdk.get_state().contains(ModifierType::CONTROL_MASK) => {
                    match on_save_button_clicked(&qr_image) {
                        Ok(_) => (),
                        Err(error) => {
                            let message = format!("Error: {}", error);
                            error!("Error: {}", error);
                            error_label.set_text(&message);
                        }
                    }
                }
                key if key == 'q' as u32 && gdk.get_state().contains(ModifierType::CONTROL_MASK) => {
                    application.quit();
                }
                _ => (),
            }
            Inhibit(false)
        });
    }

    debug!("Generating the initial QR code for empty text");
    {
        match on_text_changed(&text_entry, &error_correction_combobox, &size_spinbutton, &error_label, &qr_image) {
            Ok(_) => (),
            Err(error) => {
                let message = format!("Error: {}", error);
                error!("Error: {}", error);
                error_label.set_text(&message);
            }
        }
    }
    window.show_all();
}

fn parse_arguments() -> ArgMatches<'static> {
    app_from_crate!()
        .arg(
            Arg::with_name("verbosity")
                .short("v")
                .long("verbose")
                .multiple(true)
                .help("Sets the level of verbosity"),
        )
        .get_matches()
}

fn setup_logging(verbosity: u64) {
    let default_log_filter = match verbosity {
        0 => "warn",
        1 => "info",
        2 => "info,rust_qr_encoder=debug",
        3 | _ => "debug",
    };
    let filter = env_logger::Env::default().default_filter_or(default_log_filter);
    env_logger::Builder::from_env(filter).format_timestamp(None).init();
}

fn on_text_changed(
            entry: &Entry,
            error_correction_combobox: &ComboBoxText,
            size_spinbutton: &SpinButton,
            error_label: &Label,
            qr_image: &Image,
        ) -> anyhow::Result<()> {
    error_label.set_text("");

    let text: String = entry.get_text().unwrap().to_string();
    let error_correction: String = error_correction_combobox.get_active_text().unwrap().to_string();
    let size = size_spinbutton.get_value() as u32;
    debug!("Text: \"{}\"", text);
    debug!("Error correction: {}, size: {}", error_correction, size);

    let error_correction = match error_correction.as_str() {
        "High (30%)" => EcLevel::H,
        "Quartile (25%)" => EcLevel::Q,
        "Medium (15%)" => EcLevel::M,
        "Low (7%)" => EcLevel::L,
        _ => unreachable!(),
    };

    let qr = QrCode::with_error_correction_level(text, error_correction)?;
    let image = qr
        .render()
        .dark_color(Rgb([0, 0, 0]))
        .light_color(Rgb([255, 255, 255]))
        .quiet_zone(true)
        .min_dimensions(size, size)
        .build();

    let (n, m) = image.dimensions();

    let raw: Vec<u8> = image.into_raw();
    let bytes = Bytes::from(&raw);

    let pixbuf: Pixbuf = Pixbuf::new_from_bytes(
        &bytes,
        Colorspace::Rgb,
        false,
        8,
        n as i32,
        m as i32,
        3*n as i32,
    );

    qr_image.set_from_pixbuf(Some(&pixbuf));

    Ok(())
}

fn on_save_button_clicked(qr_image: &Image) -> anyhow::Result<()> {
    let filter = FileFilter::new();
    filter.add_pixbuf_formats();

    let dialog = FileChooserDialogBuilder::new()
        .action(FileChooserAction::Save)
        .filter(&filter)
        .do_overwrite_confirmation(true)
        .build();

    dialog.add_button("Cancel", ResponseType::Cancel);
    dialog.add_button("Save", ResponseType::Ok);

    dialog.run();
    dialog.hide();

    if let Some(path) = dialog.get_filename() {
        debug!("User selected path \"{}\"", path.display());
        let (path, type_) = match path.extension().map(|ext| ext.to_str().expect("Invalid extension")) {
            Some("png") => (path, "png"),
            Some("jpg") => (path, "jpeg"),
            Some("jpeg") => (path, "jpeg"),
            Some("bmp") => (path, "bmp"),
            _ => {
                (path.with_extension("png"), "png")
            }
        };
        debug!("Saving {} image to path: {}", type_, path.display());
        let buffer = qr_image
            .get_pixbuf()
            .ok_or_else(|| anyhow::anyhow!("Missing pixbuf"))?
            .save_to_bufferv(type_, &[])?;

        let mut file = File::create(path)?;
        file.write_all(&buffer)?;
    }

    Ok(())
}
