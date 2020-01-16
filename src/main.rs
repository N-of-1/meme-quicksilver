// Draw some multi-colored geometry to the screen
extern crate quicksilver;

use quicksilver::{
    combinators::result,
    geom::{Circle, Line, Rectangle, Shape, Transform, Triangle, Vector},
    graphics::{Background::Col, Background::Img, Color, Font, FontStyle, Image, ResizeStrategy},
    input::{ButtonState, GamepadButton, Key, MouseButton},
    lifecycle::{run, Asset, Event, Settings, State, Window},
    sound::Sound,
    Future, Result,
};

const ENV_SCREEN_SIZE: &str = "SCREEN_SIZE";
const STR_TITLE: &str = "Meme Machine";
const COLOR_GREY: Color = Color {
    r: 0.5,
    g: 0.5,
    b: 0.5,
    a: 1.0,
};
const COLOR_BACKGROUND: Color = COLOR_GREY;
const COLOR_TEXT: Color = Color::BLACK;
const COLOR_BUTTON: Color = Color::BLUE;
const COLOR_BUTTON_PRESSED: Color = Color::WHITE;

const LEFT_BUTTON_AREA: Rectangle = Rectangle {
    pos: Vector { x: 100.0, y: 350.0 },
    size: Vector { x: 100.0, y: 50.0 },
};

const RIGHT_BUTTON_AREA: Rectangle = Rectangle {
    pos: Vector { x: 350.0, y: 350.0 },
    size: Vector { x: 100.0, y: 50.0 },
};

struct DrawState {
    frame_count: u64,
    extra_bold: Asset<Image>,
    logo: Asset<Image>,
    click_sound: Asset<Sound>,
    blah_sound: Asset<Sound>,
    welcome_sound: Asset<Sound>,
    left_button_color: Color,
    right_button_color: Color,
}

impl DrawState {
    // Perform any shutdown actions like closing the datbase if the user or OS clos
    // Do not call this directly to end the app. Instead call window.close();
    fn shutdown_hooks() {
        // Perform any shutdown actions like closing the datbase if the user or OS clos
    }
}

impl State for DrawState {
    fn new() -> Result<DrawState> {
        let extra_bold = Asset::new(Font::load("WorkSans-ExtraBold.ttf").and_then(|font| {
            let style = FontStyle::new(72.0, COLOR_TEXT);
            result(font.render(STR_TITLE, &style))
        }));

        let logo = Asset::new(Image::load("nof1-logo.png"));

        let click_sound = Asset::new(Sound::load("click.ogg"));

        let blah_sound = Asset::new(Sound::load("blah.ogg"));

        let welcome_sound = Asset::new(Sound::load("moog.ogg"));

        Ok(DrawState {
            frame_count: 0,
            extra_bold: extra_bold,
            logo: logo,
            click_sound: click_sound,
            blah_sound: blah_sound,
            welcome_sound: welcome_sound,
            left_button_color: COLOR_BUTTON,
            right_button_color: COLOR_BUTTON,
        })
    }

    // This is called 60 times per second
    fn update(&mut self, window: &mut Window) -> Result<()> {
        // EXIT APP
        if window.keyboard()[Key::Escape].is_down()
            || window
                .gamepads()
                .iter()
                .any(|pad| pad[GamepadButton::FaceLeft].is_down())
        {
            window.close();
        }

        // LEFT ACTION
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
            self.left_button_color = COLOR_BUTTON_PRESSED;
            self.click_sound
                .execute(|sound| sound.play())
                .expect("Could not play left button sound");
            self.blah_sound
                .execute(|sound| sound.play())
                .expect("Could not play left button sound");
        }

        // RIGHT ACTION
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
            self.right_button_color = COLOR_BUTTON_PRESSED;
            self.click_sound
                .execute(|sound| sound.play())
                .expect("Could not play right button sound");
        }

        // LEFT SCREEN BUTTON PRESS
        if window.mouse()[MouseButton::Left] == ButtonState::Pressed
            && LEFT_BUTTON_AREA.contains(window.mouse().pos())
        {
            self.left_button_color = COLOR_BUTTON_PRESSED;
            self.click_sound
                .execute(|sound| sound.play())
                .expect("Could not play left button sound");
            self.blah_sound
                .execute(|sound| sound.play())
                .expect("Could not play left button sound");
        }

        // RIGHT SCREEN BUTTON PRESS
        if window.mouse()[MouseButton::Left] == ButtonState::Pressed
            && RIGHT_BUTTON_AREA.contains(window.mouse().pos())
        {
            self.right_button_color = COLOR_BUTTON_PRESSED;
            self.click_sound
                .execute(|sound| sound.play())
                .expect("Could not play right button sound");
        }

        Ok(())
    }

    fn event(&mut self, event: &Event, _window: &mut Window) -> Result<()> {
        if let Event::Closed = event {
            // TODO self.shutdown_hooks();
        }
        Ok(())
    }

    // This is called 30 times per second
    fn draw(&mut self, window: &mut Window) -> Result<()> {
        if self.frame_count == 0 {
            //TODO            window.set_size(window.screen_size());
            self.welcome_sound
                .execute(|sound| sound.play())
                .expect("Could not play right button sound");
        }

        window.clear(COLOR_BACKGROUND)?;

        // LOGO
        self.logo.execute(|image| {
            window.draw(&image.area().with_center((640, 150)), Img(&image));
            Ok(())
        })?;

        // TITLE TEXT
        self.extra_bold.execute(|image| {
            window.draw(&image.area().with_center((640, 300)), Img(&image));
            Ok(())
        })?;

        // LEFT BUTTON
        let left_color = self.left_button_color;
        self.click_sound.execute(|_| {
            window.draw(&LEFT_BUTTON_AREA, Col(left_color));
            Ok(())
        })?;
        self.left_button_color = COLOR_BUTTON;

        // RIGHT BUTTON
        let right_color = self.right_button_color;
        self.click_sound.execute(|_| {
            window.draw(&RIGHT_BUTTON_AREA, Col(right_color));
            Ok(())
        })?;
        self.right_button_color = COLOR_BUTTON;

        /*        window.draw(&Rectangle::new((100, 100), (32, 32)), Col(Color::BLUE));
        window.draw_ex(
            &Rectangle::new((400, 300), (32, 32)),
            Col(Color::BLUE),
            Transform::rotate(45),
            10,
        );
        window.draw(&Circle::new((400, 300), 100), Col(Color::GREEN));
        window.draw_ex(
            &Line::new((50, 80), (600, 450)).with_thickness(2.0),
            Col(Color::RED),
            Transform::IDENTITY,
            5,
        );
        window.draw_ex(
            &Triangle::new((500, 50), (450, 100), (650, 150)),
            Col(Color::RED),
            Transform::rotate(45) * Transform::scale((0.5, 0.5)),
            0,
        );*/

        self.frame_count = self.frame_count + 1;
        if self.frame_count == std::u64::MAX {
            self.frame_count = 1;
        }

        Ok(())
    }
}

fn main() {
    use quicksilver::graphics::*;

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

    let settings = Settings {
        icon_path: Some("n-icon.png"),
        fullscreen: true,
        resize: ResizeStrategy::Maintain,
        draw_rate: 35.0, // 35ms ~= max 30fps
        ..Settings::default()
    };

    run::<DrawState>("Meme Machine", Vector::new(1280, 768), settings);
}
