/// Run a test where a Muse headset collects EEG data based on a series of
/// images presented to the wearer. Push that raw collected data to a Postgresql database.
#[macro_use]
extern crate log;

// Draw some multi-colored geometry to the screen
#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
extern crate env_logger;

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
extern crate web_logger;

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
extern crate nannou_osc;

extern crate arr_macro;
extern crate mandala_quicksilver;
extern crate num_traits;
extern crate quicksilver;

use arr_macro::arr;
use csv::Writer;
use eeg_view::EegViewState;
use log::{error, info};
use mandala_quicksilver::{Mandala, MandalaState};
use muse_model::{DisplayType, MuseModel};
use quicksilver::{
    combinators::result,
    geom::{Line, Rectangle, Shape, Transform, Vector},
    graphics::{
        Background::Col, Background::Img, Color, Font, FontStyle, Image, Mesh, ShapeRenderer,
    },
    input::{ButtonState, GamepadButton, Key, MouseButton},
    lifecycle::{run, Asset, Event, Settings, State, Window},
    sound::Sound,
    Future, Result,
};
use std::error::Error;
use std::fs::File;
use std::sync::mpsc::Receiver;
use std::time::{Duration, Instant};

mod eeg_view;
mod muse_model;

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
mod muse_packet;

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
const SCREEN_SIZE: (f32, f32) = (1920.0, 1200.0);
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
const SCREEN_SIZE: (f32, f32) = (1280.0, 650.0);

const MANDALA_CENTER: (f32, f32) = (SCREEN_SIZE.0 / 2.0, SCREEN_SIZE.1 / 2.0);
const MANDALA_SCALE: (f32, f32) = (3.0, 3.0); // Adjust size of Mandala vs screen

const FPS: u64 = 60; // Frames per second
const UPS: u64 = 60; // Updates per second
const FRAME_TITLE: u64 = 4 * FPS;
const FRAME_INTRO: u64 = FRAME_TITLE + 1 * FPS;
const FRAME_SETTLE: u64 = FRAME_INTRO + 12000 * FPS;
const FRAME_MEME: u64 = FRAME_SETTLE + 4 * FPS;

const IMAGE_LOGO: &str = "Nof1-logo.png";
const MANDALA_VALENCE_PETAL_SVG_NAME: &str = "mandala_valence_petal.svg";
const MANDALA_AROUSAL_PETAL_SVG_NAME: &str = "mandala_arousal_petal.svg";
/// The visual slew time from current value to newly set value. Keep in mind that the newly set value is already smoothed, so this number should be small to provide consinuous interpolation between new values, not large to provide an additional layer of (less carefully controlled) smoothing filter.
const MANDALA_TRANSITION_DURATION: f32 = 0.5;

const FONT_EXTRA_BOLD: &str = "WorkSans-ExtraBold.ttf";
const FONT_MULI: &str = "Muli.ttf";
const FONT_EXTRA_BOLD_SIZE: f32 = 72.0;
const FONT_MULI_SIZE: f32 = 40.0;
const FONT_GRAPH_LABEL_SIZE: f32 = 40.0;
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
const COLOR_EEG_LABEL: Color = COLOR_NOF1_DARK_BLUE;
const COLOR_TEXT: Color = Color::BLACK;
const COLOR_BUTTON: Color = COLOR_NOF1_DARK_BLUE;
const COLOR_BUTTON_PRESSED: Color = COLOR_NOF1_LIGHT_BLUE;
const COLOR_EMOTION: Color = Color::YELLOW;
const COLOR_VALENCE_MANDALA_CLOSED: Color = Color {
    // Turqoise, translucent, Positive smoother more open
    r: 64.0 / 256.0,
    g: 224.0 / 256.0,
    b: 208.0 / 256.0,
    a: 0.5,
};
const COLOR_VALENCE_MANDALA_OPEN: Color = Color {
    // Crimson, Negative spiky emotion
    r: 220.0 / 256.0,
    g: 20.0 / 256.0,
    b: 60.0 / 256.0,
    a: 1.0,
};
const COLOR_AROUSAL_MANDALA_CLOSED: Color = Color {
    // Dark purple, translucent, low arousal
    r: 75.0 / 256.0,
    g: 48.0 / 255.0,
    b: 165.0 / 255.0,
    a: 0.4,
};
const COLOR_AROUSAL_MANDALA_OPEN: Color = Color {
    // Orange, opaque, high arousal
    r: 1.0,
    g: 0.67,
    b: 0.0,
    a: 1.0,
};

