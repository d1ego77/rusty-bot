mod flysky;
mod helper;
mod ppm;

use crate::robot::flysky::{FlySkyManager, Position, StickMovement};
use arduino_hal::{
    default_serial,
    hal::port::{PB0, PB1, PB2, PB3, PB4, PB5, PC0, PD0, PD1, PD3, PD4, PD5, PD6, PD7},
    pac::{TC0, TC2},
    pins,
    port::{
        mode::{self},
        Pin,
    },
    prelude::*,
    simple_pwm::{IntoPwmPin, Timer0Pwm, Timer2Pwm},
    Peripherals, Usart,
};
use flysky::Stick;

trait StickProcessor {
    /// Processes stick input and updates the robot state.
    fn process(self, robot: &mut Robot);
}

impl StickProcessor for Stick {
    /// Implements stick input processing for the robot.
    fn process(self, robot: &mut Robot) {
        match self {
            Stick::Right(movement) => {
                robot.right_stick_management(movement);
                //robot.throttle_management(&movement.up_down);
                //robot.turn_management(&movement.right_left);
            }
            //   Stick::Left(movement) => {
            //       //robot.potency_management(&movement.up_down);
            //       robot.rotation_management(&movement.right_left);
            //   }
            _ => {}
        }
    }
}

/// Loads and configures timer 0 for PWM.
fn load_timer0_pwm(tc0: TC0) -> Timer0Pwm {
    Timer0Pwm::new(tc0, arduino_hal::simple_pwm::Prescaler::Prescale64)
}

/// Loads and configures timer 2 for PWM.
fn load_timer2_pwm(tc2: TC2) -> Timer2Pwm {
    Timer2Pwm::new(tc2, arduino_hal::simple_pwm::Prescaler::Prescale64)
}

/// Initializes the FlySky manager with the given peripherals.
fn load_flysky_manager(peripherals: &Peripherals) -> FlySkyManager {
    FlySkyManager::init(&peripherals, flysky::FlySkyPpmPin::D2)
}

pub struct MotorA {
    d5: arduino_hal::port::Pin<mode::PwmOutput<Timer0Pwm>, PD5>,
    d4: Pin<arduino_hal::port::mode::Output, PD4>,
    d7: Pin<arduino_hal::port::mode::Output, PD7>,
}

pub struct MotorB {
    d6: arduino_hal::port::Pin<mode::PwmOutput<Timer0Pwm>, PD6>,
    d8: Pin<arduino_hal::port::mode::Output, PB0>,
    d12: Pin<arduino_hal::port::mode::Output, PB4>,
}

pub struct MotorC {
    d11: arduino_hal::port::Pin<mode::PwmOutput<Timer2Pwm>, PB3>,
    d10: Pin<arduino_hal::port::mode::Output, PB2>,
    d9: Pin<arduino_hal::port::mode::Output, PB1>,
}

pub struct MotorD {
    d3: arduino_hal::port::Pin<mode::PwmOutput<Timer2Pwm>, PD3>,
    d13: Pin<arduino_hal::port::mode::Output, PB5>,
    a0: Pin<arduino_hal::port::mode::Output, PC0>,
}

trait Motor {
    fn forward(&mut self, value: u8);
    fn backward(&mut self, value: u8);
    fn stop(&mut self);
}

impl Motor for MotorA {
    fn forward(&mut self, value: u8) {
        self.d5.enable();
        self.d5.set_duty(value);
        self.d4.set_low();
        self.d7.set_high();
    }
    fn backward(&mut self, value: u8) {
        self.d5.enable();
        self.d5.set_duty(value);
        self.d4.set_high();
        self.d7.set_low();
    }
    fn stop(&mut self) {
        self.d5.set_duty(0);
        self.d5.disable();
        self.d4.set_high();
        self.d7.set_high();
    }
}

impl Motor for MotorB {
    fn forward(&mut self, value: u8) {
        self.d6.enable();
        self.d6.set_duty(value);
        self.d8.set_low();
        self.d12.set_high();
    }
    fn backward(&mut self, value: u8) {
        self.d6.enable();
        self.d6.set_duty(value);
        self.d8.set_high();
        self.d12.set_low();
    }
    fn stop(&mut self) {
        self.d6.set_duty(0);
        self.d6.disable();
        self.d8.set_high();
        self.d12.set_high();
    }
}

impl Motor for MotorC {
    fn forward(&mut self, value: u8) {
        self.d11.enable();
        self.d11.set_duty(value);
        self.d10.set_low();
        self.d9.set_high();
    }
    fn backward(&mut self, value: u8) {
        self.d11.enable();
        self.d11.set_duty(value);
        self.d10.set_high();
        self.d9.set_low();
    }
    fn stop(&mut self) {
        self.d11.set_duty(0);
        self.d11.disable();
        self.d10.set_low();
        self.d9.set_low();
    }
}

