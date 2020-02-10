use crate::muse_model::MuseModel;
use crate::*;
use core::f32::consts::PI;

use quicksilver::{
    geom::{Circle, Vector},
    graphics::{Background::Col, Color},
    lifecycle::Window,
};

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
const EEG_CHANNEL_LABELS: [&str; 4] = ["TP9", "AF7", "AF8", "TP10"];
const SPIDER_GRAPH_POSITIONS: [Vector; 4] = [
    Vector { x: 300.0, y: 300.0 },
    Vector { x: 700.0, y: 300.0 },
    Vector {
        x: 1100.0,
        y: 300.0,
    },
    Vector {
        x: 1500.0,
        y: 300.0,
    },
];

const COLOR_SPIDER_GRAPH: Color = Color::WHITE;
const FIRST_EEG_CHANNEL: usize = 0;
const N_EEG_CHANNELS: usize = 4;
const FIRST_EEG_DERIVED_VALUE: usize = 0;
const N_EEG_DERIVED_VALUES: usize = 5;

const SPIDER_LINE_THICKNESS: f32 = 2.5; // Thickness of the line between points
const SPIDER_LINE_AXIS_THICKNESS: f32 = 1.5; // Thickness of the axis labels
const SPIDER_POINT_RADIUS: f32 = 10.0; // Size of the dot on each graph point
const SPIDER_GRAPH_AXIS_LENGTH: f32 = 200.0; // Distance from center to pentagon tips
const SPIDER_GRAPH_LABEL_OFFSET: Vector = Vector { x: -160., y: -160. };

/// Render concenctric circules associated with alpha, beta, gamma..
pub fn draw_view(muse_model: &MuseModel, window: &mut Window) {
    match muse_model.display_type {
        DisplayType::FourCircles => draw_four_circles_view(muse_model, window),
        DisplayType::Dowsiness => draw_drowsiness_view(muse_model, window),
        DisplayType::Emotion => draw_emotion_view(muse_model, window),
        DisplayType::EegValues => draw_eeg_values_view(muse_model, window),
    }
}

/// A bigger yellow circle indiates greater happiness. Maybe.
fn draw_emotion_view(model: &MuseModel, window: &mut Window) {
    // let global_theta = muse_model::average_from_four_electrodes(&model.theta);
    // let asymm = super.valence(&model.alpha, &model.theta);
    // let arousal_index = arousal_index(&model.theta, &model.alpha);

    // //TODO Change this to Mandala display

    // draw_polygon(&COLOR_EMOTION, asymm / 5.0, window, model.scale, (0.0, 0.0));
}

fn draw_drowsiness_view(model: &MuseModel, window: &mut Window) {
    // let lizard_mind = (average_from_four_electrodes(&model.theta)
    //     + average_from_four_electrodes(&model.delta))
    //     / 2.0;
    // draw_polygon(&COLOR_THETA, lizard_mind, window, model.scale, (0.0, 0.0));
    // draw_polygon(
    //     &COLOR_ALPHA,
    //     average_from_four_electrodes(&model.alpha),
    //     window,
    //     model.scale,
    //     (0.0, 0.0),
    // );
}

