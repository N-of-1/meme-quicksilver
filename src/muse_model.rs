use crate::muse_packet::*;
#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
use std::iter::{Map, Sum};
/// Muse data model and associated message handling from muse_packet
// #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
use std::sync::mpsc::SendError;

// use log::*;
use num_traits::float::Float;
use std::net::SocketAddr;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::{convert::From, time::Duration};

const FOREHEAD_COUNTDOWN: i32 = 30; // 60th of a second counts
const BLINK_COUNTDOWN: i32 = 30;
const CLENCH_COUNTDOWN: i32 = 30;
const HISTORY_LENGTH: usize = 120; // Used to trunacte ArousalHistory and ValenceHistory length - this is the number of samples in the normalization phase
const TP9: usize = 0; // Muse measurment array index for first electrode
const AF7: usize = 1; // Muse measurment array index for second electrode
const AF8: usize = 2; // Muse measurment array index for third electrode
const TP10: usize = 3; // Muse measurment array index for fourth electrode

/// Make it easier to print out the message receiver object for debug purposes
// struct ReceiverDebug<T> {
//     receiver: osc::Receiver<T>,
// }

// impl Debug for ReceiverDebug<T> {
//     fn fmt(&self, f: &mut Formatter<T>) -> fmt::Result {
//         write!(f, "<Receiver>")
//     }
// }

/// The different display modes supported for live screen updates based on Muse EEG signals
#[derive(Clone, Debug)]
pub enum DisplayType {
    FourCircles,
    Dowsiness,
    Emotion,
    EegValues,
}

#[derive(Clone, Debug)]
pub enum MuseMessageType {
    Eeg { a: f32, b: f32, c: f32, d: f32 }, // microVolts
    Accelerometer { x: f32, y: f32, z: f32 },
    Gyro { x: f32, y: f32, z: f32 },
    Alpha { a: f32, b: f32, c: f32, d: f32 }, // microVolts
    Beta { a: f32, b: f32, c: f32, d: f32 },  // microVolts
    Gamma { a: f32, b: f32, c: f32, d: f32 }, // microVolts
    Delta { a: f32, b: f32, c: f32, d: f32 }, // microVolts
    Theta { a: f32, b: f32, c: f32, d: f32 }, // microVolts
    Batt { batt: i32 },
    Horseshoe { a: f32, b: f32, c: f32, d: f32 },
    TouchingForehead { touch: bool },
    Blink { blink: bool },
    JawClench { clench: bool },
}

pub type TimedMuseMessage = (Duration, MuseMessageType);

#[derive(Clone, Debug)]
pub struct MuseMessage {
    pub time: Duration, // Since UNIX_EPOCH, the beginning of 1970
    pub ip_address: SocketAddr,
    pub muse_message_type: MuseMessageType,
}

/// Receive messages of EEG data from some source (OSC or websockets)
trait EegMessageReceiver {
    fn new() -> inner_receiver::InnerMessageReceiver;
    fn receive_packets(&self) -> Vec<MuseMessage>;
}

/// An OSC USB packet receiver for all platforms except WASM
#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
mod inner_receiver {
    use super::{EegMessageReceiver, MuseMessage};
    use nannou_osc;

    const PORT: u16 = 34254;

    pub struct InnerMessageReceiver {
        receiver: nannou_osc::Receiver,
    }

    impl EegMessageReceiver for InnerMessageReceiver {
        fn new() -> InnerMessageReceiver {
            info!("Connecting to EEG");

            let receiver = nannou_osc::receiver(PORT)
                .expect("Can not bind to port- is another copy of this app already running?");

            InnerMessageReceiver { receiver }
        }

        /// Receive any pending osc packets.
        fn receive_packets(&self) -> Vec<MuseMessage> {
            let receivables: Vec<(nannou_osc::Packet, std::net::SocketAddr)> =
                self.receiver.try_iter().collect();

            let mut muse_messages: Vec<MuseMessage> = Vec::new();

            for (packet, addr) in receivables {
                let mut additional_messages: Vec<MuseMessage> =
                    super::parse_muse_packet(addr, &packet);
                muse_messages.append(&mut additional_messages);
            }

            muse_messages
        }
    }
}

