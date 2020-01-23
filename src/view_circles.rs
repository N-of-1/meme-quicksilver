use crate::muse_model::MuseModel;
use crate::*;

use quicksilver::{
    geom::Circle,
    graphics::{Background::Col, Color},
    lifecycle::Window,
};

const LINE_WEIGHT: f32 = 10.0;
const KEY_X: f32 = 600.0;
const KEY_Y: f32 = -100.0;
const KEY_VERT_SPACING: f32 = 30.0;

const TEXT_ALPHA: &str = "Alpha";
const TEXT_BETA: &str = "Beta";
const TEXT_GAMMA: &str = "Gamma";
const TEXT_DELTA: &str = "Delta";
const TEXT_THETA: &str = "Theta";

pub const COLOR_ALPHA: Color = Color {
    r: 178. / 256.,
    g: 178. / 256.,
    b: 1.0,
    a: 1.0,
};
pub const COLOR_BETA: Color = Color {
    r: 178. / 256.,
    g: 1.0,
    b: 178. / 256.,
    a: 1.0,
};
pub const COLOR_GAMMA: Color = Color {
    r: 1.0,
    g: 178. / 256.,
    b: 178. / 256.,
    a: 1.0,
};
pub const COLOR_DELTA: Color = Color {
    r: 178. / 256.,
    g: 1.0,
    b: 1.0,
    a: 1.0,
};
pub const COLOR_THETA: Color = Color {
    r: 1.0,
    g: 178. / 256.,
    b: 1.0,
    a: 1.0,
};

/// Render concenctric circules associated with alpha, beta, gamma..
pub fn draw(model: &MuseModel, window: &mut Window) {
    match model.display_type {
        DisplayType::FourCircles => draw_four_circles_view(&model, window),
        DisplayType::Dowsiness => draw_drowsiness_view(&model, window),
        DisplayType::Emotion => draw_emotion_view(&model, window),
    }
}

fn average_from_four_electrodes(x: &[f32; 4]) -> f32 {
    (x[0] + x[1] + x[2] + x[3]) / 4.0
}

fn asymmetry(x: &[f32; 4], n: f32) -> f32 {
    let base = std::f32::consts::E;
    base.powf(x[1] / n - x[2] / n)
}

fn draw_emotion_view(model: &MuseModel, window: &mut Window) {
    let lizard_mind = average_from_four_electrodes(&model.theta);
    let asymm = asymmetry(&model.alpha, lizard_mind);

    draw_polygon(COLOR_ALPHA, asymm, window, model.scale, (0.0, 0.0));
    draw_polygon(COLOR_THETA, lizard_mind, window, model.scale, (0.0, 0.0));
}

fn draw_drowsiness_view(model: &MuseModel, window: &mut Window) {
    let lizard_mind = (average_from_four_electrodes(&model.theta)
        + average_from_four_electrodes(&model.delta))
        / 2.0;
    draw_polygon(COLOR_THETA, lizard_mind, window, model.scale, (0.0, 0.0));
    draw_polygon(
        COLOR_ALPHA,
        average_from_four_electrodes(&model.alpha),
        window,
        model.scale,
        (0.0, 0.0),
    );
}

fn draw_four_circles_view(model: &MuseModel, window: &mut Window) {
    const DISTANCE: f32 = 100.0;
    const LEFT_FRONT: (f32, f32) = (-DISTANCE, -DISTANCE);
    const RIGHT_FRONT: (f32, f32) = (DISTANCE, -DISTANCE);
    const RIGHT_REAR: (f32, f32) = (DISTANCE, DISTANCE);
    const LEFT_REAR: (f32, f32) = (-DISTANCE, DISTANCE);

    // draw_key(0, "Blink", blink_color(model.is_blink()), &window);
    // draw_key(1, "Jaw Clench", blink_color(model.is_jaw_clench()), &window);
    // draw_key(
    //     2,
    //     "Forehead",
    //     blink_color(model.is_touching_forehead()),
    //     &window,
    // );
    // draw_key(3, TEXT_ALPHA, COLOR_ALPHA, &window);
    // draw_key(4, TEXT_BETA, COLOR_BETA, &window);
    // draw_key(5, TEXT_GAMMA, COLOR_GAMMA, &window);
    // draw_key(6, TEXT_DELTA, COLOR_DELTA, &window);
    // draw_key(7, TEXT_THETA, COLOR_THETA, &window);

    draw_concentric_polygons(&model, window, 0, LEFT_REAR);
    draw_concentric_polygons(&model, window, 1, LEFT_FRONT);
    draw_concentric_polygons(&model, window, 2, RIGHT_FRONT);
    draw_concentric_polygons(&model, window, 3, RIGHT_REAR);
}

fn draw_concentric_polygons(
    model: &MuseModel,
    window: &mut Window,
    index: usize,
    offset: (f32, f32),
) {
    draw_polygon(COLOR_ALPHA, model.alpha[index], window, model.scale, offset);
    draw_polygon(COLOR_BETA, model.beta[index], window, model.scale, offset);
    draw_polygon(COLOR_GAMMA, model.gamma[index], window, model.scale, offset);
    draw_polygon(COLOR_DELTA, model.delta[index], window, model.scale, offset);
    draw_polygon(COLOR_THETA, model.theta[index], window, model.scale, offset);
}

fn blink_color(blink: bool) -> Color {
    if blink {
        return COLOR_NOF1_LIGHT_BLUE;
    }

    COLOR_BACKGROUND
}

const RECT_KEY: (f32, f32) = (50.0, 10.0);

// fn draw_key(i: i32, text: &str, &text: Asset<Image>, line_color: Color, window: &Window) {
//     let y = KEY_Y - KEY_VERT_SPACING * i as f32;

//     draw.rect().x(KEY_X).y(y).w(50.0).h(10.0).color(line_color);

//     draw.text(text).x(KEY_X).y(y - 10.0);

//     text.execute(|image| {
//         window.draw(
//             &image
//                 .area()
//                 .with_center((SCREEN_SIZE.0 / 2.0, TEXT_V_MARGIN)),
//             Img(&image),
//         );
//         Ok(())
//     })?;
// }

fn draw_polygon(line_color: Color, value: f32, window: &mut Window, scale: f32, shift: (f32, f32)) {
    let screen_size = window.screen_size();
    let scale = screen_size.x / scale;
    let radius = value * scale;
    let x = (screen_size.x / 2.0) + shift.0;
    let y = (screen_size.y / 2.0) + shift.1;

    window.draw(&Circle::new((x, y), radius), Col(line_color));
}
