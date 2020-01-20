use nannou_osc::rosc::OscMessage;
use nannou_osc::rosc::OscType;
use std::net::SocketAddr;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug)]
pub struct MuseMessage {
    //TODO Add message receive time here
    pub time: Duration, // Since UNIX_EPOCH, the beginning of 1970
    pub ip_address: SocketAddr,
    pub muse_message_type: MuseMessageType,
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

pub fn parse_muse_packet(addr: SocketAddr, packet: &nannou_osc::Packet) -> Vec<MuseMessage> {
    let mut raw_messages = Vec::new();
    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("System clock is not set correctly");

    //TODO Add current time as message receive time here

    packet.clone().unfold(&mut raw_messages);
    let mut muse_messages = Vec::with_capacity(raw_messages.len());

    for raw_message in raw_messages {
        let muse_message_type_option = parse_muse_message_type(raw_message);
        match muse_message_type_option {
            Some(muse_message_type) => {
                let muse_message = MuseMessage {
                    time: time,
                    ip_address: addr,
                    muse_message_type: muse_message_type,
                };
                muse_messages.push(muse_message);
            }
            None => (),
        };
    }

    muse_messages
}

fn parse_muse_message_type(raw_message: OscMessage) -> Option<MuseMessageType> {
    let service = raw_message.addr.as_ref();
    let args = raw_message
        .clone()
        .args
        .expect("Expected value was not sent by Muse");

    let r = match service {
        "/muse/eeg" => {
            let a = get_float_from_args(0, &args);
            let b = get_float_from_args(0, &args);
            let c = get_float_from_args(0, &args);
            let d = get_float_from_args(0, &args);

            // println!("EEG: [{:#?}, {:#?}, {:#?}, {:#?}]", a, b, c, d);

            Some(MuseMessageType::Eeg {
                a: a,
                b: b,
                c: c,
                d: d,
            })
        }

        "/muse/acc" => Some(MuseMessageType::Accelerometer {
            x: get_float_from_args(0, &args),
            y: get_float_from_args(1, &args),
            z: get_float_from_args(2, &args),
        }),

        "/muse/gyro" => Some(MuseMessageType::Gyro {
            x: get_float_from_args(0, &args),
            y: get_float_from_args(1, &args),
            z: get_float_from_args(2, &args),
        }),

        "/muse/elements/touching_forehead" => Some(MuseMessageType::TouchingForehead {
            touch: get_int_from_args(0, &args) != 0,
        }),

        "/muse/elements/horseshoe" => Some(MuseMessageType::Horseshoe {
            a: get_float_from_args(0, &args),
            b: get_float_from_args(1, &args),
            c: get_float_from_args(2, &args),
            d: get_float_from_args(3, &args),
        }),

        "/muse/elements/alpha_absolute" => Some(MuseMessageType::Alpha {
            a: get_float_from_args(0, &args),
            b: get_float_from_args(1, &args),
            c: get_float_from_args(2, &args),
            d: get_float_from_args(3, &args),
        }),

        "/muse/elements/beta_absolute" => Some(MuseMessageType::Beta {
            a: get_float_from_args(0, &args),
            b: get_float_from_args(1, &args),
            c: get_float_from_args(2, &args),
            d: get_float_from_args(3, &args),
        }),

        "/muse/elements/gamma_absolute" => Some(MuseMessageType::Gamma {
            a: get_float_from_args(0, &args),
            b: get_float_from_args(1, &args),
            c: get_float_from_args(2, &args),
            d: get_float_from_args(3, &args),
        }),

        "/muse/elements/delta_absolute" => Some(MuseMessageType::Delta {
            a: get_float_from_args(0, &args),
            b: get_float_from_args(1, &args),
            c: get_float_from_args(2, &args),
            d: get_float_from_args(3, &args),
        }),

        "/muse/elements/theta_absolute" => Some(MuseMessageType::Theta {
            a: get_float_from_args(0, &args),
            b: get_float_from_args(1, &args),
            c: get_float_from_args(2, &args),
            d: get_float_from_args(3, &args),
        }),

        "/muse/elements/blink" => {
            let blink = get_int_from_args(0, &args);
            //            println!("Blink: {:#?}", blink);

            Some(MuseMessageType::Blink { blink: blink != 0 })
        }

        "/muse/batt" => Some(MuseMessageType::Batt {
            batt: (get_int_from_args(1, &args) as f32 / get_int_from_args(0, &args) as f32) as i32,
        }),

        "/muse/elements/jaw_clench" => Some(MuseMessageType::JawClench {
            clench: get_int_from_args(0, &args) != 0,
        }),

        _ => {
            eprintln!("Unparsed message type: {:#?} {:#?}", service, raw_message);
            None
        }
    };

    r
}

fn get_float_from_args(i: usize, args: &Vec<OscType>) -> f32 {
    let f = args.get(i).expect("Float was not provided");

    match f {
        OscType::Float(value) => *value,
        _ => panic!("Muse value was not a float"),
    }
}

fn get_int_from_args(i: usize, args: &Vec<OscType>) -> i32 {
    let j = args.get(i).expect("Int was not provided");
    match j {
        OscType::Int(value) => *value,
        _ => panic!("Muse value was not an int"),
    }
}
