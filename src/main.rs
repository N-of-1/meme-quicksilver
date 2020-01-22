// Draw some multi-colored geometry to the screen
#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
extern crate env_logger;

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
extern crate web_logger;

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
extern crate nannou_osc;

extern crate quicksilver;

#[macro_use]
extern crate log;

use quicksilver::{
    combinators::result,
    geom::{Circle, Line, Rectangle, Shape, Transform, Triangle, Vector},
    graphics::{Background::Col, Background::Img, Color, Font, FontStyle, Image, ResizeStrategy},
    input::{ButtonState, GamepadButton, Key, MouseButton},
    lifecycle::{run, Asset, Event, Settings, State, Window},
    sound::Sound,
    Future, Result,
};
use std::env;

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
mod muse_packet;

const SCREEN_WIDTH: f32 = 1280.0;

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
const SCREEN_HEIGHT: f32 = 768.0;
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
const SCREEN_HEIGHT: f32 = 650.0;

const FPS: u64 = 30;
const FRAME_TITLE: u64 = 3 * FPS; // 30 frames/sec
const FRAME_INTRO: u64 = FRAME_TITLE + 4 * FPS;
const FRAME_SETTLE: u64 = FRAME_INTRO + 4 * FPS;
const FRAME_MEME: u64 = FRAME_SETTLE + 4 * FPS;

const IMG_LOGO: &str = "N_of_1_logo_blue_transparent.png";

const FONT_EXTRA_BOLD: &str = "WorkSans-ExtraBold.ttf";
const FONT_MULI: &str = "Muli-VariableFont_wght.ttf";
const FONT_EXTRA_BOLD_SIZE: f32 = 72.0;
const FONT_MULI_SIZE: f32 = 40.0;

const SND_CLICK: &str = "click.ogg";
const SND_BLAH: &str = "blah.ogg";

const ENV_SCREEN_SIZE: &str = "SCREEN_SIZE";

const STR_TITLE: &str = "Meme Machine";
const STR_HELP_TEXT: &str = "Blah blah blah some blah\nmore blah";

const CLR_GREY: Color = Color {
    r: 0.5,
    g: 0.5,
    b: 0.5,
    a: 1.0,
};
const CLR_CLEAR: Color = Color {
    r: 0.5,
    g: 0.5,
    b: 0.5,
    a: 0.0,
};
const CLR_NOF1_DK_BLUE: Color = Color {
    r: 31. / 256.,
    g: 18. / 256.,
    b: 71. / 256.,
    a: 1.0,
};
const CLR_NOF1_LT_BLUE: Color = Color {
    r: 189. / 256.,
    g: 247. / 256.,
    b: 255. / 256.,
    a: 1.0,
};
const CLR_NOF1_TURQOISE: Color = Color {
    r: 0. / 256.,
    g: 200. / 256.,
    b: 200. / 256.,
    a: 1.0,
};

const CLR_BACKGROUND: Color = CLR_GREY;
const CLR_TITLE: Color = CLR_NOF1_DK_BLUE;
const CLR_TEXT: Color = Color::BLACK;
const CLR_BUTTON: Color = CLR_NOF1_DK_BLUE;
const CLR_BUTTON_PRESSED: Color = CLR_NOF1_LT_BLUE;

const BTN_WIDTH: f32 = 200.0;
const BTN_HEIGHT: f32 = 50.0;
const BTN_H_MARGIN: f32 = 20.0;
const BTN_V_MARGIN: f32 = 20.0;

const TITLE_V_MARGIN: f32 = 40.0;
const TEXT_V_MARGIN: f32 = 200.0;

const RECT_LEFT_BUTTON: Rectangle = Rectangle {
    pos: Vector {
        x: BTN_H_MARGIN,
        y: SCREEN_HEIGHT - BTN_V_MARGIN - BTN_HEIGHT,
    },
    size: Vector {
        x: BTN_WIDTH,
        y: BTN_HEIGHT,
    },
};

const RECT_RIGHT_BUTTON: Rectangle = Rectangle {
    pos: Vector {
        x: SCREEN_WIDTH - BTN_H_MARGIN - BTN_WIDTH,
        y: SCREEN_HEIGHT - BTN_V_MARGIN - BTN_HEIGHT,
    },
    size: Vector {
        x: BTN_WIDTH,
        y: BTN_HEIGHT,
    },
};

struct DrawState {
    frame_count: u64,
    font_extra_bold: Asset<Image>,
    font_muli: Asset<Image>,
    logo: Asset<Image>,
    sound_click: Asset<Sound>,
    sound_blah: Asset<Sound>,
    left_button_color: Color,
    right_button_color: Color,
}

impl DrawState {
    // Perform any shutdown actions
    // Do not call this directly to end the app. Instead call window.close();
    fn shutdown_hooks(&mut self) -> Result<()> {
        // TODO Notify database session ended

        Ok(())
    }
}

