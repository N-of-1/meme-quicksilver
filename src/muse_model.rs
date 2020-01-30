/// Muse data model and associated message handling from muse_packet
#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
use crate::muse_packet::*;

// use log::*;
use std::fmt;
// use std::fmt::Debug;
use std::fmt::Formatter;
use std::net::SocketAddr;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
use nannou_osc as osc;

const FOREHEAD_COUNTDOWN: i32 = 30; // 60th of a second counts
const BLINK_COUNTDOWN: i32 = 30;
const CLENCH_COUNTDOWN: i32 = 30;

// Make sure this matches the `TARGET_PORT` in the `osc_sender.rs` example.
const PORT: u16 = 34254;

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

/// Mose recently collected values from Muse EEG headset
pub struct MuseModel {
    message_receive_time: Duration,
    // rx: Option<T>,
    tx_eeg: Sender<(Duration, MuseMessageType)>,
    rx_eeg: Receiver<(Duration, MuseMessageType)>,
    clicked: bool,
    clear_background: bool,
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
    pub fn new() -> MuseModel {
        let (tx_eeg, rx_eeg): (
            Sender<(Duration, MuseMessageType)>,
            Receiver<(Duration, MuseMessageType)>,
        ) = mpsc::channel();

        info!("Creating model");

        // #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
        // let receiver = Some(
        //     nannou_osc::receiver(PORT)
        //         .expect("Can not bind to port- is another copy of this app already running?"),
        // );

        // #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
        // let receiver = None();

        MuseModel {
            message_receive_time: Duration::from_secs(0),
            // rx: receiver,
            rx_eeg,
            tx_eeg,
            clicked: false,
            clear_background: false,
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
        }
    }

    /// Receive any pending osc packets.
    // pub fn receive_packets(&mut self) {
    //     if let Some(receiver) = self.rx {
    //         let receivables: Vec<(nannou_osc::Packet, std::net::SocketAddr)> =
    //             receiver.try_iter().collect();

    //         for (packet, addr) in receivables {
    //             let muse_messages = parse_muse_packet(addr, &packet);

    //             for muse_message in muse_messages {
    //                 self.handle_message(&muse_message);
    //             }
    //         }
    //     }
    // }

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

    /// Update state based on an incoming message
    pub fn handle_message(&mut self, muse_message: &MuseMessage) {
        match muse_message.muse_message_type {
            MuseMessageType::Accelerometer { x, y, z } => {
                self.accelerometer = [x, y, z];
                self.tx_eeg
                    .send((
                        muse_message.time,
                        MuseMessageType::Accelerometer { x: x, y: y, z: z },
                    ))
                    .expect("Could not tx Accelerometer");
            }
            MuseMessageType::Gyro { x, y, z } => {
                self.gyro = [x, y, z];
                self.tx_eeg
                    .send((
                        muse_message.time,
                        MuseMessageType::Gyro { x: x, y: y, z: z },
                    ))
                    .expect("Could not tx Gyro");
            }
            MuseMessageType::Horseshoe { a, b, c, d } => {
                self.horseshoe = [a, b, c, d];
                self.tx_eeg
                    .send((
                        muse_message.time,
                        MuseMessageType::Horseshoe {
                            a: a,
                            b: b,
                            c: c,
                            d: d,
                        },
                    ))
                    .expect("Could not tx Horeshoe");
            }
            MuseMessageType::Eeg { a, b, c, d } => {
                self.tx_eeg
                    .send((
                        muse_message.time,
                        MuseMessageType::Eeg {
                            a: a,
                            b: b,
                            c: c,
                            d: d,
                        },
                    ))
                    .expect("Could not send tx Eeg");
            }
            MuseMessageType::Alpha { a, b, c, d } => {
                self.alpha = [a, b, c, d];
                self.tx_eeg
                    .send((
                        muse_message.time,
                        MuseMessageType::Alpha {
                            a: a,
                            b: b,
                            c: c,
                            d: d,
                        },
                    ))
                    .expect("Could not send tx Alpha");
            }
            MuseMessageType::Beta { a, b, c, d } => {
                self.beta = [a, b, c, d];
                self.tx_eeg
                    .send((
                        muse_message.time,
                        MuseMessageType::Beta {
                            a: a,
                            b: b,
                            c: c,
                            d: d,
                        },
                    ))
                    .expect("Could not send tx Beta");
            }
            MuseMessageType::Gamma { a, b, c, d } => {
                self.gamma = [a, b, c, d];
                self.tx_eeg
                    .send((
                        muse_message.time,
                        MuseMessageType::Gamma {
                            a: a,
                            b: b,
                            c: c,
                            d: d,
                        },
                    ))
                    .expect("Could not send tx Gamma");
            }
            MuseMessageType::Delta { a, b, c, d } => {
                self.delta = [a, b, c, d];
                self.tx_eeg
                    .send((
                        muse_message.time,
                        MuseMessageType::Delta {
                            a: a,
                            b: b,
                            c: c,
                            d: d,
                        },
                    ))
                    .expect("Could not send tx Delta");
            }
            MuseMessageType::Theta { a, b, c, d } => {
                self.theta = [a, b, c, d];
                self.tx_eeg
                    .send((
                        muse_message.time,
                        MuseMessageType::Theta {
                            a: a,
                            b: b,
                            c: c,
                            d: d,
                        },
                    ))
                    .expect("Could not send tx Theta");
            }
            MuseMessageType::Batt { batt } => {
                self.batt = batt;
                self.tx_eeg
                    .send((muse_message.time, MuseMessageType::Batt { batt: batt }))
                    .expect("Could not tx Batt");
            }
            MuseMessageType::TouchingForehead { touch } => {
                if !touch {
                    self.touching_forehead_countdown = FOREHEAD_COUNTDOWN;
                }
                self.tx_eeg
                    .send((
                        muse_message.time,
                        MuseMessageType::TouchingForehead { touch: touch },
                    ))
                    .expect("Could not tx TouchingForehead");
            }
            MuseMessageType::Blink { blink } => {
                if blink {
                    self.blink_countdown = BLINK_COUNTDOWN;
                }
                self.tx_eeg
                    .send((muse_message.time, MuseMessageType::Blink { blink: blink }))
                    .expect("Could not tx Blink");
            }
            MuseMessageType::JawClench { clench } => {
                if clench {
                    self.jaw_clench_countdown = CLENCH_COUNTDOWN;
                }
                self.tx_eeg
                    .send((
                        muse_message.time,
                        MuseMessageType::JawClench { clench: clench },
                    ))
                    .expect("Could not tx Clench");
            }
        }
    }
}

/// Pull new data from the OSC Socket (if there is one on this build target)
pub fn osc_socket_receive(muse_model: &mut MuseModel) {
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
}
