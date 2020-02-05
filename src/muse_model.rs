#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
use crate::muse_packet::*;
/// Muse data model and associated message handling from muse_packet
// #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
use std::sync::mpsc::SendError;

// use log::*;
use std::net::SocketAddr;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;

const FOREHEAD_COUNTDOWN: i32 = 30; // 60th of a second counts
const BLINK_COUNTDOWN: i32 = 30;
const CLENCH_COUNTDOWN: i32 = 30;

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

    // Make sure this matches the `TARGET_PORT` in the `osc_sender.rs` example.
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

/// Snapshot of the most recently collected values from Muse EEG headset
pub struct MuseModel {
    most_recent_message_receive_time: Duration,
    pub inner_receiver: inner_receiver::InnerMessageReceiver,
    tx_eeg: Sender<(Duration, MuseMessageType)>,
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
}

impl MuseModel {
    /// Create a new model for storing received values
    pub fn new() -> (Receiver<(Duration, MuseMessageType)>, MuseModel) {
        let (tx_eeg, rx_eeg): (
            Sender<(Duration, MuseMessageType)>,
            Receiver<(Duration, MuseMessageType)>,
        ) = mpsc::channel();

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
                display_type: DisplayType::Emotion, // Current drawing mode
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

        for muse_message in muse_messages {
            self.handle_message(&muse_message)
                .expect("Could not send to internal receiver message");
            self.most_recent_message_receive_time = muse_message.time;
        }
    }

    /// Update state based on an incoming message
    fn handle_message(
        &mut self,
        muse_message: &MuseMessage,
    ) -> Result<(), SendError<(Duration, MuseMessageType)>> {
        let time = muse_message.time;

        match muse_message.muse_message_type {
            MuseMessageType::Accelerometer { x, y, z } => {
                self.accelerometer = [x, y, z];
                self.tx_eeg
                    .send((time, MuseMessageType::Accelerometer { x, y, z }))
            }
            MuseMessageType::Gyro { x, y, z } => {
                self.gyro = [x, y, z];
                self.tx_eeg.send((time, MuseMessageType::Gyro { x, y, z }))
            }
            MuseMessageType::Horseshoe { a, b, c, d } => {
                self.horseshoe = [a, b, c, d];
                self.tx_eeg
                    .send((time, MuseMessageType::Horseshoe { a, b, c, d }))
            }
            MuseMessageType::Eeg { a, b, c, d } => self
                .tx_eeg
                .send((time, MuseMessageType::Eeg { a, b, c, d })),
            MuseMessageType::Alpha { a, b, c, d } => {
                self.alpha = [a, b, c, d];
                self.tx_eeg
                    .send((time, MuseMessageType::Alpha { a, b, c, d }))
            }
            MuseMessageType::Beta { a, b, c, d } => {
                self.beta = [a, b, c, d];
                self.tx_eeg
                    .send((time, MuseMessageType::Beta { a, b, c, d }))
            }
            MuseMessageType::Gamma { a, b, c, d } => {
                self.gamma = [a, b, c, d];
                self.tx_eeg
                    .send((time, MuseMessageType::Gamma { a, b, c, d }))
            }
            MuseMessageType::Delta { a, b, c, d } => {
                self.delta = [a, b, c, d];
                self.tx_eeg
                    .send((time, MuseMessageType::Delta { a, b, c, d }))
            }
            MuseMessageType::Theta { a, b, c, d } => {
                self.theta = [a, b, c, d];
                self.tx_eeg
                    .send((time, MuseMessageType::Theta { a, b, c, d }))
            }
            MuseMessageType::Batt { batt } => {
                self.batt = batt;
                self.tx_eeg
                    .send((muse_message.time, MuseMessageType::Batt { batt }))
            }
            MuseMessageType::TouchingForehead { touch } => {
                if !touch {
                    self.touching_forehead_countdown = FOREHEAD_COUNTDOWN;
                }
                self.tx_eeg
                    .send((time, MuseMessageType::TouchingForehead { touch }))
            }
            MuseMessageType::Blink { blink } => {
                if blink {
                    self.blink_countdown = BLINK_COUNTDOWN;
                }
                self.tx_eeg.send((time, MuseMessageType::Blink { blink }))
            }
            MuseMessageType::JawClench { clench } => {
                if clench {
                    self.jaw_clench_countdown = CLENCH_COUNTDOWN;
                }
                self.tx_eeg
                    .send((time, MuseMessageType::JawClench { clench }))
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