impl Motor for MotorD {
    fn forward(&mut self, value: u8) {
        self.d3.enable();
        self.d3.set_duty(value);
        self.d13.set_low();
        self.a0.set_high();
    }
    fn backward(&mut self, value: u8) {
        self.d3.enable();
        self.d3.set_duty(value);
        self.d13.set_high();
        self.a0.set_low();
    }
    fn stop(&mut self) {
        self.d3.set_duty(0);
        self.d3.disable();
        self.d13.set_low();
        self.a0.set_low();
    }
}

#[derive(Default)]
pub struct PwmValues {
    pwm_y: u8,
    pwm_x: u8,
}

#[allow(unused)]
pub struct Robot {
    serial: Usart<
        arduino_hal::pac::USART0,
        Pin<arduino_hal::port::mode::Input, PD0>,
        Pin<arduino_hal::port::mode::Output, PD1>,
    >,
    motor_a: MotorA,
    motor_b: MotorB,
    motor_c: MotorC,
    motor_d: MotorD,
    flysky: FlySkyManager,
    tick_duration_us: u32,
    pwm_values: PwmValues,
}

impl Robot {
    /// Creates a new Robot instance and initializes peripherals.
    pub fn new(peripherals: Peripherals, baudrate: u32, tick_duration_us: u32) -> Self {
        // Init PPM protocol of flysky radio control
        let flysky = load_flysky_manager(&peripherals);
        let timer0 = load_timer0_pwm(peripherals.TC0);
        let timer2 = load_timer2_pwm(peripherals.TC2);
        let pins = pins!(peripherals);
        let serial = default_serial!(peripherals, pins, baudrate);

        let motor_a = MotorA {
            d5: pins.d5.into_output().into_pwm(&timer0),
            d4: pins.d4.into_output(),
            d7: pins.d7.into_output(),
        };

        let motor_b = MotorB {
            d6: pins.d6.into_output().into_pwm(&timer0),
            d8: pins.d8.into_output(),
            d12: pins.d12.into_output(),
        };

        let motor_c = MotorC {
            d11: pins.d11.into_output().into_pwm(&timer2),
            d10: pins.d10.into_output(),
            d9: pins.d9.into_output(),
        };

        let motor_d = MotorD {
            d3: pins.d3.into_output().into_pwm(&timer2),
            d13: pins.d13.into_output(),
            a0: pins.a0.into_output(),
        };

        Self {
            serial,
            motor_a,
            motor_b,
            motor_c,
            motor_d,
            flysky,
            tick_duration_us,
            pwm_values: PwmValues::default(),
        }
    }

    /// Processes all FlySky sticks inputs and updates robot state.
    fn process_flysky_sticks(&mut self) {
        let flysky = self.flysky.get_status();
        flysky.left.process(self);
        flysky.right.process(self);
        flysky.vra.process(self);
        flysky.vrb.process(self);
        ufmt::uwrite!(&mut self.serial, "\r\n").unwrap_infallible();
    }

    fn right_stick_management(&mut self, movement: StickMovement) {
        let y = match movement.up_down {
            Position::Up(v) => helper::map_u16_to_i16(v, self.pwm_values.pwm_y), // adelante
            Position::Down(v) => helper::map_u16_to_i16(v, self.pwm_values.pwm_y), // atrás
            _ => 0,
        };

        let x = match movement.right_left {
            Position::Right(v) => helper::map_u16_to_i16(v, self.pwm_values.pwm_x), // derecha lateral
            Position::Left(v) => helper::map_u16_to_i16(v, self.pwm_values.pwm_x), // izquierda lateral
            _ => 0,
        };

        self.pwm_values.pwm_y = y as u8;
        self.pwm_values.pwm_x = x as u8;

        let r = 0; // Todo: Implement left stick

        let mut a = y + x + r;
        let mut b = y - x - r;
        let mut c = y - x + r;
        let mut d = y + x - r;

        // Normalize values from 0–255
        let max = a.abs().max(b.abs().max(c.abs().max(d.abs()))) as f32;
        if max > 255.0 {
            let scale = 255.0 / max;
            a = (a as f32 * scale) as i16;
            b = (b as f32 * scale) as i16;
            c = (c as f32 * scale) as i16;
            d = (d as f32 * scale) as i16;
        }

        ufmt::uwrite!(self.serial, "a: {}, b: {}, c: {}, d: {}", a, b, c, d).unwrap_infallible();
        // Apply direction and magnitud of each motor
        apply_motor(&mut self.motor_a, a);
        apply_motor(&mut self.motor_b, b);
        apply_motor(&mut self.motor_c, c);
        apply_motor(&mut self.motor_d, d);
    }

    /// Starts the robot's main loop, processing inputs and updating state.
    pub fn start(&mut self) -> ! {
        loop {
            self.process_flysky_sticks();
            arduino_hal::delay_us(self.tick_duration_us);
        }
    }
}
fn apply_motor(motor: &mut impl Motor, value: i16) {
    if value > 0 {
        motor.forward(value as u8);
    } else if value < 0 {
        motor.backward((-value) as u8);
    } else {
        motor.stop();
    }
}
