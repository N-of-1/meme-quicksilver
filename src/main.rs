// Draw some multi-colored geometry to the screen
extern crate quicksilver;

use quicksilver::{
    combinators::result,
    geom::{Circle, Line, Rectangle, Shape, Transform, Triangle, Vector},
    graphics::{Background::Col, Background::Img, Color, Font, FontStyle, Image, ResizeStrategy},
    input::{ButtonState, GamepadButton, Key, MouseButton},
    lifecycle::{run, Asset, Event, Settings, State, Window},
    Future, Result,
};

struct DrawState {
    extra_bold: Asset<Image>,
    logo: Asset<Image>,
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
            let style = FontStyle::new(72.0, Color::BLACK);
            result(font.render("Meme Machine", &style))
        }));

        let logo = Asset::new(Image::load("nof1-logo.png"));

        Ok(DrawState { extra_bold, logo })
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
            || window.mouse()[MouseButton::Right].is_down()
            || window
                .gamepads()
                .iter()
                .any(|pad| pad[GamepadButton::TriggerLeft].is_down())
            || window
                .gamepads()
                .iter()
                .any(|pad| pad[GamepadButton::ShoulderLeft].is_down())
        {
            // TODO Left press
        }

        // RIGHT ACTION
        if window.keyboard()[Key::RShift] == ButtonState::Pressed
            || window.mouse()[MouseButton::Right].is_down()
            || window
                .gamepads()
                .iter()
                .any(|pad| pad[GamepadButton::TriggerRight].is_down())
            || window
                .gamepads()
                .iter()
                .any(|pad| pad[GamepadButton::ShoulderRight].is_down())
        {
            // TODO Right press
        }

        Ok(())
    }

    fn event(&mut self, event: &Event, window: &mut Window) -> Result<()> {
        if let Event::Closed = event {
            // TODO self.shutdown_hooks();
        }
        Ok(())
    }

    // This is called 30 times per second
    fn draw(&mut self, window: &mut Window) -> Result<()> {
        window.clear(Color::WHITE)?;

        self.logo.execute(|image| {
            window.draw(&image.area().with_center((400, 150)), Img(&image));
            Ok(())
        })?;

        self.extra_bold.execute(|image| {
            window.draw(&image.area().with_center((400, 300)), Img(&image));
            Ok(())
        })?;

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
        draw_rate: 35.0, // Max 30fps
        ..Settings::default()
    };

    run::<DrawState>("Meme Machine", Vector::new(1280, 768), settings);
}
