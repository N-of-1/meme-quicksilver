/// Run a test where a Muse headset collects EEG data based on a series of
/// images presented to the wearer. Push that raw collected data to a Postgresql database.

// Draw some multi-colored geometry to the screen
#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
extern crate env_logger;

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
extern crate web_logger;

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
extern crate nannou_osc;

extern crate arr_macro;
extern crate num_traits;
extern crate quicksilver;

#[macro_use]
extern crate log;

use arr_macro::arr;
use muse_model::{DisplayType, MuseModel};
use quicksilver::{
    combinators::result,
    geom::{Line, Rectangle, Shape, Vector},
    graphics::{Background::Col, Background::Img, Color, Font, FontStyle, Image},
    input::{ButtonState, GamepadButton, Key, MouseButton},
    lifecycle::{run, Asset, Event, Settings, State, Window},
    sound::Sound,
    Future, Result,
};
use std::sync::mpsc::Receiver;
use std::time::Duration;

mod eeg_view;
mod muse_model;

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
mod muse_packet;

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
const SCREEN_SIZE: (f32, f32) = (1280.0, 768.0);
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
const SCREEN_SIZE: (f32, f32) = (1280.0, 650.0);

const FPS: u64 = 30;
const FRAME_TITLE: u64 = 1 * FPS; // 30 frames/sec
const FRAME_INTRO: u64 = FRAME_TITLE + 1 * FPS;
const FRAME_SETTLE: u64 = FRAME_INTRO + 12000 * FPS;
const FRAME_MEME: u64 = FRAME_SETTLE + 4 * FPS;

const IMAGE_LOGO: &str = "N_of_1_logo_blue_transparent.png";

const FONT_EXTRA_BOLD: &str = "WorkSans-ExtraBold.ttf";
const FONT_MULI: &str = "Muli.ttf";
const FONT_EXTRA_BOLD_SIZE: f32 = 72.0;
const FONT_MULI_SIZE: f32 = 40.0;
const FONT_EEG_LABEL_SIZE: f32 = 30.0;

const SOUND_CLICK: &str = "click.ogg";
const SOUND_BLAH: &str = "blah.ogg";

const STR_TITLE: &str = "Meme Machine";
const STR_HELP_TEXT: &str = "First relax and watch your mind calm\n\nYou will then be shown some images. Press the left and right images to tell us if they are\nfamiliar and how they make you feel.";

const COLOR_GREY: Color = Color {
    r: 0.5,
    g: 0.5,
    b: 0.5,
    a: 1.0,
};
const COLOR_CLEAR: Color = Color {
    r: 0.5,
    g: 0.5,
    b: 0.5,
    a: 0.0,
};
const COLOR_NOF1_DARK_BLUE: Color = Color {
    r: 31. / 256.,
    g: 18. / 256.,
    b: 71. / 256.,
    a: 1.0,
};
const COLOR_NOF1_LIGHT_BLUE: Color = Color {
    r: 189. / 256.,
    g: 247. / 256.,
    b: 255. / 256.,
    a: 1.0,
};
const COLOR_NOF1_TURQOISE: Color = Color {
    r: 0. / 256.,
    g: 200. / 256.,
    b: 200. / 256.,
    a: 1.0,
};
const COLOR_BACKGROUND: Color = COLOR_GREY;
const COLOR_TITLE: Color = COLOR_NOF1_DARK_BLUE;
const COLOR_EEG_LABEL: Color = Color::BLACK;
const COLOR_TEXT: Color = Color::BLACK;
const COLOR_BUTTON: Color = COLOR_NOF1_DARK_BLUE;
const COLOR_BUTTON_PRESSED: Color = COLOR_NOF1_LIGHT_BLUE;
const COLOR_EMOTION: Color = Color::YELLOW;

pub const EEG_LABELS: [&str; 5] = ["A", "B", "G", "D", "T"];

const BUTTON_WIDTH: f32 = 200.0;
const BUTTON_HEIGHT: f32 = 50.0;
const BUTTON_H_MARGIN: f32 = 20.0;
const BUTTON_V_MARGIN: f32 = 20.0;

const TITLE_V_MARGIN: f32 = 40.0;
const TEXT_V_MARGIN: f32 = 200.0;

const OSC_PORT: u16 = 34254; // Incoming Muse OSC UDP packets

const RECT_LEFT_BUTTON: Rectangle = Rectangle {
    pos: Vector {
        x: BUTTON_H_MARGIN,
        y: SCREEN_SIZE.1 - BUTTON_V_MARGIN - BUTTON_HEIGHT,
    },
    size: Vector {
        x: BUTTON_WIDTH,
        y: BUTTON_HEIGHT,
    },
};