/// A placeholder structure for WASM to avoid dependency on non-existing package issues
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
mod inner_receiver {
    use super::{EegMessageReceiver, MuseMessage};

    /// TODO Receive messages from the server in the web implementation
    pub struct InnerMessageReceiver {}

    impl EegMessageReceiver for InnerMessageReceiver {
        fn new() -> InnerMessageReceiver {
            info!("PLACEHOLDER: Will be indirectly connecting to EEG");

            InnerMessageReceiver {}
        }

        /// Receive any pending osc packets.
        fn receive_packets(&self) -> Vec<MuseMessage> {
            Vec::new()
        }
    }
}

pub struct NormalizedValue<T: Float> {
    current: T,
    min: T,
    max: T,
    mean: Option<T>,
    deviation: Option<T>,
    history: Vec<T>,
}

impl<T> NormalizedValue<T>
where
    T: Float + From<i16>,
{
    pub fn new() -> Self {
        Self {
            current: 0.into(),
            min: T::max_value(),
            max: T::min_value(),
            mean: None,
            deviation: None,
            history: Vec::new(),
        }
    }

    // Set the value if it is a change and a rational number
    pub fn set(&mut self, val: T) -> bool {
        let acceptable_new_value = val.is_finite() && val != self.current;

        if acceptable_new_value {
            self.current = val;
            if val > self.max {
                self.max = val;
            }
            if val < self.min {
                self.min = val;
            }
            self.history.push(val);
            if self.history.len() < HISTORY_LENGTH {
                self.mean = mean(&self.history);
                self.deviation = std_deviation(&self.history, self.mean);
            }
        }

        acceptable_new_value
    }

    pub fn percent_normalization_complete(&self) -> f32 {
        self.history.len() as f32 / HISTORY_LENGTH as f32
    }

    // Return the current value normalized based on the initial calibration period
    pub fn normalized_value(&self) -> Option<T> {
        let mean_and_deviation = (self.mean, self.deviation);

        match mean_and_deviation {
            (Some(mean), Some(deviation)) => Some((self.current - mean) / deviation),
            _ => None,
        }
    }
}

/// Snapshot of the most recently collected values from Muse EEG headset
pub struct MuseModel {
    most_recent_message_receive_time: Duration,
    pub inner_receiver: inner_receiver::InnerMessageReceiver,
    tx_eeg: Sender<TimedMuseMessage>,
    accelerometer: [f32; 3],
    gyro: [f32; 3],
    pub alpha: [f32; 4],
    pub beta: [f32; 4],
    pub gamma: [f32; 4],
    pub delta: [f32; 4],
    pub theta: [f32; 4],
    batt: i32,
    horseshoe: [f32; 4],
    blink_countdown: i32,
    touching_forehead_countdown: i32,
    jaw_clench_countdown: i32,
    pub scale: f32,
    pub display_type: DisplayType,
    pub arousal: NormalizedValue<f32>,
    pub valence: NormalizedValue<f32>,
}

fn std_deviation<T>(data: &Vec<T>, mean: Option<T>) -> Option<T>
where
    T: Float + From<i16>,
{
    match (mean, data.len()) {
        (Some(data_mean), count) if count > 0 => {
            let squared_difference_vec: Vec<T> = data
                .iter()
                .map(|value| {
                    let diff = data_mean - (*value as T);

                    diff * diff
                })
                .collect();

            let variance_sum = sum(&squared_difference_vec) / (count as i16).into();

            Some(variance_sum.sqrt())
        }
        _ => None,
    }
}

fn sum<T>(data: &Vec<T>) -> T
where
    T: Float + From<i16>,
{
    let mut sum: T = 0.into();

    for t in data {
        sum = sum + *t;
    }

    sum
}

fn mean<T>(data: &Vec<T>) -> Option<T>
where
    T: Float + From<i16>,
{
    let count = data.len() as i16;
    let sum = sum(data);

    match count {
        positive if positive > 0 => Some(sum / count.into()),
        _ => None,
    }
}

/// Average the raw values
fn average_from_four_electrodes(x: &[f32; 4]) -> f32 {
    (x[0] + x[1] + x[2] + x[3]) / 4.0
}