const BUTTON_WIDTH: f32 = 200.0;
const BUTTON_HEIGHT: f32 = 50.0;
const BUTTON_H_MARGIN: f32 = 20.0;
const BUTTON_V_MARGIN: f32 = 20.0;

const TITLE_V_MARGIN: f32 = 40.0;
const TEXT_V_MARGIN: f32 = 200.0;

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
    start_time: Instant,
    title_text: Asset<Image>,
    help_text: Asset<Image>,
    logo: Asset<Image>,
    sound_click: Asset<Sound>,
    sound_blah: Asset<Sound>,
    left_button_color: Color,
    right_button_color: Color,
    mandala_valence: Mandala,
    mandala_arousal: Mandala,
    muse_model: MuseModel,
    eeg_view_state: EegViewState,
    _rx_eeg: Receiver<(Duration, muse_model::MuseMessageType)>,
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
        Ok(())
    }

    fn right_action(&mut self, _window: &mut Window) -> Result<()> {
        self.right_button_color = COLOR_BUTTON_PRESSED;
        self.sound_click.execute(|sound| sound.play())
    }
}

impl AppState {
    fn seconds_since_start(&self) -> f32 {
        self.start_time.elapsed().as_nanos() as f32 / 1000000000.0
    }

    fn draw_mandala(&mut self, window: &mut Window) {
        let mut mesh = Mesh::new();

        let mut shape_renderer = ShapeRenderer::new(&mut mesh, Color::RED);
        let seconds_since_start = self.seconds_since_start();
        self.mandala_valence
            .draw(seconds_since_start, &mut shape_renderer);
        self.mandala_arousal
            .draw(seconds_since_start, &mut shape_renderer);
        window.mesh().extend(&mesh);
    }
}

#[allow(dead_code)]
fn bound_normalized_value(normalized: f32) -> f32 {
    normalized.max(3.0).min(-3.0)
}

