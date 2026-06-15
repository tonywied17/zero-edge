//! Turning a raw reading into real-world units.

/// A linear map from raw sensor counts to calibrated units.
///
/// Most cheap analog sensors report arbitrary counts - ADC steps, a raw voltage -
/// that mean nothing until they are converted to real units. A [`Calibration`]
/// applies the line `value = scale * raw + offset`. Build it from two readings
/// whose true values are known, then apply it to every sample.
///
/// # Examples
///
/// ```
/// use pamoja_kit::Calibration;
///
/// // A humidity probe reads 0.5 V at 0 % and 2.5 V at 100 %.
/// let humidity = Calibration::two_point(0.5, 0.0, 2.5, 100.0);
/// assert_eq!(humidity.apply(1.5), 50.0);
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Calibration {
    scale: f32,
    offset: f32,
}

impl Calibration {
    /// Builds a calibration from a scale and offset directly.
    ///
    /// # Arguments
    ///
    /// * `scale` - the multiplier applied to a raw reading.
    /// * `offset` - the constant added after scaling.
    ///
    /// # Returns
    ///
    /// The calibration `value = scale * raw + offset`.
    pub fn linear(scale: f32, offset: f32) -> Self {
        Self { scale, offset }
    }

    /// Builds a calibration from two known `(raw, value)` points.
    ///
    /// # Arguments
    ///
    /// * `raw_low` - a raw reading.
    /// * `value_low` - the true value at `raw_low`.
    /// * `raw_high` - another raw reading.
    /// * `value_high` - the true value at `raw_high`.
    ///
    /// # Returns
    ///
    /// The line through both points. If the two raw readings are equal the slope is
    /// undefined, so the calibration falls back to the constant `value_low`.
    pub fn two_point(raw_low: f32, value_low: f32, raw_high: f32, value_high: f32) -> Self {
        let span = raw_high - raw_low;
        let scale = if span == 0.0 {
            0.0
        } else {
            (value_high - value_low) / span
        };
        Self {
            scale,
            offset: value_low - scale * raw_low,
        }
    }

    /// Converts a raw reading into calibrated units.
    ///
    /// # Arguments
    ///
    /// * `raw` - the uncalibrated sensor reading.
    ///
    /// # Returns
    ///
    /// The calibrated value.
    pub fn apply(&self, raw: f32) -> f32 {
        self.scale * raw + self.offset
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_point_maps_its_endpoints_exactly() {
        let calibration = Calibration::two_point(0.5, 0.0, 2.5, 100.0);
        assert!((calibration.apply(0.5) - 0.0).abs() < 1e-4);
        assert!((calibration.apply(2.5) - 100.0).abs() < 1e-4);
    }

    #[test]
    fn two_point_interpolates_linearly() {
        let calibration = Calibration::two_point(0.5, 0.0, 2.5, 100.0);
        assert!((calibration.apply(1.5) - 50.0).abs() < 1e-4);
    }

    #[test]
    fn equal_raw_points_fall_back_to_a_constant() {
        let calibration = Calibration::two_point(1.0, 42.0, 1.0, 99.0);
        assert!((calibration.apply(5.0) - 42.0).abs() < 1e-4);
    }

    #[test]
    fn linear_applies_scale_and_offset() {
        let calibration = Calibration::linear(2.0, -1.0);
        assert!((calibration.apply(3.0) - 5.0).abs() < 1e-4);
    }
}