impl MuseModel {
    /// Create a new model for storing received values
    pub fn new() -> (Receiver<TimedMuseMessage>, MuseModel) {
        let (tx_eeg, rx_eeg): (Sender<TimedMuseMessage>, Receiver<TimedMuseMessage>) =
            mpsc::channel();

        let inner_receiver = inner_receiver::InnerMessageReceiver::new();

        (
            rx_eeg,
            MuseModel {
                most_recent_message_receive_time: Duration::from_secs(0),
                inner_receiver,
                tx_eeg,
                accelerometer: [0.0, 0.0, 0.0],
                gyro: [0.0, 0.0, 0.0],
                alpha: [0.0, 0.0, 0.0, 0.0], // 7.5-13Hz
                beta: [0.0, 0.0, 0.0, 0.0],  // 13-30Hz
                gamma: [0.0, 0.0, 0.0, 0.0], // 30-44Hz
                delta: [0.0, 0.0, 0.0, 0.0], // 1-4Hz
                theta: [0.0, 0.0, 0.0, 0.0], // 4-8Hz
                batt: 0,
                horseshoe: [0.0, 0.0, 0.0, 0.0],
                blink_countdown: 0,
                touching_forehead_countdown: 0,
                jaw_clench_countdown: 0,
                scale: 1.5, // Make the circles relatively larger or smaller
                display_type: DisplayType::EegValues, // Current drawing mode
                arousal: NormalizedValue::new(),
                valence: NormalizedValue::new(),
            },
        )
    }

    /// User has recently clamped their teeth, creating myoelectric interference so interrupting the EEG signal
    pub fn is_jaw_clench(&self) -> bool {
        self.jaw_clench_countdown > 0
    }

    /// User has recently blinked their eyes, creating myoelectric interference so interrupting the EEG signal
    pub fn is_blink(&self) -> bool {
        self.blink_countdown > 0
    }

    /// The Muse headband is recently positioned to touch the user's forehead
    pub fn is_touching_forehead(&self) -> bool {
        self.touching_forehead_countdown > 0
    }

    /// This is called 60x/sec and allows various temporary display states to time out
    pub fn count_down(&mut self) {
        if self.blink_countdown > 0 {
            self.blink_countdown = self.blink_countdown - 1;
        }

        if self.jaw_clench_countdown > 0 {
            self.jaw_clench_countdown = self.jaw_clench_countdown - 1;
        }

        if self.touching_forehead_countdown > 0 {
            self.touching_forehead_countdown = self.touching_forehead_countdown - 1;
        }
    }

    pub fn receive_packets(&mut self) {
        let muse_messages = self.inner_receiver.receive_packets();
        let mut updated_numeric_values = false;

        for muse_message in muse_messages {
            updated_numeric_values = updated_numeric_values
                || self
                    .handle_message(&muse_message)
                    .expect("Could not receive OSC message");
            self.most_recent_message_receive_time = muse_message.time;
        }

        if updated_numeric_values {
            self.update_arousal();
            self.update_valence();
            println!(
                "%Normalized {}   Abs Valence: {}       Abs Arousal: {}",
                self.valence.percent_normalization_complete(),
                self.valence.current,
                self.arousal.current
            );
            if let (Some(valence), Some(arousal)) = (
                self.valence.normalized_value(),
                self.arousal.normalized_value(),
            ) {
                println!("  N Valence: {}       Arousal: {}", valence, arousal);
            }
        }
    }

    /// Front assymetry- higher values mean more positive mood
    fn front_assymetry(&self) -> f32 {
        let base = std::f32::consts::E;
        base.powf(self.alpha[AF7] - self.alpha[AF8])
    }

    /// Positive-negative balance of emotion
    fn absolute_valence(&self) -> f32 {
        self.front_assymetry() / average_from_four_electrodes(&self.theta)
    }

    /// Level of emotional intensity
    fn abolute_arousal(&self) -> f32 {
        (self.alpha[TP9] + self.alpha[TP10]) / (self.theta[TP9] + self.theta[AF7])
    }

    /// Calculate the current arousal value and add it to the length-limited history
    pub fn update_arousal(&mut self) -> bool {
        let a = self.abolute_arousal();

        self.arousal.set(a)
    }

