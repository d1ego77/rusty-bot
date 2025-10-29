use super::ppm::PositionValue;
use crate::robot::ppm::Ppm;
use arduino_hal::Peripherals;

// Stick positions middle range
pub const RANGE_MID_POSITION_MAX: u16 = 1550;
pub const RANGE_MID_POSITION_MIN: u16 = 1450;

// Stick positions
pub const MAX_POSITION: u16 = 2000;
pub const MIN_POSITION: u16 = 1000;
pub const MID_POSITION: u16 = 1500;

// Channels
const CHANNEL_0: usize = 0;
const CHANNEL_1: usize = 1;
const CHANNEL_2: usize = 2;
const CHANNEL_3: usize = 3;
const CHANNEL_4: usize = 4;
const CHANNEL_5: usize = 5;

pub enum Stick {
    Right(StickMovement),
    Left(StickMovement),
    Vra(StickMovement),
    Vrb(StickMovement),
}

pub struct FlySky {
    pub right: Stick,
    pub left: Stick,
    pub vra: Stick,
    pub vrb: Stick,
}

impl Stick {
    /// Sets the right position value for the stick.
    fn set_right_value(&mut self, value: u16) {
        match self {
            Stick::Right(movement) => movement.right_left = Position::Right(value),
            Stick::Left(movement) => movement.right_left = Position::Right(value),
            Stick::Vra(movement) => movement.right_left = Position::Right(value),
            Stick::Vrb(movement) => movement.right_left = Position::Right(value),
        }
    }

    /// Sets the left position value for the stick.
    fn set_left_value(&mut self, value: u16) {
        match self {
            Stick::Right(movement) => movement.right_left = Position::Left(value),
            Stick::Left(movement) => movement.right_left = Position::Left(value),
            Stick::Vra(movement) => movement.right_left = Position::Left(value),
            Stick::Vrb(movement) => movement.right_left = Position::Left(value),
        }
    }

    /// Sets the up position value for the stick.
    fn set_up_value(&mut self, value: u16) {
        match self {
            Stick::Right(movement) => movement.up_down = Position::Up(value),
            Stick::Left(movement) => movement.up_down = Position::Up(value),
            Stick::Vra(movement) => movement.up_down = Position::Up(value),
            Stick::Vrb(movement) => movement.up_down = Position::Up(value),
        }
    }

    /// Sets the down position value for the stick.
    fn set_down_value(&mut self, value: u16) {
        match self {
            Stick::Right(movement) => movement.up_down = Position::Down(value),
            Stick::Left(movement) => movement.up_down = Position::Down(value),
            Stick::Vra(movement) => movement.up_down = Position::Down(value),
            Stick::Vrb(movement) => movement.up_down = Position::Down(value),
        }
    }

    /// Sets the center position value for the stick.
    fn set_center_value(&mut self, value: u16) {
        match self {
            Stick::Right(movement) => {
                movement.center = Position::Center(value);
            }
            Stick::Left(movement) => {
                movement.center = Position::Center(value);
            }
            Stick::Vra(movement) => movement.center = Position::Center(value),
            Stick::Vrb(movement) => movement.center = Position::Center(value),
        }
    }
}

pub struct StickMovement {
    pub up_down: Position,
    pub right_left: Position,
    pub center: Position,
}

#[allow(dead_code)]
pub enum Position {
    Up(PositionValue),
    Down(PositionValue),
    Left(PositionValue),
    Right(PositionValue),
    Center(PositionValue),
}

pub enum FlySkyPpmPin {
    D2,
    #[allow(dead_code)]
    D3,
}

impl Default for FlySky {
    /// Returns a default FlySky instance with all sticks centered.
    fn default() -> Self {
        Self {
            right: Stick::Right(StickMovement {
                right_left: Position::Center(MID_POSITION),
                up_down: Position::Center(MID_POSITION),
                center: Position::Center(MID_POSITION),
            }),
            left: Stick::Left(StickMovement {
                right_left: Position::Center(MID_POSITION),
                up_down: Position::Center(MIN_POSITION),
                center: Position::Center(MID_POSITION),
            }),
            vra: Stick::Vra(StickMovement {
                right_left: Position::Center(MID_POSITION),
                up_down: Position::Center(MID_POSITION),
                center: Position::Center(MID_POSITION),
            }),
            vrb: Stick::Vrb(StickMovement {
                right_left: Position::Center(MID_POSITION),
                up_down: Position::Center(MID_POSITION),
                center: Position::Center(MID_POSITION),
            }),
        }
    }
}

pub trait StickConverter {
    fn to_flysky(self) -> FlySky;
}

pub struct FlySkyManager {
    ppm: Ppm,
}

impl FlySkyManager {
    /// Initializes the FlySkyManager with the given peripherals and PPM pin.
    pub fn init(peripherals: &Peripherals, flysky_ppm_pin: FlySkyPpmPin) -> Self {
        let current_ppm = match flysky_ppm_pin {
            FlySkyPpmPin::D2 => Ppm::init_from_d2(peripherals),
            FlySkyPpmPin::D3 => Ppm::init_from_d3(peripherals),
        };
        FlySkyManager { ppm: current_ppm }
    }

    /// Returns the current FlySky status by converting PPM channels to stick positions.
    pub fn get_status(&self) -> FlySky {
        self.ppm.get_channels().to_flysky()
    }
}

impl StickConverter for core::iter::Enumerate<core::array::IntoIter<PositionValue, 6>> {
    /// Converts an iterator of PPM channel values to a FlySky status struct.
    fn to_flysky(self) -> FlySky {
        let mut status = FlySky::default();

        for (id_channel, value) in self.into_iter() {
            match id_channel {
                CHANNEL_0 => {
                    if value > RANGE_MID_POSITION_MAX && value <= MAX_POSITION {
                        status.right.set_right_value(value);
                    } else if value < RANGE_MID_POSITION_MIN && value >= MIN_POSITION {
                        status.right.set_left_value(value);
                    } else {
                        status.right.set_center_value(value);
                    }
                }
                CHANNEL_1 => {
                    if value > RANGE_MID_POSITION_MAX {
                        status.right.set_up_value(value);
                    } else if value < RANGE_MID_POSITION_MIN {
                        status.right.set_down_value(value);
                    } else {
                        status.right.set_center_value(value);
                    }
                }
                CHANNEL_2 => {
                    if value > MID_POSITION && value <= MAX_POSITION {
                        status.left.set_up_value(value);
                    } else if value < MID_POSITION && value >= MIN_POSITION {
                        status.left.set_down_value(value);
                    } else {
                        status.left.set_center_value(value);
                    }
                }
                CHANNEL_3 => {
                    if value > RANGE_MID_POSITION_MAX {
                        status.left.set_right_value(value);
                    } else if value < RANGE_MID_POSITION_MIN {
                        status.left.set_left_value(value);
                    } else {
                        status.left.set_center_value(value);
                    }
                }
                CHANNEL_4 => {
                    if value > RANGE_MID_POSITION_MAX {
                        status.vra.set_right_value(value);
                    } else if value < RANGE_MID_POSITION_MIN {
                        status.vra.set_left_value(value);
                    } else {
                        status.vra.set_center_value(value);
                    }
                }
                CHANNEL_5 => {
                    if value > RANGE_MID_POSITION_MAX {
                        status.vrb.set_right_value(value);
                    } else if value < RANGE_MID_POSITION_MIN {
                        status.vrb.set_left_value(value);
                    } else {
                        status.vrb.set_center_value(value);
                    }
                }
                _ => {}
            }
        }
        status
    }
}
