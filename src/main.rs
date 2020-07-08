use std::thread;

use eye::hal::traits::Device;
use eye::prelude::*;

use gdk_pixbuf::{Colorspace, Pixbuf};
use glib::Bytes;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, Image};

pub struct DeviceWrapper {
    dev: Box<dyn Device>,
}

unsafe impl Send for DeviceWrapper {}

fn main() {
    if gtk::init().is_err() {
        println!("Failed to initialize GTK.");
        return;
    }

    let mut dev = DeviceFactory::create(0).unwrap();
    let mut format = dev.get_format().unwrap();
    format.width = 1280;
    format.height = 720;
    format.fourcc = FourCC::new(b"RGB3");
    format = dev.set_format(&format).unwrap();

    if format.fourcc != FourCC::new(b"RGB3") {
        panic!("Failed to set RGB format.");
    }

    let application = Application::new(Some("org.raymanfx.eye-gtk"), Default::default())
        .expect("Failed to initialize GTK.");
    let window = ApplicationWindow::new(&application);
    window.set_title("Eye");
    window.set_default_size(format.width as i32, format.height as i32);

    let img = Image::new();
    window.add(&img);
    window.show_all();

    let (tx, rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
    let mut dev_wrapper = DeviceWrapper { dev };

    thread::spawn(move || {
        // create an image capture stream
        let mut stream = dev_wrapper.dev.stream().unwrap();

        loop {
            // fetch a new frame
            let frame = stream.next().unwrap();

            // convert into a glib buffer by copying the bytes
            let bytes = Bytes::from(frame.raw().as_slice().unwrap());

            // send it to the main thread
            tx.send(bytes)
                .expect("Failed to send frame to main thread.");
        }
    });

    rx.attach(None, move |bytes| {
        let pixbuf = Pixbuf::new_from_bytes(
            &bytes,
            Colorspace::Rgb,
            false, /* has_alpha */
            8,     /* bits per sample */
            format.width as i32,
            format.height as i32,
            format.width as i32 * 3 /* stride */,
        );
        img.set_from_pixbuf(Some(&pixbuf));
        glib::Continue(true)
    });

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    gtk::main();
}
