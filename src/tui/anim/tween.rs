//! Time-based tweens. Step with `Tween::advance(dt)` from the frame loop,
//! read `value()` when rendering. Respects `reduce_motion()` — snaps to `end`
//! on first advance.

use super::easing::Ease;
use super::reduce_motion;
use ratatui::style::Color;
use std::time::Duration;

pub trait Lerp: Copy {
    fn lerp(a: Self, b: Self, t: f32) -> Self;
}

impl Lerp for f32 {
    fn lerp(a: f32, b: f32, t: f32) -> f32 {
        a + (b - a) * t
    }
}

impl Lerp for u16 {
    fn lerp(a: u16, b: u16, t: f32) -> u16 {
        let af = a as f32;
        let bf = b as f32;
        (af + (bf - af) * t).round().clamp(0.0, u16::MAX as f32) as u16
    }
}

impl Lerp for Color {
    fn lerp(a: Color, b: Color, t: f32) -> Color {
        match (a, b) {
            (Color::Rgb(ar, ag, ab), Color::Rgb(br, bg, bb)) => Color::Rgb(
                f32::lerp(ar as f32, br as f32, t).round() as u8,
                f32::lerp(ag as f32, bg as f32, t).round() as u8,
                f32::lerp(ab as f32, bb as f32, t).round() as u8,
            ),
            // Non-RGB colors don't interpolate meaningfully — snap at halfway.
            _ => {
                if t < 0.5 {
                    a
                } else {
                    b
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Tween<T: Lerp> {
    pub start: T,
    pub end: T,
    pub duration: Duration,
    pub elapsed: Duration,
    pub ease: Ease,
}

impl<T: Lerp> Tween<T> {
    pub fn new(start: T, end: T, duration: Duration, ease: Ease) -> Self {
        Self {
            start,
            end,
            duration,
            elapsed: Duration::ZERO,
            ease,
        }
    }

    pub fn advance(&mut self, dt: Duration) {
        if reduce_motion() {
            self.elapsed = self.duration;
            return;
        }
        self.elapsed = (self.elapsed + dt).min(self.duration);
    }

    pub fn progress(&self) -> f32 {
        if self.duration.is_zero() {
            return 1.0;
        }
        (self.elapsed.as_secs_f32() / self.duration.as_secs_f32()).clamp(0.0, 1.0)
    }

    pub fn value(&self) -> T {
        let t = self.ease.apply(self.progress());
        T::lerp(self.start, self.end, t)
    }

    pub fn is_done(&self) -> bool {
        self.elapsed >= self.duration
    }

    /// Retarget a tween from its current value to a new end. Useful when the
    /// user re-triggers a transition before it finishes.
    pub fn retarget(&mut self, new_end: T, duration: Duration) {
        self.start = self.value();
        self.end = new_end;
        self.duration = duration;
        self.elapsed = Duration::ZERO;
    }
}

/// Periodic `[0,1]` phase, for looping pulses (REC, LIVE). Returns a triangle
/// wave in `[0,1]` with `period` seconds per full cycle (up-then-down).
pub fn pulse_phase(elapsed_secs: f32, period_secs: f32) -> f32 {
    if period_secs <= 0.0 {
        return 0.0;
    }
    let t = (elapsed_secs / period_secs).fract();
    if t < 0.5 {
        t * 2.0
    } else {
        (1.0 - t) * 2.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_lerp_midpoint() {
        let a = Color::Rgb(0, 0, 0);
        let b = Color::Rgb(100, 200, 50);
        let m = Color::lerp(a, b, 0.5);
        assert_eq!(m, Color::Rgb(50, 100, 25));
    }

    #[test]
    fn tween_advances_and_completes() {
        let mut t = Tween::new(0.0_f32, 10.0, Duration::from_millis(100), Ease::Linear);
        t.advance(Duration::from_millis(50));
        assert!((t.value() - 5.0).abs() < 0.01);
        t.advance(Duration::from_millis(60));
        assert!(t.is_done());
        assert_eq!(t.value(), 10.0);
    }

    #[test]
    fn pulse_is_triangular() {
        assert!((pulse_phase(0.0, 2.0) - 0.0).abs() < 1e-3);
        assert!((pulse_phase(1.0, 2.0) - 1.0).abs() < 1e-3);
        assert!((pulse_phase(2.0, 2.0) - 0.0).abs() < 1e-3);
    }
}