impl DrawState {
    fn left_action(&mut self, _window: &mut Window) -> Result<()> {
        self.left_button_color = CLR_BUTTON_PRESSED;
        self.sound_click
            .execute(|sound| sound.play())
            .expect("Could not play left button sound");
        self.sound_blah.execute(|sound| sound.play())
    }

    fn right_action(&mut self, _window: &mut Window) -> Result<()> {
        self.right_button_color = CLR_BUTTON_PRESSED;
        self.sound_click.execute(|sound| sound.play())
    }
}

impl State for DrawState {
    fn new() -> Result<DrawState> {
        let font_extra_bold = Asset::new(Font::load(FONT_EXTRA_BOLD).and_then(|font| {
            let style = FontStyle::new(FONT_EXTRA_BOLD_SIZE, CLR_TITLE);
            result(font.render(STR_TITLE, &style))
        }));
        let font_muli = Asset::new(Font::load(FONT_MULI).and_then(|font| {
            let style = FontStyle::new(FONT_MULI_SIZE, CLR_TEXT);
            result(font.render(STR_HELP_TEXT, &style))
        }));

        let logo = Asset::new(Image::load(IMG_LOGO));
        let sound_click = Asset::new(Sound::load(SND_CLICK));
        let sound_blah = Asset::new(Sound::load(SND_BLAH));

        Ok(DrawState {
            frame_count: 0,
            font_extra_bold,
            font_muli,
            logo,
            sound_click,
            sound_blah,
            left_button_color: CLR_CLEAR,
            right_button_color: CLR_CLEAR,
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
        window.clear(CLR_BACKGROUND)?;

        if self.frame_count < FRAME_TITLE {
            // LOGO
            self.logo.execute(|image| {
                window.draw(
                    &image
                        .area()
                        .with_center((SCREEN_WIDTH / 2.0, SCREEN_HEIGHT / 2.0)),
                    Img(&image),
                );
                Ok(())
            })?;
        } else if self.frame_count < FRAME_INTRO {
            // TITLE
            self.font_extra_bold.execute(|image| {
                window.draw(
                    &image
                        .area()
                        .with_center((SCREEN_WIDTH / 2.0, TITLE_V_MARGIN)),
                    Img(&image),
                );
                Ok(())
            })?;

            // TITLE TEXT
            self.font_muli.execute(|image| {
                window.draw(
                    &image
                        .area()
                        .with_center((SCREEN_WIDTH / 2.0, TEXT_V_MARGIN)),
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
            self.right_button_color = CLR_BUTTON;
        } else if self.frame_count < FRAME_SETTLE {
            window.draw(
                &Circle::new((SCREEN_WIDTH / 2.0, SCREEN_HEIGHT / 2.0), 100),
                Col(Color::GREEN),
            );
        } else if self.frame_count < FRAME_MEME {
            // LEFT BUTTON
            let left_color = self.left_button_color;
            self.sound_click.execute(|_| {
                window.draw(&RECT_LEFT_BUTTON, Col(left_color));
                Ok(())
            })?;
            self.left_button_color = CLR_BUTTON;

            // RIGHT BUTTON
            let right_color = self.right_button_color;
            self.sound_click.execute(|_| {
                window.draw(&RECT_RIGHT_BUTTON, Col(right_color));
                Ok(())
            })?;
            self.right_button_color = CLR_BUTTON;
        } else {
            // LOGO
            self.logo.execute(|image| {
                window.draw(
                    &image
                        .area()
                        .with_center((SCREEN_WIDTH / 2.0, SCREEN_HEIGHT / 2.0)),
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

    /* default Settings {
        show_cursor: true,
        min_size: None,
        max_size: None,
        resize: ResizeStrategy::default(),
        scale: ImageScaleStrategy::default(),
        fullscreen: false,
        update_rate: 1000. / 60.,
        max_updates: 0,
        draw_rate: 0.,
        icon_path: None,
        vsync: true,
        multisampling: None,
    };*/
    let mut screen_size = Vector::new(SCREEN_WIDTH, SCREEN_HEIGHT);
    match env::var(ENV_SCREEN_SIZE) {
        Ok(ss) => {
            let parsed: Vec<&str> = ss.split(',').collect();
            screen_size.x = parsed[0]
                .parse::<f32>()
                .expect("Local variable for screen size in wrong format 'x,y'");
            screen_size.y = parsed[1]
                .parse::<f32>()
                .expect("Local variable for screen size in wrong format 'x,y'");
        }
        _ => (),
    }

    let settings = Settings {
        icon_path: Some("n-icon.png"),
        fullscreen: true,
        resize: ResizeStrategy::Maintain,
        draw_rate: 35.0,          // 35ms ~= max 30fps
        update_rate: 1000. / 60., // 60 times per second
        ..Settings::default()
    };

    run::<DrawState>(STR_TITLE, screen_size, settings);
}
