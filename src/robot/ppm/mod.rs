use arduino_hal::Peripherals;
use avr_device::interrupt::{CriticalSection, Mutex};
use core::{
    cell::{Cell, RefCell},
    iter::Enumerate,
};

const MICROSECONDS_PER_TICK: u16 = 4;
const FIRST_CHANNEL_INDEX: u8 = 0;
const MAX_NUM_CHANNELS: usize = 6;
const BLANK_PPM_SIGNAL_IN_MILLISECONDS: u16 = 4000;

static LAST_TICK: Mutex<Cell<u16>> = Mutex::new(Cell::new(1));
static CHANNEL_INDEX: Mutex<Cell<u8>> = Mutex::new(Cell::new(0));
pub static CHANNEL_VALUES: Mutex<RefCell<[u16; MAX_NUM_CHANNELS]>> =
    Mutex::new(RefCell::new([0; MAX_NUM_CHANNELS]));

pub type PositionValue = u16;

pub struct Ppm {}

impl Ppm {
    /// Initializes PPM reading from digital pin D2.
    pub fn init_from_d2(ph: &Peripherals) -> Self {
        ph.TC1.tccr1b.write(|w| w.cs1().prescale_64());
        ph.EXINT.eicra.write(|w| w.isc0().bits(0b11));
        ph.EXINT.eimsk.write(|w| w.int0().set_bit());
        avr_interrupt_enable();
        Self {}
    }

    /// Initializes PPM reading from digital pin D3.
    pub fn init_from_d3(ph: &Peripherals) -> Self {
        ph.TC1.tccr1b.write(|w| w.cs1().prescale_64());
        ph.EXINT.eicra.write(|w| w.isc1().bits(0b11));
        ph.EXINT.eimsk.write(|w| w.int1().set_bit());
        avr_interrupt_enable();
        Self {}
    }

    /// Returns an iterator over the current PPM channel values.
    pub fn get_channels(&self) -> Enumerate<core::array::IntoIter<PositionValue, 6>> {
        avr_device::interrupt::free(|cs| {
            let chanel_values = CHANNEL_VALUES.borrow(cs).borrow();
            chanel_values.into_iter().enumerate()
        })
    }
}

/// Enables global AVR interrupts.
fn avr_interrupt_enable() {
    unsafe {
        avr_device::interrupt::enable();
    }
}

#[avr_device::interrupt(atmega328p)]
fn INT0() {
    let current_ticks = get_ticks_of_timer_counter_1();
    avr_device::interrupt::free(|cs| {
        let time_elapse_in_microseconds =
            get_microseconds_from_last_interruption(cs, current_ticks);
        if is_reading_channels(time_elapse_in_microseconds) {
            process_channel(cs, time_elapse_in_microseconds);
        } else {
            reset_channels(cs);
        }
    });
}

#[avr_device::interrupt(atmega328p)]
fn INT1() {
    let current_ticks = get_ticks_of_timer_counter_1();
    avr_device::interrupt::free(|cs| {
        let time_elapse_in_microseconds =
            get_microseconds_from_last_interruption(cs, current_ticks);
        if is_reading_channels(time_elapse_in_microseconds) {
            process_channel(cs, time_elapse_in_microseconds);
        } else {
            reset_channels(cs);
        }
    });
}

/// Returns the current value of Timer/Counter1 (TCNT1).
fn get_ticks_of_timer_counter_1() -> u16 {
    unsafe { (*avr_device::atmega328p::TC1::ptr()).tcnt1.read().bits() }
}

/// Calculates microseconds elapsed since the last timer interrupt.
/// 1 Tick every 4 microseconds. Returns elapsed microseconds.
fn get_microseconds_from_last_interruption(cs: CriticalSection, current_tick: u16) -> u16 {
    let last_tick = LAST_TICK.borrow(cs).get();
    LAST_TICK.borrow(cs).set(current_tick);
    let elapsed_ticks = current_tick.wrapping_sub(last_tick);
    elapsed_ticks * MICROSECONDS_PER_TICK
}

/// Resets the channel index to the first channel.
fn reset_channels(cs: CriticalSection) {
    CHANNEL_INDEX.borrow(cs).set(FIRST_CHANNEL_INDEX);
}

/// Advances the channel index to the next channel.
fn set_next_channel(cs: CriticalSection, id_channel: usize) {
    if id_channel < MAX_NUM_CHANNELS {
        CHANNEL_INDEX.borrow(cs).set(id_channel as u8 + 1);
    }
}

/// Returns true if the elapsed time indicates a valid channel reading.
fn is_reading_channels(time_elapse_in_microseconds: u16) -> bool {
    time_elapse_in_microseconds < BLANK_PPM_SIGNAL_IN_MILLISECONDS
}

/// Processes a channel value and advances to the next channel.
fn process_channel(cs: CriticalSection, value: u16) {
    let current_id_channel = get_current_channel(cs);
    set_channel_value(cs, current_id_channel, value);
    set_next_channel(cs, current_id_channel);
}

/// Returns the current channel index.
fn get_current_channel(cs: CriticalSection) -> usize {
    CHANNEL_INDEX.borrow(cs).get() as usize
}

/// Sets the value for the specified channel index.
fn set_channel_value(cs: CriticalSection, id_channel: usize, value: u16) {
    if id_channel < MAX_NUM_CHANNELS {
        CHANNEL_VALUES.borrow(cs).borrow_mut()[id_channel] = value;
    }
}
