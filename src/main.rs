use std::mem;

use eye::hal::traits::{Device, Stream};
use eye::prelude::*;

use ffimage::packed::{DynamicImageBuffer, DynamicImageView};

use iced::widget::image::Handle as ImageHandle;
use iced::{
    executor, time, Application, Command, Container, Element, Image, Settings, Subscription,
};

fn main() {
    Eyece::run(Settings::default())
}

struct Eyece<'a> {
    device: Option<Box<dyn Device>>,
    stream: Option<Box<dyn Stream<Item = DynamicImageView<'a>>>>,

    handle: Option<ImageHandle>,
    dim: (u16, u16),
}

#[derive(Debug, Clone, Copy)]
enum Message {
    Open(usize),
    Capture,
}

impl<'a> Application for Eyece<'a> {
    type Executor = executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: ()) -> (Self, Command<Message>) {
        let mut pixels = Vec::new();
        let res = (1000, 1000);
        pixels.resize((res.0 * res.1) as usize * 4, 0);
        let handle = ImageHandle::from_pixels(1000, 1000, pixels);

        let mut dev = DeviceFactory::create(0).unwrap();
        let mut format = dev.get_format().unwrap();
        format.width = 1280;
        format.height = 720;
        format.fourcc = FourCC::new(b"AB24");
        format = dev.set_format(&format).unwrap();
        if format.fourcc != FourCC::new(b"AB24") {
            println!("got fmt: {}", format.fourcc);
            panic!("NO SUITABLE FORMAT");
        }

        let mut ret = (
            Eyece {
                device: Some(dev),
                stream: None,
                handle: Some(handle),
                dim: (0, 0),
            },
            Command::none(),
        );

        unsafe {
            ret.0.stream = mem::transmute(ret.0.device.as_mut().unwrap().stream().unwrap());
        }

        ret
    }

    fn title(&self) -> String {
        String::from("Eye (iced)")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Open(index) => {
                let mut dev = DeviceFactory::create(index).unwrap();
                let mut format = dev.get_format().unwrap();
                format.width = 1280;
                format.height = 720;
                format.fourcc = FourCC::new(b"AB24");
                format = dev.set_format(&format).unwrap();
                if format.fourcc == FourCC::new(b"AB24") {
                    self.device = Some(dev);
                    unsafe {
                        self.stream = Some(mem::transmute(
                            self.device.as_mut().unwrap().stream().unwrap(),
                        ));
                    }
                }

                self.dim = (format.width as u16, format.height as u16);
            }
            Message::Capture => {
                let handle: ImageHandle;
                let res: (u32, u32);
                if let Some(stream) = &mut self.stream {
                    println!("[MSG] got real frame");
                    let image = stream.next().unwrap();
                    res = (image.width(), image.height());
                    handle = ImageHandle::from_pixels(
                        res.0,
                        res.1,
                        image.raw().as_slice().unwrap().to_vec(),
                    );
                } else {
                    let mut pixels = Vec::new();
                    res = (1000, 1000);
                    pixels.resize((res.0 * res.1) as usize * 4, 0);
                    handle = ImageHandle::from_pixels(1000, 1000, pixels);
                }
                self.handle = Some(handle);
                println!("[MSG] Capture");
            }
        }

        Command::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        time::every(std::time::Duration::from_millis(30)).map(|_| Message::Capture)
    }

    fn view(&mut self) -> Element<Message> {
        println!("[FN] {}", "view");
        let image = Image::new(self.handle.as_ref().unwrap().clone());
        Container::new(image)
            .width(self.dim.0.into())
            .height(self.dim.1.into())
            .center_x()
            .center_y()
            .into()
    }
}
