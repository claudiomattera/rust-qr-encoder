use gtk::prelude::*;
use gio::prelude::*;

use gtk::{Application, ApplicationWindow, Builder, ComboBoxText, Entry, Image, Label, SpinButton};

use gdk_pixbuf::{Colorspace, Pixbuf};

use std::env::args;

use log::*;

use clap::{app_from_crate, crate_authors, crate_description, crate_name, crate_version};
use clap::{Arg, ArgMatches};

use qrcode::{QrCode, EcLevel};

use image::Luma;

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

    application.run(&args().collect::<Vec<_>>());
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
    let mode_combobox: ComboBoxText = builder
        .get_object("mode_combobox").
        expect("Couldn't get mode_combobox");
    let size_spinbutton: SpinButton = builder
        .get_object("size_spinbutton").
        expect("Couldn't get size_spinbutton");
    let error_label: Label = builder
        .get_object("error_label").
        expect("Couldn't get error_label");

    window.set_title("QR Encoder");
    mode_combobox.set_sensitive(false);

    text_entry.connect_changed(move |entry| {
        error_label.set_text("");

        let text: String = entry.get_text().unwrap().to_string();
        let error_correction: String = error_correction_combobox.get_active_text().unwrap().to_string();
        let mode: String = mode_combobox.get_active_text().unwrap().to_string();
        let size = size_spinbutton.get_value() as u32;
        debug!("Text: \"{}\"", text);
        debug!("Error correction: {}, mode: {}, size: {}", error_correction, mode, size);

        let error_correction = match error_correction.as_str() {
            "High (30%)" => EcLevel::H,
            "Quartile (25%)" => EcLevel::Q,
            "Medium (15%)" => EcLevel::M,
            "Low (7%)" => EcLevel::L,
            _ => unreachable!(),
        };

        match QrCode::with_error_correction_level(text, error_correction) {
            Ok(qr) => {
                let image = qr
                    .render()
                    .dark_color(Luma([0]))
                    .light_color(Luma([255]))
                    .quiet_zone(true)
                    .min_dimensions(size, size)
                    .build();

                let (n, m) = image.dimensions();

                let pixbuf: Pixbuf = Pixbuf::new(
                    Colorspace::Rgb,
                    false,
                    8,
                    n as i32,
                    m as i32,
                ).expect("Could not create pixbuf");

                for (x, y, value) in image.enumerate_pixels() {
                    let (red, green, blue, alpha) = value_to_color(*value);
                    pixbuf.put_pixel(x as i32, y as i32, red, green, blue, alpha);
                }

                qr_image.set_from_pixbuf(Some(&pixbuf));
            }
            Err(error) => {
                let message = format!("Error: {}", error);
                warn!("Error: {}", error);
                error_label.set_text(&message);
            }
        }
    });

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

fn value_to_color(value: Luma<u8>) -> (u8, u8, u8, u8) {
    let [red] = value.0;
    let [green] = value.0;
    let [blue] = value.0;
    let alpha = 255;
    (red, green, blue, alpha)
}
