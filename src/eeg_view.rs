use crate::muse_model::MuseModel;
use crate::*;
use core::f32::consts::PI;

use quicksilver::{
    geom::{Circle, Scalar},
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
pub const EEG_COLORS: [Color; 5] = [
    COLOR_ALPHA,
    COLOR_BETA,
    COLOR_GAMMA,
    COLOR_DELTA,
    COLOR_THETA,
];
const COLOR_SPIDER_GRAPH: Color = Color::WHITE;
const FIRST_EEG_CHANNEL: usize = 0;
const N_EEG_CHANNELS: usize = 4;
const FIRST_EEG_DERIVED_VALUE: usize = 0;
const N_EEG_DERIVED_VALUES: usize = 5;
const SPIDER_LINE_THICKNESS: f32 = 2.5; // Thickness of the line between points
const SPIDER_LINE_AXIS_THICKNESS: f32 = 1.0; // Thickness of the axis labels
const SPIDER_SPACING: f32 = 200.0; // How far apart the spider graphs are drawn
const SPIDER_POINT_RADIUS: f32 = 10.0; // Size of the dot on each graph point
const SPIDER_GRAPH_SCALE: f32 = 0.15; // Size of graph as % of screen size

/// Render concenctric circules associated with alpha, beta, gamma..
pub fn draw_view(muse_model: &MuseModel, window: &mut Window) {
    match muse_model.display_type {
        DisplayType::FourCircles => draw_four_circles_view(muse_model, window),
        DisplayType::Dowsiness => draw_drowsiness_view(muse_model, window),
        DisplayType::Emotion => draw_emotion_view(muse_model, window),
        DisplayType::EegValues => draw_eeg_values_view(muse_model, window),
    }
}

fn average_from_four_electrodes(x: &[f32; 4]) -> f32 {
    (x[0] + x[1] + x[2] + x[3]) / 4.0
}

fn asymmetry(x: &[f32; 4], n: f32) -> f32 {
    let base = std::f32::consts::E;
    base.powf(x[1] / n - x[2] / n)
}

/// A bigger yellow circle indiates greater happiness. Maybe.
fn draw_emotion_view(model: &MuseModel, window: &mut Window) {
    let lizard_mind = average_from_four_electrodes(&model.theta);
    let asymm = asymmetry(&model.alpha, lizard_mind);

    draw_polygon(&COLOR_EMOTION, asymm / 5.0, window, model.scale, (0.0, 0.0));
}

fn draw_drowsiness_view(model: &MuseModel, window: &mut Window) {
    let lizard_mind = (average_from_four_electrodes(&model.theta)
        + average_from_four_electrodes(&model.delta))
        / 2.0;
    draw_polygon(&COLOR_THETA, lizard_mind, window, model.scale, (0.0, 0.0));
    draw_polygon(
        &COLOR_ALPHA,
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
    draw_polygon(
        &COLOR_ALPHA,
        model.alpha[index],
        window,
        model.scale,
        offset,
    );
    draw_polygon(&COLOR_BETA, model.beta[index], window, model.scale, offset);
    draw_polygon(
        &COLOR_GAMMA,
        model.gamma[index],
        window,
        model.scale,
        offset,
    );
    draw_polygon(
        &COLOR_DELTA,
        model.delta[index],
        window,
        model.scale,
        offset,
    );
    draw_polygon(
        &COLOR_THETA,
        model.theta[index],
        window,
        model.scale,
        offset,
    );
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

/// Put a circle on screen, manually scaled based on screen size and 'scale' factor, shifted from screen center by 'shift'
fn draw_polygon(
    line_color: &Color,
    value: f32,
    window: &mut Window,
    scale: f32,
    shift: (f32, f32),
) {
    let screen_size = window.screen_size();
    let scale = screen_size.x / scale;
    let radius = value * scale;
    let x = (screen_size.x / 2.0) + shift.0;
    let y = (screen_size.y / 2.0) + shift.1;

    window.draw(&Circle::new((x, y), radius), Col(*line_color));
}

/// A set of all EEG values displayed for diagnostic purposes
fn draw_eeg_values_view(muse_model: &MuseModel, window: &mut Window) {
    let mut shift = [(0.0, 0.0); 4];
    let mut spider_values = [[0.0; 5]; 4];
    let mut graph_label_text: [Asset<Image>; 5] = [
        Asset::new(Font::load(FONT_MULI).and_then(|font| {
            result(font.render(
                EEG_LABELS[0],
                &FontStyle::new(FONT_EEG_LABEL_SIZE, COLOR_EEG_LABEL),
            ))
        })),
        Asset::new(Font::load(FONT_MULI).and_then(|font| {
            result(font.render(
                EEG_LABELS[1],
                &FontStyle::new(FONT_EEG_LABEL_SIZE, COLOR_EEG_LABEL),
            ))
        })),
        Asset::new(Font::load(FONT_MULI).and_then(|font| {
            result(font.render(
                EEG_LABELS[2],
                &FontStyle::new(FONT_EEG_LABEL_SIZE, COLOR_EEG_LABEL),
            ))
        })),
        Asset::new(Font::load(FONT_MULI).and_then(|font| {
            result(font.render(
                EEG_LABELS[3],
                &FontStyle::new(FONT_EEG_LABEL_SIZE, COLOR_EEG_LABEL),
            ))
        })),
        Asset::new(Font::load(FONT_MULI).and_then(|font| {
            result(font.render(
                EEG_LABELS[4],
                &FontStyle::new(FONT_EEG_LABEL_SIZE, COLOR_EEG_LABEL),
            ))
        })),
    ];

    for chan in FIRST_EEG_CHANNEL..N_EEG_CHANNELS {
        spider_values[chan][0] = muse_model.alpha[chan];
        spider_values[chan][1] = muse_model.beta[chan];
        spider_values[chan][2] = muse_model.gamma[chan];
        spider_values[chan][3] = muse_model.delta[chan];
        spider_values[chan][4] = muse_model.theta[chan];

        shift[chan] = (
            SPIDER_SPACING * (-PI / 4. + (chan as f32 * PI / 2.).cos()),
            SPIDER_SPACING * (-PI / 4. + (chan as f32 * PI / 2.).sin()),
        );

        draw_spider_graph(
            &mut graph_label_text,
            &EEG_COLORS,
            spider_values[chan],
            window,
            SPIDER_GRAPH_SCALE,
            shift[chan],
        );
    }
}

/// Put a circle on screen, manually scaled based on screen size and 'scale' factor, shifted from screen center by 'shift'
fn draw_spider_graph(
    label_images: &mut [Asset<Image>],
    line_color: &[Color],
    value: [f32; 5],
    window: &mut Window,
    graph_scale: f32,
    shift: (f32, f32),
) {
    let screen_size = window.screen_size();
    let scale = screen_size.x * graph_scale;
    let mut x = [[0.0; 5]; 4];
    let mut y = [[0.0; 5]; 4];
    let mut angle = [0.0; 5];

    assert!(FIRST_EEG_CHANNEL <= N_EEG_CHANNELS);
    assert!(FIRST_EEG_DERIVED_VALUE <= N_EEG_DERIVED_VALUES);
    assert!(N_EEG_DERIVED_VALUES <= EEG_COLORS.len());
    assert!(N_EEG_DERIVED_VALUES <= EEG_LABELS.len());

    for val in FIRST_EEG_DERIVED_VALUE..N_EEG_DERIVED_VALUES {
        angle[val] = ((val as f32 * 2. * PI) - (PI / 2.))
            / (N_EEG_DERIVED_VALUES - FIRST_EEG_DERIVED_VALUE) as f32;
    }

    // Calculate graph endpoints
    for chan in FIRST_EEG_CHANNEL..N_EEG_CHANNELS {
        for val in FIRST_EEG_DERIVED_VALUE..N_EEG_DERIVED_VALUES {
            let radius = scale * value[val];
            let (xv, yv) =
                end_of_spider_graph(val, radius, angle[val], shift, graph_scale, &screen_size);
            x[chan][val] = xv;
            y[chan][val] = yv;
        }
    }

    // Draw axis lines for each spider graph
    let axis_length = 150.0;
    for chan in FIRST_EEG_CHANNEL..N_EEG_CHANNELS {
        for val in FIRST_EEG_DERIVED_VALUE..N_EEG_DERIVED_VALUES {
            // Draw from center to outside edge of spider graph
            let center =
                end_of_spider_graph(val, 0.0, angle[val], shift, graph_scale, &screen_size);
            let tip = end_of_spider_graph(
                val,
                axis_length,
                angle[val],
                shift,
                graph_scale,
                &screen_size,
            );
            window.draw(
                &Line::new(center, tip).with_thickness(SPIDER_LINE_AXIS_THICKNESS),
                Col(COLOR_SPIDER_GRAPH),
            );

            // Draw outside border of spider graph
            let wrap_val = wrap_eeg_derived_value_index(val);
            let next_spoke_tip = end_of_spider_graph(
                wrap_eeg_derived_value_index(val),
                axis_length,
                angle[wrap_val],
                shift,
                graph_scale,
                &screen_size,
            );
            window.draw(
                &Line::new(tip, next_spoke_tip).with_thickness(SPIDER_LINE_AXIS_THICKNESS),
                Col(COLOR_SPIDER_GRAPH),
            );
        }
    }

    // Label the endpoints
    for chan in FIRST_EEG_CHANNEL..N_EEG_CHANNELS {
        for val in FIRST_EEG_DERIVED_VALUE..N_EEG_DERIVED_VALUES {
            // Draw the dot at each point on the spider graph
            window.draw(
                &Circle::new((x[chan][val], y[chan][val]), SPIDER_POINT_RADIUS),
                Col(line_color[val]),
            );

            // Draw the label over the dot
            &label_images[val].execute(|image| {
                window.draw(
                    &image.area().with_center((x[chan][val], y[chan][val])),
                    Img(&image),
                );
                Ok(())
            });
        }
    }
}

// Find the index of the next value with wrap-around
fn wrap_eeg_derived_value_index(i: usize) -> usize {
    ((i + 1) % N_EEG_DERIVED_VALUES) as usize
}

// Find the screen location of a spider graph value
fn end_of_spider_graph(
    graph_number: usize,
    radius: f32,
    angle: f32,
    shift: (f32, f32),
    graph_scale: f32,
    screen_size: &Vector,
) -> (f32, f32) {
    let scale = screen_size.x * graph_scale;

    (
        (screen_size.x / 2.0) + radius * angle.cos() as f32 + shift.0,
        (screen_size.x / 2.0) + radius * angle.sin() as f32 + shift.1,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrap_eeg_derived_value_index() {
        let i = 4;
        let next_i = 0;

        assert_eq!(next_i, wrap_eeg_derived_value_index(i));
    }
}