const RECT_RIGHT_BUTTON: Rectangle = Rectangle {
    pos: Vector {
        x: SCREEN_SIZE.0 - BUTTON_H_MARGIN - BUTTON_WIDTH,
        y: SCREEN_SIZE.1 - BUTTON_V_MARGIN - BUTTON_HEIGHT,
    },
    size: Vector {
        x: BUTTON_WIDTH,
        y: BUTTON_HEIGHT,
    },
};

pub trait OscSocket: Sized {
    fn osc_socket_receive();
}

struct AppState {
    frame_count: u64,
    title_text: Asset<Image>,
    help_text: Asset<Image>,
    logo: Asset<Image>,

    sound_click: Asset<Sound>,
    sound_blah: Asset<Sound>,
    left_button_color: Color,
    right_button_color: Color,
    muse_model: MuseModel,
    rx_eeg: Receiver<(Duration, muse_model::MuseMessageType)>,
    calm_ext: ImageSet,
    neg_pos: ImageSet,
}

impl AppState {
    // Perform any shutdown actions
    // Do not call this directly to end the app. Instead call window.close();
    fn shutdown_hooks(&mut self) -> Result<()> {
        // TODO Notify database session ended

        Ok(())
    }

    fn left_action(&mut self, _window: &mut Window) -> Result<()> {
        self.left_button_color = COLOR_BUTTON_PRESSED;
        self.sound_click
            .execute(|sound| sound.play())
            .expect("Could not play left button sound");
        self.sound_blah.execute(|sound| sound.play())
    }

    fn right_action(&mut self, _window: &mut Window) -> Result<()> {
        self.right_button_color = COLOR_BUTTON_PRESSED;
        self.sound_click.execute(|sound| sound.play())
    }
}

impl State for AppState {
    fn new() -> Result<AppState> {
        let title_font = Font::load(FONT_EXTRA_BOLD);
        let help_font = Font::load(FONT_MULI);

        let title_text = Asset::new(title_font.and_then(|font| {
            result(font.render(
                STR_TITLE,
                &FontStyle::new(FONT_EXTRA_BOLD_SIZE, COLOR_TITLE),
            ))
        }));
        let help_text = Asset::new(help_font.and_then(|font| {
            result(font.render(STR_HELP_TEXT, &FontStyle::new(FONT_MULI_SIZE, COLOR_TEXT)))
        }));

        let logo = Asset::new(Image::load(IMAGE_LOGO));
        let sound_click = Asset::new(Sound::load(SOUND_CLICK));
        let sound_blah = Asset::new(Sound::load(SOUND_BLAH));
        let (rx_eeg, muse_model) = muse_model::MuseModel::new();

        let calm_ext = ImageSet::new("calm_ext");
        let neg_pos = ImageSet::new("neg_pos");

        Ok(AppState {
            frame_count: 0,
            title_text,
            help_text,
            logo,
            sound_click,
            sound_blah,
            left_button_color: COLOR_CLEAR,
            right_button_color: COLOR_CLEAR,
            muse_model,
            rx_eeg,
            calm_ext,
            neg_pos,
        })
    }

    // This is called 60 times per second
    fn update(&mut self, window: &mut Window) -> Result<()> {
        // EXIT APP
        #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
        {
            if window.keyboard()[Key::Escape].is_down()
                || window
                    .gamepads()
                    .iter()
                    .any(|pad| pad[GamepadButton::FaceLeft].is_down())
            {
                window.close();
            }
        }

        // LEFT SHIFT OR GAMEPAD ACTION
        if window.keyboard()[Key::LShift] == ButtonState::Pressed
            || window
                .gamepads()
                .iter()
                .any(|pad| pad[GamepadButton::TriggerLeft].is_down())
            || window
                .gamepads()
                .iter()
                .any(|pad| pad[GamepadButton::ShoulderLeft].is_down())
        {
            self.left_action(window)?;
        }

        // RIGHT SHIFT OR GAMEPAD ACTION
        if window.keyboard()[Key::RShift] == ButtonState::Pressed
            || window
                .gamepads()
                .iter()
                .any(|pad| pad[GamepadButton::TriggerRight].is_down())
            || window
                .gamepads()
                .iter()
                .any(|pad| pad[GamepadButton::ShoulderRight].is_down())
        {
            self.right_action(window)?;
        }

        // LEFT SCREEN BUTTON PRESS
        if window.mouse()[MouseButton::Left] == ButtonState::Pressed
            && RECT_LEFT_BUTTON.contains(window.mouse().pos())
        {
            self.left_action(window)?;
        }

        // RIGHT SCREEN BUTTON PRESS
        if window.mouse()[MouseButton::Left] == ButtonState::Pressed
            && RECT_RIGHT_BUTTON.contains(window.mouse().pos())
        {
            self.right_action(window)?;
        }

        // TODO NANO SEEED BUTTON PRESS

        // F1
        if window.keyboard()[Key::F1] == ButtonState::Pressed {
            self.muse_model.display_type = DisplayType::FourCircles;
        }

        // F2
        if window.keyboard()[Key::F2] == ButtonState::Pressed {
            self.muse_model.display_type = DisplayType::Dowsiness;
        }

        // F3
        if window.keyboard()[Key::F3] == ButtonState::Pressed {
            self.muse_model.display_type = DisplayType::Emotion;
        }

        // F4
        if window.keyboard()[Key::F4] == ButtonState::Pressed {
            self.muse_model.display_type = DisplayType::EegValues;
        }

        self.muse_model.receive_packets();
        self.muse_model.count_down();

        Ok(())
    }