fn draw_four_circles_view(model: &MuseModel, window: &mut Window) {
    const DISTANCE: f32 = 100.0;
    const LEFT_FRONT: (f32, f32) = (-DISTANCE, -DISTANCE);
    const RIGHT_FRONT: (f32, f32) = (DISTANCE, -DISTANCE);
    const RIGHT_REAR: (f32, f32) = (DISTANCE, DISTANCE);
    const LEFT_REAR: (f32, f32) = (-DISTANCE, DISTANCE);

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

struct GraphLabel {
    image: Asset<Image>,
    position: (f64, f64),
}

impl GraphLabel {
    fn new(text: String, position: (f64, f64)) -> Self {
        let image = Asset::new(Font::load(FONT_MULI).and_then(move |font| {
            result(font.render(&text, &FontStyle::new(FONT_EEG_LABEL_SIZE, COLOR_EEG_LABEL)))
        }));

        Self { image, position }
    }
}

/// A set of all EEG values displayed for diagnostic purposes
fn draw_eeg_values_view(muse_model: &MuseModel, window: &mut Window) {
    assert!(FIRST_EEG_CHANNEL <= N_EEG_CHANNELS);
    assert!(FIRST_EEG_DERIVED_VALUE <= N_EEG_DERIVED_VALUES);
    assert!(N_EEG_DERIVED_VALUES <= EEG_COLORS.len());
    assert!(N_EEG_DERIVED_VALUES <= EEG_LABELS.len());

    let mut shift = [(0.0, 0.0); 4];
    let mut spider_values = [[0.0; 5]; 4];
    let mut axis_label: [Asset<Image>; 5] = [
        Asset::new(Font::load(FONT_MULI).and_then(|font| {
            result(font.render(
                EEG_CHANNEL_LABELS[0],
                &FontStyle::new(FONT_EEG_LABEL_SIZE, COLOR_EEG_LABEL),
            ))
        })),
        Asset::new(Font::load(FONT_MULI).and_then(|font| {
            result(font.render(
                EEG_CHANNEL_LABELS[1],
                &FontStyle::new(FONT_EEG_LABEL_SIZE, COLOR_EEG_LABEL),
            ))
        })),
        Asset::new(Font::load(FONT_MULI).and_then(|font| {
            result(font.render(
                EEG_CHANNEL_LABELS[2],
                &FontStyle::new(FONT_EEG_LABEL_SIZE, COLOR_EEG_LABEL),
            ))
        })),
        Asset::new(Font::load(FONT_MULI).and_then(|font| {
            result(font.render(
                EEG_CHANNEL_LABELS[3],
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

        draw_spider_graph(
            chan,
            &mut axis_label,
            &EEG_COLORS,
            spider_values[chan],
            window,
        );
    }

    // Draw current arousal and valence values
}

/// Put a circle on screen, manually scaled based on screen size and 'scale' factor, shifted from screen center by 'shift'
fn draw_spider_graph(
    chan: usize,
    label_images: &mut [Asset<Image>],
    line_color: &[Color],
    value: [f32; 5],
    window: &mut Window,
) {
    let mut position: [Vector; 5] = [
        Vector { x: 0.0, y: 0.0 },
        Vector { x: 0.0, y: 0.0 },
        Vector { x: 0.0, y: 0.0 },
        Vector { x: 0.0, y: 0.0 },
        Vector { x: 0.0, y: 0.0 },
    ];
    let mut angle = [0.0; 5];

    for val in FIRST_EEG_DERIVED_VALUE..N_EEG_DERIVED_VALUES {
        angle[val] = ((val as f32 * 2. * PI) - (PI / 2.))
            / (N_EEG_DERIVED_VALUES - FIRST_EEG_DERIVED_VALUE) as f32;
    }

    // Calculate graph endpoints
    for val in FIRST_EEG_DERIVED_VALUE..N_EEG_DERIVED_VALUES {
        let radius = value[val] / 10.0; //TODO Bound the values better
        let (x, y) = end_of_spider_graph(chan, radius, angle[val]);
        position[val] = SPIDER_GRAPH_POSITIONS[chan] + Vector { x, y };
    }

    // Label the graph
    &label_images[chan].execute(|image| {
        window.draw(
            &image
                .area()
                .with_center(SPIDER_GRAPH_POSITIONS[chan] + SPIDER_GRAPH_LABEL_OFFSET),
            Img(&image),
        );
        Ok(())
    });

    // Draw axis lines for each spider graph
    for val in FIRST_EEG_DERIVED_VALUE..N_EEG_DERIVED_VALUES {
        // Draw from center to outside edge of spider graph
        let center = end_of_spider_graph(chan, 0.0, angle[val]);
        let tip = end_of_spider_graph(chan, SPIDER_GRAPH_AXIS_LENGTH, angle[val]);
        window.draw(
            &Line::new(center, tip).with_thickness(SPIDER_LINE_AXIS_THICKNESS),
            Col(COLOR_SPIDER_GRAPH),
        );

        // Draw outside border of spider graph
        let wrap_val = wrap_eeg_derived_value_index(val);
        let next_spoke_tip = end_of_spider_graph(chan, SPIDER_GRAPH_AXIS_LENGTH, angle[wrap_val]);
        window.draw(
            &Line::new(tip, next_spoke_tip).with_thickness(SPIDER_LINE_AXIS_THICKNESS),
            Col(COLOR_SPIDER_GRAPH),
        );

        // Draw spider graph tip connection shape
        window.draw(
            &Line::new(position[val], position[wrap_val]).with_thickness(SPIDER_LINE_THICKNESS),
            Col(COLOR_SPIDER_GRAPH),
        );
    }

    // Label the endpoints
    for val in FIRST_EEG_DERIVED_VALUE..N_EEG_DERIVED_VALUES {
        // Draw the dot at each point on the spider graph
        window.draw(
            &Circle::new(position[val], SPIDER_POINT_RADIUS),
            Col(line_color[val]),
        );

        // Draw the label over the dot
        &label_images[val].execute(|image| {
            window.draw(
                &image.area().with_center(SPIDER_GRAPH_POSITIONS[chan]),
                Img(&image),
            );
            Ok(())
        });
    }
}

// Find the index of the next value with wrap-around
fn wrap_eeg_derived_value_index(i: usize) -> usize {
    ((i + 1) % N_EEG_DERIVED_VALUES) as usize
}

// Find the screen location of a spider graph value
fn end_of_spider_graph(channel: usize, radius: f32, angle: f32) -> (f32, f32) {
    (
        radius * angle.cos() as f32 + SPIDER_GRAPH_POSITIONS[channel].x,
        radius * angle.sin() as f32 + SPIDER_GRAPH_POSITIONS[channel].y,
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

// Measure for 1 minute, "calibration"
// Store max and min and standard deviaition and mean, for assymetry and arrousal
// Compute the assymetry and scale those
// TODO: check the assymetry calculation, mean and standard deviation
// TODO: add the arousal calculation
// 650 pixel high images of complete mandala
// "Arousal" is 5 points, 10 PNG images -> Ivan
// "Valence" is 12 points, 10 PNG images -> Ivan
// "CenterImage" -> Ivan
// Draw valence first. Opaque or translucent -> Paul
// 2min "NegativeSequence" means 25 images per sequence
// 2min   "Breathing exercise"
//        Scale manadla up and down with fixed time for breathing
//        "Now breathe with the mandala"
//         P1 seconds pause
//         X1 seconds in
//         P2 seconds pause
//         X2 seconds out
// 2min "PositiveSequence" means 25 images per sequence
//         Randomize the order (nice to have)
// 2min "FreeRide" - Try to control the mandala
// ExitScreen - "Thank You"
//         Tweetable image, check the script
// Break between images 1-2.5sec (random)
// Show image 5 seconds
