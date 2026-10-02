//! Source-rate HP scaling for `damageNpHpratioLow`.
//!
//! Chaldea commit daf4e0af5ec6b4ae42f3e9410b26ae143aa3d725,
//! lib/app/battle/functions/damage.dart:89-91,179-182 first truncates
//! `(1 - current / max) * Target` as a floating-point expression, then adds
//! it to Value. Integer-rational division differs at some exact boundaries.

use serde::{Deserialize, Serialize};

pub use crate::model::AttackerHp;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LowHpScaling {
    /// Atlas Value, in thousandths, retained without a float round-trip.
    pub source_base_rates: [u32; 5],
    /// Atlas numeric Target, in thousandths; distinct from the enemy target.
    pub coefficients: [u32; 5],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LowHpBreakdown {
    pub base_multiplier: f64,
    pub hp_contribution: f64,
    pub effective_multiplier: f64,
}

impl LowHpScaling {
    pub fn is_valid_for(&self, multipliers: &[f64; 5], rates: &[u16; 5]) -> bool {
        (0..5).all(|level| {
            multipliers[level] == f64::from(self.source_base_rates[level]) / 1000.0
                && (rates[level] != 0 || self.coefficients[level] == 0)
        })
    }

    pub fn breakdown(&self, np_level: u8, hp: AttackerHp) -> Option<LowHpBreakdown> {
        hp.validate().ok()?;
        let level = usize::from(np_level.checked_sub(1)?);
        let base = *self.source_base_rates.get(level)?;
        let coefficient = *self.coefficients.get(level)?;
        // Preserve the reference's division, subtraction, multiplication order.
        let contribution = ((1.0 - hp.ratio()) * f64::from(coefficient)).trunc() as u32;
        Some(LowHpBreakdown {
            base_multiplier: f64::from(base) / 1000.0,
            hp_contribution: f64::from(contribution) / 1000.0,
            effective_multiplier: (f64::from(base) + f64::from(contribution)) / 1000.0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scaling(coefficient: u32) -> LowHpScaling {
        LowHpScaling {
            source_base_rates: [6000; 5],
            coefficients: [coefficient; 5],
        }
    }

    #[test]
    fn full_half_one_and_odd_hp_keep_source_units() {
        let scale = scaling(6000);
        for (current, max, contribution) in [
            (10000, 10000, 0.0),
            (5000, 10000, 3.0),
            (1, 10000, 5.999),
            (2, 3, 2.0),
            (3, 7, 3.428),
        ] {
            let result = scale
                .breakdown(1, AttackerHp::new(current, max).unwrap())
                .unwrap();
            assert_eq!(result.base_multiplier, 6.0);
            assert_eq!(result.hp_contribution, contribution);
            assert!((result.effective_multiplier - (6.0 + contribution)).abs() < 1e-12);
        }
    }

    #[test]
    fn floating_point_boundary_matches_reference_instead_of_rational_arithmetic() {
        // Integer arithmetic would produce 3; Dart's expression produces
        // 2.999999999999999 and toInt truncates to 2.
        let result = scaling(30)
            .breakdown(1, AttackerHp::new(9, 10).unwrap())
            .unwrap();
        assert_eq!(result.hp_contribution, 0.002);
        assert_eq!(result.effective_multiplier, 6.002);
        assert_eq!(
            scaling(6000)
                .breakdown(1, AttackerHp::new(9999, 10000).unwrap())
                .unwrap()
                .hp_contribution,
            0.0
        );
        assert_eq!(
            scaling(6000)
                .breakdown(1, AttackerHp::new(9998, 10000).unwrap())
                .unwrap()
                .hp_contribution,
            0.001
        );
    }

    #[test]
    fn all_valid_hp_values_are_monotonic_and_limits_do_not_overflow() {
        let scale = scaling(20000);
        let mut previous = f64::INFINITY;
        for current in 1..=10001 {
            let result = scale
                .breakdown(5, AttackerHp::new(current, 10001).unwrap())
                .unwrap();
            assert!(result.effective_multiplier <= previous);
            previous = result.effective_multiplier;
        }
        let scale = LowHpScaling {
            source_base_rates: [u32::MAX; 5],
            coefficients: [u32::MAX; 5],
        };
        assert!(
            scale
                .breakdown(1, AttackerHp::new(1, u32::MAX).unwrap())
                .unwrap()
                .effective_multiplier
                > f64::from(u32::MAX) / 1000.0
        );
        assert!(scale.breakdown(0, AttackerHp::new(1, 1).unwrap()).is_none());
        assert!(scale.breakdown(6, AttackerHp::new(1, 1).unwrap()).is_none());
        assert!(
            scale
                .breakdown(1, AttackerHp { current: 0, max: 1 })
                .is_none()
        );
    }
}