/// Create a log of values and events collected during a session
fn create_log_writer(filename: &str) -> Writer<File> {
    let writer: Writer<File> =
        Writer::from_path(filename).expect("Could not open CSV file for writing");

    writer
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
        let mandala_valence_state_open = MandalaState::new(
            COLOR_VALENCE_MANDALA_OPEN,
            Transform::rotate(90),
            Transform::translate((50.0, 0.0)),
            Transform::scale((1.0, 1.0)),
        );
        let mandala_valence_state_closed = MandalaState::new(
            COLOR_VALENCE_MANDALA_CLOSED,
            Transform::rotate(0.0),
            Transform::translate((0.0, 0.0)),
            Transform::scale((0.1, 1.0)),
        );
        let mut mandala_valence = Mandala::new(
            MANDALA_VALENCE_PETAL_SVG_NAME,
            MANDALA_CENTER,
            MANDALA_SCALE,
            12,
            mandala_valence_state_open,
            mandala_valence_state_closed,
            1.0,
        );
        let mandala_arousal_state_open = MandalaState::new(
            COLOR_AROUSAL_MANDALA_OPEN,
            Transform::rotate(5),
            Transform::translate((0.0, 0.0)),
            Transform::scale((0.4, 0.8)),
        );
        let mandala_arousal_state_closed = MandalaState::new(
            COLOR_AROUSAL_MANDALA_CLOSED,
            Transform::rotate(90.0),
            Transform::translate((0.0, 0.0)),
            Transform::scale((0.2, 1.0)),
        );
        let mut mandala_arousal = Mandala::new(
            MANDALA_AROUSAL_PETAL_SVG_NAME,
            MANDALA_CENTER,
            MANDALA_SCALE,
            20,
            mandala_arousal_state_open,
            mandala_arousal_state_closed,
            0.0,
        );
        mandala_valence.start_transition(0.0, 3.0, 0.0);
        mandala_arousal.start_transition(0.0, 3.0, 1.0);

        let eeg_view_state = EegViewState::new();
        let start_time = Instant::now();
        println!("Start instant: {:?}", start_time);

        Ok(AppState {
            frame_count: 0,
            start_time,
            title_text,
            help_text,
            logo,
            sound_click,
            sound_blah,
            mandala_valence,
            mandala_arousal,
            left_button_color: COLOR_CLEAR,
            right_button_color: COLOR_CLEAR,
            eeg_view_state,
            _rx_eeg: rx_eeg,
            muse_model,
        })
    }

    // This is called UPS times per second
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
                self.muse_model.flush_all();
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
            self.muse_model.display_type = DisplayType::Mandala;
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

        let (normalized_valence_option, normalized_arousal_option) =
            self.muse_model.receive_packets();
        if self.frame_count > FRAME_TITLE {
            let current_time = self.seconds_since_start();
            // println!("Time: {}", current_time);
            if let Some(normalized_valence) = normalized_valence_option {
                // println!("Normalized valence: {}", normalized_valence);
                if normalized_valence.is_finite() {
                    self.mandala_valence.start_transition(
                        current_time,
                        MANDALA_TRANSITION_DURATION,
                        // bound_normalized_value(normalized_valence),
                        normalized_valence,
                    );
                }
            }
            if let Some(normalized_arousal) = normalized_arousal_option {
                // println!("Normalized arousal: {}", normalized_arousal);
                if normalized_arousal.is_finite() {
                    self.mandala_arousal.start_transition(
                        current_time,
                        MANDALA_TRANSITION_DURATION,
                        // bound_normalized_value(normalized_arousal),
                        normalized_arousal,
                    );
                }
            }
        }
        self.muse_model.count_down();

        Ok(())
    }

    fn event(&mut self, event: &Event, _window: &mut Window) -> Result<()> {
        if let Event::Closed = event {
            self.shutdown_hooks()?;
        }

        Ok(())
    }

    // This is called FPS times per second
    fn draw(&mut self, window: &mut Window) -> Result<()> {
        let background_color = match self.frame_count < FRAME_TITLE {
            true => Color::BLACK,
            false => COLOR_BACKGROUND,
        };
        window.clear(background_color)?;

        if self.frame_count == FRAME_INTRO {
            // PLAY INTRO AUDIO AUTOMATICALLY WHEN THE TEXT APPEARS
            let _result = self.sound_blah.execute(|sound| sound.play());
        }

        if self.frame_count < FRAME_TITLE {
            self.draw_mandala(window);

            // LOGO
            self.logo.execute(|image| {
                window.draw(
                    &image
                        .area()
                        .with_center((SCREEN_SIZE.0 / 2.0, SCREEN_SIZE.1 / 4.0)),
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
        // let right_color = self.right_button_color;
        // self.sound_click.execute(|_| {
        //     window.draw(&RECT_RIGHT_BUTTON, Col(right_color));
        //     Ok(())
        // })?;
        // self.right_button_color = COLOR_BUTTON;
        } else if self.frame_count < FRAME_SETTLE {
            match self.muse_model.display_type {
                DisplayType::Mandala => self.draw_mandala(window),
                _ => eeg_view::draw_view(&self.muse_model, window, &mut self.eeg_view_state),
            }
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
    let draw_rate: f64 = 1000. / FPS as f64;
    let update_rate: f64 = 1000. / UPS as f64;

    let settings = Settings {
        icon_path: Some("n-icon.png"),
        fullscreen: true,
        resize: ResizeStrategy::Fit,
        draw_rate,
        update_rate,
        ..Settings::default()
    };

    run::<AppState>(
        STR_TITLE,
        Vector::new(SCREEN_SIZE.0, SCREEN_SIZE.1),
        settings,
    )
}
