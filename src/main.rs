// Draw some multi-colored geometry to the screen
extern crate quicksilver;

use quicksilver::{
    combinators::result,
    geom::{Circle, Line, Rectangle, Shape, Transform, Triangle, Vector},
    graphics::{Background::Col, Background::Img, Color, Font, FontStyle, Image},
    lifecycle::{run, Asset, Settings, State, Window},
    Future, Result,
};

struct DrawGeometry {
    asset: Asset<Image>,
}

impl State for DrawGeometry {
    fn new() -> Result<DrawGeometry> {
        let asset = Asset::new(Font::load("WorkSans-ExtraBold.ttf").and_then(|font| {
            let style = FontStyle::new(72.0, Color::BLACK);
            result(font.render("Sample Text", &style))
        }));

        Ok(DrawGeometry { asset })
    }

    fn draw(&mut self, window: &mut Window) -> Result<()> {
        window.clear(Color::WHITE)?;

        self.asset.execute(|image| {
            window.draw(&image.area().with_center((400, 300)), Img(&image));
            Ok(())
        })?;

        window.draw(&Rectangle::new((100, 100), (32, 32)), Col(Color::BLUE));
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
        );
        Ok(())
    }
}

fn main() {
    use quicksilver::graphics::*;

    let settings = Settings {
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
    };

    run::<DrawGeometry>("Draw Geometry", Vector::new(800, 600), settings);
}
