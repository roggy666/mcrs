use core::f64::consts::{PI, TAU};

pub const DAY_LENGTH: i64 = 24_000;
pub const SKY_DARKEN_STEPS: f64 = 11.0;
pub const BRIGHT_OUTSIDE_BELOW: u8 = 4;
const WEATHER_DARKEN: f64 = 5.0 / 16.0;

#[must_use]
pub fn sun_angle(ticks: i64) -> f64 {
    let fraction = (ticks.rem_euclid(DAY_LENGTH) as f64 / DAY_LENGTH as f64 - 0.25).rem_euclid(1.0);
    let eased = 1.0 - ((fraction * PI).cos() + 1.0) / 2.0;
    let angle = fraction + (eased - fraction) / 3.0;
    angle * TAU
}

#[must_use]
pub fn sky_darken(ticks: i64, rain: f64, thunder: f64) -> u8 {
    let daylight = 0.5 + 2.0 * sun_angle(ticks).cos().clamp(-0.25, 0.25);
    let daylight = daylight * (1.0 - rain * WEATHER_DARKEN) * (1.0 - thunder * WEATHER_DARKEN);
    ((1.0 - daylight) * SKY_DARKEN_STEPS) as u8
}

#[must_use]
pub fn is_bright_outside(darken: u8) -> bool {
    darken < BRIGHT_OUTSIDE_BELOW
}

#[must_use]
pub fn sun_brightness(darken: u8) -> f64 {
    1.0 - f64::from(darken) / SKY_DARKEN_STEPS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_sun_is_up_by_day_and_down_at_midnight() {
        assert_eq!(sky_darken(0, 0.0, 0.0), 0);
        assert_eq!(sky_darken(6000, 0.0, 0.0), 0);
        assert!(is_bright_outside(sky_darken(12_000, 0.0, 0.0)));
        assert!(!is_bright_outside(sky_darken(13_500, 0.0, 0.0)));
        assert_eq!(sky_darken(18_000, 0.0, 0.0), 11);
        assert!(!is_bright_outside(sky_darken(18_000, 0.0, 0.0)));
        assert!(is_bright_outside(sky_darken(23_500, 0.0, 0.0)));
    }

    #[test]
    fn rain_darkens_the_sky_by_a_fraction() {
        assert_eq!(sky_darken(6000, 1.0, 0.0), 3);
        assert_eq!(sky_darken(6000, 1.0, 1.0), 5);
    }
}
