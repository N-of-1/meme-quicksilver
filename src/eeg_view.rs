use crate::muse_model::MuseModel;
use crate::*;
use core::f32::consts::PI;

use quicksilver::{
    geom::{Circle, Vector},
    graphics::{Background::Col, Color, Image},
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
pub const EEG_COLORS: [Color; N_EEG_DERIVED_VALUES] = [
    COLOR_ALPHA,
    COLOR_BETA,
    COLOR_GAMMA,
    COLOR_DELTA,
    COLOR_THETA,
];
const EEG_CHANNEL_LABELS: [&str; N_EEG_CHANNELS] = ["TP9", "AF7", "AF8", "TP10"];
const SPIDER_GRAPH_POSITIONS: [Vector; N_EEG_CHANNELS] = [
    // Location of the
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

const EEG_FREQUENCY_BAND_LABELS: [&str; N_EEG_DERIVED_VALUES] = ["A", "B", "G", "D", "T"];

const COLOR_SPIDER_GRAPH: Color = Color::WHITE; // Thin lines marking the axes and outer border
const COLOR_SPIDER_GRAPH_OUTLINE: Color = COLOR_NOF1_TURQOISE; // Thick line connecting dots of the graph values
const N_EEG_CHANNELS: usize = 4;
const N_EEG_DERIVED_VALUES: usize = 5;

const SPIDER_LINE_THICKNESS: f32 = 3.5; // Thickness of the line between points
const SPIDER_LINE_AXIS_THICKNESS: f32 = 1.5; // Thickness of the axis labels
const SPIDER_POINT_RADIUS: f32 = 10.0; // Size of the dot on each graph point
const SPIDER_GRAPH_AXIS_LENGTH: f32 = 200.0; // Distance from center to pentagon tips
const SPIDER_GRAPH_LABEL_OFFSET: Vector = Vector { x: -160., y: -160. }; // Shift labels up and right from the center of the spider graph
const FREQUENCY_LABEL_OFFSET: Vector = Vector { x: 0.5, y: -1.5 }; // Shift letters up slightly to center in the circle
const SPIDER_SCALE: f32 = 150.0; // Make alpha etc larger for display purposes

const IMAGE_SET_SIZE: usize = 10;
struct ImageSet {
    _images: [Asset<Image>; IMAGE_SET_SIZE],
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
        let _images: [Asset<Image>; IMAGE_SET_SIZE] = arr![Asset::new(Image::load(filename(filename_prefix, {
                i = i + 1;
                i - 1
            }))); 10];

        Self { _images }
    }

    fn _draw(&mut self, image_number: usize, window: &mut Window) {
        self._images[image_number]
            .execute(|image| {
                window.draw(
                    &image
                        .area()
                        .with_center((SCREEN_SIZE.0 / 2.0, SCREEN_SIZE.1 / 2.0)),
                    Img(&image),
                );
                Ok(())
            })
            .expect("Can not draw image from ImageSet");
    }
}

pub struct EegViewState {
    touching_forehead_box: LabeledBox,
    blink_box: LabeledBox,
    clench_box: LabeledBox,
    graph_label_images: [Asset<Image>; N_EEG_CHANNELS],
    frequency_label_images: [Asset<Image>; N_EEG_DERIVED_VALUES],
    _calm_ext: ImageSet,
    _pos_neg: ImageSet,
    _valence_index: usize,
    _arousal_index: usize,
}

impl EegViewState {
    pub fn new() -> Self {
        assert!(N_EEG_DERIVED_VALUES == EEG_COLORS.len());
        assert!(N_EEG_DERIVED_VALUES == EEG_FREQUENCY_BAND_LABELS.len());

        let graph_label_images: [Asset<Image>; N_EEG_CHANNELS] = [
            Asset::new(Font::load(FONT_EXTRA_BOLD).and_then(|font| {
                result(font.render(
                    EEG_CHANNEL_LABELS[0],
                    &FontStyle::new(FONT_GRAPH_LABEL_SIZE, COLOR_EEG_LABEL),
                ))
            })),
            Asset::new(Font::load(FONT_EXTRA_BOLD).and_then(|font| {
                result(font.render(
                    EEG_CHANNEL_LABELS[1],
                    &FontStyle::new(FONT_GRAPH_LABEL_SIZE, COLOR_EEG_LABEL),
                ))
            })),
            Asset::new(Font::load(FONT_EXTRA_BOLD).and_then(|font| {
                result(font.render(
                    EEG_CHANNEL_LABELS[2],
                    &FontStyle::new(FONT_GRAPH_LABEL_SIZE, COLOR_EEG_LABEL),
                ))
            })),
            Asset::new(Font::load(FONT_EXTRA_BOLD).and_then(|font| {
                result(font.render(
                    EEG_CHANNEL_LABELS[3],
                    &FontStyle::new(FONT_GRAPH_LABEL_SIZE, COLOR_EEG_LABEL),
                ))
            })),
        ];

        let frequency_label_images: [Asset<Image>; N_EEG_DERIVED_VALUES] = [
            Asset::new(Font::load(FONT_MULI).and_then(|font| {
                result(font.render(
                    EEG_FREQUENCY_BAND_LABELS[0],
                    &FontStyle::new(FONT_EEG_LABEL_SIZE, COLOR_EEG_LABEL),
                ))
            })),
            Asset::new(Font::load(FONT_MULI).and_then(|font| {
                result(font.render(
                    EEG_FREQUENCY_BAND_LABELS[1],
                    &FontStyle::new(FONT_EEG_LABEL_SIZE, COLOR_EEG_LABEL),
                ))
            })),
            Asset::new(Font::load(FONT_MULI).and_then(|font| {
                result(font.render(
                    EEG_FREQUENCY_BAND_LABELS[2],
                    &FontStyle::new(FONT_EEG_LABEL_SIZE, COLOR_EEG_LABEL),
                ))
            })),
            Asset::new(Font::load(FONT_MULI).and_then(|font| {
                result(font.render(
                    EEG_FREQUENCY_BAND_LABELS[3],
                    &FontStyle::new(FONT_EEG_LABEL_SIZE, COLOR_EEG_LABEL),
                ))
            })),
            Asset::new(Font::load(FONT_MULI).and_then(|font| {
                result(font.render(
                    EEG_FREQUENCY_BAND_LABELS[4],
                    &FontStyle::new(FONT_EEG_LABEL_SIZE, COLOR_EEG_LABEL),
                ))
            })),
        ];

        Self {
            touching_forehead_box: LabeledBox::new(
                "Forehead",
                Vector::new(200., 500.),
                Vector::new(200., 50.),
                Color::RED,
                COLOR_BACKGROUND,
                COLOR_TEXT,
            ),
            blink_box: LabeledBox::new(
                "Blink",
                Vector::new(500., 500.),
                Vector::new(200., 50.),
                Color::BLUE,
                COLOR_BACKGROUND,
                COLOR_TEXT,
            ),
            clench_box: LabeledBox::new(
                "Jaw Clench",
                Vector::new(800., 500.),
                Vector::new(200., 50.),
                Color::BLUE,
                COLOR_BACKGROUND,
                COLOR_TEXT,
            ),
            graph_label_images,
            frequency_label_images,
            _calm_ext: ImageSet::new("calm_ex"),
            _pos_neg: ImageSet::new("pos_neg"),
            _valence_index: 5,
            _arousal_index: 5,
        }
    }
}

/// Render concenctric circules associated with alpha, beta, gamma..
pub fn draw_view(muse_model: &MuseModel, window: &mut Window, eeg_view_state: &mut EegViewState) {
    match muse_model.display_type {
        DisplayType::Mandala => draw_mandala_view(muse_model, window, eeg_view_state),
        DisplayType::Dowsiness => draw_drowsiness_view(muse_model, window),
        DisplayType::Emotion => draw_emotion_sun_view(muse_model, window),
        DisplayType::EegValues => draw_eeg_values_view(muse_model, window, eeg_view_state),
    }
}

/// A bigger yellow circle indiates greater happiness. Maybe.
fn draw_emotion_sun_view(model: &MuseModel, window: &mut Window) {
    let asymm = model.calc_absolute_valence();

    draw_circle(&COLOR_EMOTION, asymm / 5.0, window, model.scale, (0.0, 0.0));
}

fn draw_drowsiness_view(model: &MuseModel, window: &mut Window) {
    let lizard_mind = (muse_model::average_from_front_electrodes(&model.theta)
        + muse_model::average_from_front_electrodes(&model.delta))
        / 2.0;

    draw_circle(&COLOR_THETA, lizard_mind, window, model.scale, (0.0, 0.0));

    draw_circle(
        &COLOR_ALPHA,
        muse_model::average_from_front_electrodes(&model.alpha),
        window,
        model.scale,
        (0.0, 0.0),
    );
}

// TODO Add maximum slew rate to visualized value for mandala to change "smoothly"

fn _range_raw_values_to_0_to_9(val: f32) -> usize {
    ((val + 3.0) / 0.6).max(0.0).min(9.0) as usize
}

fn draw_mandala_view(_model: &MuseModel, _window: &mut Window, _eeg_view_state: &mut EegViewState) {
    // match (
    //     model.valence.moving_average(),
    //     model.arousal.moving_average(),
    // ) {
    //     (Some(val), Some(arou)) => {

    // // Shift raw values from -3.0 to 3.0 range into 0..9 (camped)
    // let vma = range_raw_values_to_0_to_9(val);
    // let ama = range_raw_values_to_0_to_9(arou);

    // if vma > eeg_view_state.valence_index + 1 {
    //     eeg_view_state.valence_index = (eeg_view_state.valence_index + 1).min(9);
    // } else if vma < eeg_view_state.valence_index - 1 {
    //     eeg_view_state.valence_index = (eeg_view_state.valence_index - 1).max(0);
    // }

    // if ama > eeg_view_state.arousal_index + 1 {
    //     eeg_view_state.arousal_index = (eeg_view_state.arousal_index + 1).min(9);
    // } else if ama < eeg_view_state.valence_index - 1 {
    //     eeg_view_state.arousal_index = (eeg_view_state.arousal_index - 1).max(0);
    // }
    // // Draw from original image set
    // eeg_view_state
    //     .pos_neg
    //     .draw(eeg_view_state.valence_index, window);
    // eeg_view_state
    //     .calm_ext
    //     .draw(eeg_view_state.arousal_index, window);
    // }
    // _ => draw_eeg_values_view(model, window, eeg_view_state), // Nothing to display- help the user setup
    // };
}

/// Put a circle on screen, manually scaled based on screen size and 'scale' factor, shifted from screen center by 'shift'
fn draw_circle(line_color: &Color, value: f32, window: &mut Window, scale: f32, shift: (f32, f32)) {
    let screen_size = window.screen_size();
    let scale = screen_size.x / scale;
    let radius = value * scale;
    let x = (screen_size.x / 2.0) + shift.0;
    let y = (screen_size.y / 2.0) + shift.1;

    window.draw(&Circle::new((x, y), radius), Col(*line_color));
}

/// A set of all EEG values displayed for diagnostic purposes
fn draw_eeg_values_view(
    muse_model: &MuseModel,
    window: &mut Window,
    eeg_view_state: &mut EegViewState,
) {
    for chan in 0..N_EEG_CHANNELS {
        let mut spider_values = [0.0; 5];
        spider_values[0] = SPIDER_SCALE * muse_model.alpha[chan];
        spider_values[1] = SPIDER_SCALE * muse_model.beta[chan];
        spider_values[2] = SPIDER_SCALE * muse_model.gamma[chan];
        spider_values[3] = SPIDER_SCALE * muse_model.delta[chan];
        spider_values[4] = SPIDER_SCALE * muse_model.theta[chan];

        draw_spider_graph(
            chan,
            &mut eeg_view_state.graph_label_images,
            &mut eeg_view_state.frequency_label_images,
            &EEG_COLORS,
            spider_values,
            window,
        );
    }

    // Draw current Muse headset state
    eeg_view_state
        .touching_forehead_box
        .draw(muse_model.is_touching_forehead(), window);
    eeg_view_state.blink_box.draw(muse_model.is_blink(), window);
    eeg_view_state
        .clench_box
        .draw(muse_model.is_jaw_clench(), window);
}

/// Put five circles on screen in a pentagon shape, bouncing outward from the center based on EEG frequency band intensity
fn draw_spider_graph(
    chan: usize,
    graph_label_images: &mut [Asset<Image>; N_EEG_CHANNELS],
    frequency_label_images: &mut [Asset<Image>; N_EEG_DERIVED_VALUES],
    line_color: &[Color],
    spider_values: [f32; 5],
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

    for val in 0..N_EEG_DERIVED_VALUES {
        angle[val] = ((val as f32 * 2. * PI) - (PI / 2.)) / (N_EEG_DERIVED_VALUES) as f32;
    }

    // Calculate graph endpoints
    for val in 0..N_EEG_DERIVED_VALUES {
        let radius = spider_values[val]; //TODO Bound the values better
        let (x, y) = end_of_spider_graph(chan, radius, angle[val]);
        position[val] = Vector { x, y };
    }

    // Label the graph
    let _result = graph_label_images[chan].execute(|image| {
        window.draw(
            &image
                .area()
                .with_center(SPIDER_GRAPH_POSITIONS[chan] + SPIDER_GRAPH_LABEL_OFFSET),
            Img(&image),
        );
        Ok(())
    });

    // Draw axis lines for each spider graph
    for val in 0..N_EEG_DERIVED_VALUES {
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

        // Draw lines between spider graph tips to create a shifting shape
        window.draw(
            &Line::new(position[val], position[wrap_val]).with_thickness(SPIDER_LINE_THICKNESS),
            Col(COLOR_SPIDER_GRAPH_OUTLINE),
        );
    }

    // Label the endpoints
    for val in 0..N_EEG_DERIVED_VALUES {
        // Draw the dot at each point on the spider graph
        window.draw(
            &Circle::new(position[val], SPIDER_POINT_RADIUS),
            Col(line_color[val]),
        );

        // Draw the label over the dot
        let _result = frequency_label_images[val].execute(|image| {
            window.draw(
                &image
                    .area()
                    .with_center(position[val] + FREQUENCY_LABEL_OFFSET),
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

/// A rectangular screen area with text label which changes background color ACTIVE and INACTIVE using a bound function
pub struct LabeledBox {
    position: Vector,
    size: Vector,
    active_color: Color,
    inactive_color: Color,
    label_image: Asset<Image>,
}

impl LabeledBox {
    pub fn new(
        label: &'static str,
        position: Vector,
        size: Vector,
        active_color: Color,
        inactive_color: Color,
        text_color: Color,
    ) -> Self {
        let label_image = Asset::new(Font::load(FONT_EXTRA_BOLD).and_then(move |font| {
            result(font.render(label, &FontStyle::new(FONT_GRAPH_LABEL_SIZE, text_color)))
        }));

        Self {
            position,
            size,
            active_color,
            inactive_color,
            label_image,
        }
    }

    fn draw(&mut self, active: bool, window: &mut Window) {
        let background_color = match active {
            true => self.active_color,
            false => self.inactive_color,
        };

        window.draw(&Rectangle::new(self.position, self.size), background_color);

        let pos = self.position + self.size / 2.0;
        &self.label_image.execute(|image| {
            window.draw(&image.area().with_center(pos), Img(&image));
            Ok(())
        });
    }
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

    #[test]
    fn test_range_small_number() {
        let expected = 0;
        let val = -100.0;

        assert_eq!(_range_raw_values_to_0_to_9(val), expected);
    }

    #[test]
    fn test_range_large_number() {
        let expected = 9;
        let val = 100.0;

        assert_eq!(_range_raw_values_to_0_to_9(val), expected);
    }

    #[test]
    fn test_range_middle_number() {
        let expected = 5;
        let val = 0.0;

        assert_eq!(_range_raw_values_to_0_to_9(val), expected);
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
