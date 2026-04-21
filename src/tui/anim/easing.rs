//! Easing curves. All functions take `t` in `[0.0, 1.0]` and return eased
//! `[0.0, 1.0]`. `standard()` is the StriVo default (DESIGN.md:198 —
//! cubic-bezier(0.16, 1, 0.3, 1)).

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Ease {
    Linear,
    InCubic,
    OutCubic,
    InOutSine,
    OutExpo,
    Standard,
}

impl Ease {
    pub fn apply(self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Ease::Linear => t,
            Ease::InCubic => t * t * t,
            Ease::OutCubic => {
                let u = 1.0 - t;
                1.0 - u * u * u
            }
            Ease::InOutSine => 0.5 - 0.5 * (std::f32::consts::PI * t).cos(),
            Ease::OutExpo => {
                if t >= 1.0 {
                    1.0
                } else {
                    1.0 - 2f32.powf(-10.0 * t)
                }
            }
            Ease::Standard => cubic_bezier(0.16, 1.0, 0.3, 1.0, t),
        }
    }
}

/// Approximate a cubic Bezier `y(t)` given control points (x1,y1),(x2,y2) with
/// endpoints (0,0) and (1,1). Uses Newton-Raphson to find parametric `s` such
/// that `bezier_x(s) = t`, then returns `bezier_y(s)`.
pub fn cubic_bezier(x1: f32, y1: f32, x2: f32, y2: f32, t: f32) -> f32 {
    if t <= 0.0 {
        return 0.0;
    }
    if t >= 1.0 {
        return 1.0;
    }
    let bx = |s: f32| 3.0 * (1.0 - s).powi(2) * s * x1 + 3.0 * (1.0 - s) * s * s * x2 + s.powi(3);
    let by = |s: f32| 3.0 * (1.0 - s).powi(2) * s * y1 + 3.0 * (1.0 - s) * s * s * y2 + s.powi(3);
    let dbx = |s: f32| {
        3.0 * (1.0 - s).powi(2) * x1 + 6.0 * (1.0 - s) * s * (x2 - x1) + 3.0 * s * s * (1.0 - x2)
    };

    let mut s = t;
    for _ in 0..8 {
        let x = bx(s) - t;
        let d = dbx(s);
        if d.abs() < 1e-6 {
            break;
        }
        s -= x / d;
        s = s.clamp(0.0, 1.0);
    }
    by(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn endpoints() {
        for ease in [
            Ease::Linear,
            Ease::InCubic,
            Ease::OutCubic,
            Ease::InOutSine,
            Ease::OutExpo,
            Ease::Standard,
        ] {
            assert!((ease.apply(0.0) - 0.0).abs() < 1e-3, "{ease:?} at 0");
            assert!((ease.apply(1.0) - 1.0).abs() < 1e-3, "{ease:?} at 1");
        }
    }

    #[test]
    fn monotonic() {
        // Standard curve should be non-decreasing.
        let mut prev = 0.0;
        for i in 0..=20 {
            let y = Ease::Standard.apply(i as f32 / 20.0);
            assert!(y + 1e-4 >= prev, "non-monotonic at {i}: {prev} -> {y}");
            prev = y;
        }
    }
}