    fn event(&mut self, event: &Event, _window: &mut Window) -> Result<()> {
        if let Event::Closed = event {
            self.shutdown_hooks()?;
        }

        Ok(())
    }

    // This is called 30 times per second
    fn draw(&mut self, window: &mut Window) -> Result<()> {
        window.clear(COLOR_BACKGROUND)?;

        if self.frame_count < FRAME_TITLE {
            // LOGO
            self.logo.execute(|image| {
                window.draw(
                    &image
                        .area()
                        .with_center((SCREEN_SIZE.0 / 2.0, SCREEN_SIZE.1 / 2.0)),
                    Img(&image),
                );
                Ok(())
            })?;
        } else if self.frame_count < FRAME_INTRO {
            // TITLE
            self.title_text.execute(|image| {
                window.draw(
                    &image
                        .area()
                        .with_center((SCREEN_SIZE.0 / 2.0, TITLE_V_MARGIN)),
                    Img(&image),
                );
                Ok(())
            })?;

            // TEXT
            self.help_text.execute(|image| {
                window.draw(
                    &image
                        .area()
                        .with_center((SCREEN_SIZE.0 / 2.0, TEXT_V_MARGIN)),
                    Img(&image),
                );
                Ok(())
            })?;

            // RIGHT BUTTON
            let right_color = self.right_button_color;
            self.sound_click.execute(|_| {
                window.draw(&RECT_RIGHT_BUTTON, Col(right_color));
                Ok(())
            })?;
            self.right_button_color = COLOR_BUTTON;
        } else if self.frame_count < FRAME_SETTLE {
            eeg_view::draw_view(&self.muse_model, window);
        } else if self.frame_count < FRAME_MEME {
            // LEFT BUTTON
            let left_color = self.left_button_color;
            self.sound_click.execute(|_| {
                window.draw(&RECT_LEFT_BUTTON, Col(left_color));
                Ok(())
            })?;
            self.left_button_color = COLOR_BUTTON;

            // RIGHT BUTTON
            let right_color = self.right_button_color;
            self.sound_click.execute(|_| {
                window.draw(&RECT_RIGHT_BUTTON, Col(right_color));
                Ok(())
            })?;
            self.right_button_color = COLOR_BUTTON;
        } else {
            // LOGO
            self.logo.execute(|image| {
                window.draw(
                    &image
                        .area()
                        .with_center((SCREEN_SIZE.0 / 2.0, SCREEN_SIZE.1 / 2.0)),
                    Img(&image),
                );
                Ok(())
            })?;
        }

        self.frame_count = self.frame_count + 1;
        if self.frame_count == std::u64::MAX {
            self.frame_count = 1;
        }

        Ok(())
    }

    fn handle_error(error: quicksilver::Error) {
        error!("Unhandled error: {:?}", error);
        panic!("Unhandled error: {:?}", error);
    }
}

const IMAGE_SET_SIZE: usize = 10;
struct ImageSet {
    images: [Asset<Image>; IMAGE_SET_SIZE],
}

fn filename(filename_prefix: &str, i: usize) -> String {
    let mut filename = String::new();
    filename.push_str(filename_prefix);
    filename.push_str(&format!("{}", i));
    filename.push_str(".png");

    filename
}

impl ImageSet {
    fn new(filename_prefix: &str) -> Self {
        let mut i: usize = 0;
        let mut images: [Asset<Image>; IMAGE_SET_SIZE] = arr![Asset::new(Image::load(filename(filename_prefix, {
                i += 1;
                i - 1
            }))); 10];

        // for i in 0..IMAGE_SET_SIZE {
        //     images[i] = Asset::new(Image::load(filename(filename_prefix, i)));
        // }

        Self { images }
    }
}

fn main() {
    use quicksilver::graphics::*;

    #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
    {
        env_logger::init();
    }

    #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
    {
        web_logger::init();
    }

    info!("meme_quicksilver start");

    let settings = Settings {
        icon_path: Some("n-icon.png"),
        fullscreen: true,
        resize: ResizeStrategy::Maintain,
        draw_rate: 35.0,          // 35ms ~= max 30fps
        update_rate: 1000. / 60., // 60 times per second
        ..Settings::default()
    };

    run::<AppState>(
        STR_TITLE,
        Vector::new(SCREEN_SIZE.0, SCREEN_SIZE.1),
        settings,
    )
}