    /// Calculate the current valence value and add it to the length-limited history
    pub fn update_valence(&mut self) -> bool {
        let v = self.absolute_valence();

        self.valence.set(v)
    }

    fn send(&self, timed_muse_message: TimedMuseMessage) {
        let success = self.tx_eeg.send(timed_muse_message);
        assert!(!success.is_err(), "Can not send message to local receiver");
    }

    /// Update state based on an incoming message
    fn handle_message(
        &mut self,
        muse_message: &MuseMessage,
    ) -> Result<bool, SendError<(Duration, MuseMessageType)>> {
        let time = muse_message.time;

        match muse_message.muse_message_type {
            MuseMessageType::Accelerometer { x, y, z } => {
                self.accelerometer = [x, y, z];
                let _success = self
                    .tx_eeg
                    .send((time, MuseMessageType::Accelerometer { x, y, z }));
                Ok(false)
            }
            MuseMessageType::Gyro { x, y, z } => {
                self.gyro = [x, y, z];
                self.send((time, MuseMessageType::Gyro { x, y, z }));
                Ok(false)
            }
            MuseMessageType::Horseshoe { a, b, c, d } => {
                self.horseshoe = [a, b, c, d];
                self.send((time, MuseMessageType::Horseshoe { a, b, c, d }));
                Ok(false)
            }
            MuseMessageType::Eeg { a, b, c, d } => {
                self.send((time, MuseMessageType::Eeg { a, b, c, d }));
                Ok(false)
            }
            MuseMessageType::Alpha { a, b, c, d } => {
                self.alpha = [a, b, c, d];
                self.send((time, MuseMessageType::Alpha { a, b, c, d }));
                Ok(true)
            }
            MuseMessageType::Beta { a, b, c, d } => {
                self.beta = [a, b, c, d];
                self.send((time, MuseMessageType::Beta { a, b, c, d }));
                Ok(true)
            }
            MuseMessageType::Gamma { a, b, c, d } => {
                self.gamma = [a, b, c, d];
                self.send((time, MuseMessageType::Gamma { a, b, c, d }));
                Ok(true)
            }
            MuseMessageType::Delta { a, b, c, d } => {
                self.delta = [a, b, c, d];
                // println!("Delta {} {} {} {}", a, b, c, d);
                self.send((time, MuseMessageType::Delta { a, b, c, d }));
                Ok(true)
            }
            MuseMessageType::Theta { a, b, c, d } => {
                self.theta = [a, b, c, d];
                // println!("Theta {} {} {} {}", a, b, c, d);
                self.send((time, MuseMessageType::Theta { a, b, c, d }));
                Ok(true)
            }
            MuseMessageType::Batt { batt } => {
                self.batt = batt;
                self.send((muse_message.time, MuseMessageType::Batt { batt }));
                Ok(false)
            }
            MuseMessageType::TouchingForehead { touch } => {
                if !touch {
                    self.touching_forehead_countdown = FOREHEAD_COUNTDOWN;
                }
                self.send((time, MuseMessageType::TouchingForehead { touch }));
                Ok(false)
            }
            MuseMessageType::Blink { blink } => {
                if blink {
                    self.blink_countdown = BLINK_COUNTDOWN;
                }
                self.send((time, MuseMessageType::Blink { blink }));
                Ok(false)
            }
            MuseMessageType::JawClench { clench } => {
                if clench {
                    self.jaw_clench_countdown = CLENCH_COUNTDOWN;
                }
                self.send((time, MuseMessageType::JawClench { clench }));
                Ok(false)
            }
        }
    }
}

// Pull new data from the OSC Socket (if there is one on this build target)
// pub fn osc_socket_receive(muse_model: &mut MuseModel) {
// if let Some(receiver) = muse_model.rx {
//     let receivables: Vec<(nannou_osc::Packet, std::net::SocketAddr)> =
//         receiver.try_iter().collect();

//     for (packet, addr) in receivables {
//         let muse_messages = parse_muse_packet(addr, &packet);

//         for muse_message in muse_messages {
//             muse_model.handle_message(&muse_message);
//         }
//     }
// }
// }
