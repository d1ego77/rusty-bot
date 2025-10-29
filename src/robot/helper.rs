use crate::robot::{
    flysky::{MAX_POSITION, MIN_POSITION, RANGE_MID_POSITION_MAX, RANGE_MID_POSITION_MIN},
    ppm::PositionValue,
};
pub const MAX_POTENCY: u8 = 255;
pub const MIN_POTENCY: u8 = 0;
/// Maps a PositionValue (u16) from the FlySky PPM range to a potency value (u8).
/// Ensures the value is clamped to the expected range before scaling.
pub fn map_value_to_potency(value: PositionValue) -> u8 {
    let in_min: u16 = MIN_POSITION;
    let in_max: u16 = MAX_POSITION;
    let out_min: u16 = MIN_POTENCY as u16;
    let out_max: u16 = MAX_POTENCY as u16;

    // Aseguramos que el valor esté dentro del rango esperado
    let clamped = value.clamp(in_min, in_max);

    let scaled = (clamped - in_min) as u32 * (out_max - out_min) as u32 / (in_max - in_min) as u32
        + out_min as u32;

    scaled as u8
}

/// Maps a u16 value from the FlySky PPM range to a u8 value, applying a central deadzone.
/// Returns 0 for values in the deadzone, otherwise scales to 0-255.
pub fn map_u16_to_u8_2(value: u16) -> u8 {
    let clamped = value.clamp(MIN_POSITION, MAX_POSITION);
    let mid = (MIN_POSITION + MAX_POSITION) / 2;

    if clamped < mid {
        // Tramo izquierdo: MIN_POSITION -> 255, MID_POSITION -> 0
        let scaled = (mid - clamped) as u32 * MAX_POTENCY as u32 / (mid - MIN_POSITION) as u32;
        scaled as u8
    } else {
        // Tramo derecho: MID_POSITION -> 0, MAX_POSITION -> 255
        let scaled = (clamped - mid) as u32 * MAX_POTENCY as u32 / (MAX_POSITION - mid) as u32;
        scaled as u8
    }
}
pub fn map_u16_to_i16(value: u16, _prev: u8) -> i16 {
    let center = 1500.0;
    let deadzone = 50.0; // zona muerta para evitar vibraciones
    let val = value as f32;

    let mapped = if val > center + deadzone {
        // Adelante o derecha
        ((val - (center + deadzone)) / (2000.0 - (center + deadzone)) * 255.0) as i16
    } else if val < center - deadzone {
        // Atrás o izquierda
        -(((center - deadzone) - val) / ((center - deadzone) - 1000.0) * 255.0) as i16
    } else {
        0 // Dentro de la zona muerta
    };

    mapped
}
pub fn map_u16_to_u8(value: u16, last: u8) -> u8 {
    let clamped = value.clamp(MIN_POSITION, MAX_POSITION);
    let mid = (MIN_POSITION + MAX_POSITION) / 2;

    let mut scaled = if clamped < mid {
        // Tramo izquierdo: MIN_POSITION -> 255, MID -> 0
        (mid - clamped) as u32 * MAX_POTENCY as u32 / (mid - MIN_POSITION) as u32
    } else {
        // Tramo derecho: MID -> 0, MAX -> 255
        (clamped - mid) as u32 * MAX_POTENCY as u32 / (MAX_POSITION - mid) as u32
    } as u8;

    // ---- Histeresis: solo cambia si la diferencia es significativa ----
    let threshold = 2; // tolerancia (ajustable)
    if scaled.abs_diff(last) <= threshold {
        scaled = last;
    }

    scaled
}
