//! `lau-camera` — a 2D camera system for games.
//!
//! Provides viewport projection, smooth following, screen shake,
//! animated transitions between states, and a controller that
//! composes them all together.
//!
//! # Example
//!
//! ```
//! use lau_camera::{Camera, Vec2};
//!
//! let mut cam = Camera::new(800.0, 600.0);
//! let world = Vec2::new(100.0, 50.0);
//! let screen = cam.world_to_screen(world);
//! let back = cam.screen_to_world(screen);
//! assert!((back.x - world.x).abs() < 1e-9);
//! assert!((back.y - world.y).abs() < 1e-9);
//! ```

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Vec2
// ---------------------------------------------------------------------------

/// A 2D vector.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

impl Vec2 {
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub const fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    /// Euclidean distance to another vector.
    pub fn distance(self, other: Self) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }

    /// Linear interpolation toward `target` by factor `t` (clamped 0..=1).
    pub fn lerp(self, target: Self, t: f64) -> Self {
        let t = t.clamp(0.0, 1.0);
        Self {
            x: self.x + (target.x - self.x) * t,
            y: self.y + (target.y - self.y) * t,
        }
    }
}

impl std::ops::Add for Vec2 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self { x: self.x + rhs.x, y: self.y + rhs.y }
    }
}

impl std::ops::Sub for Vec2 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self { x: self.x - rhs.x, y: self.y - rhs.y }
    }
}

impl std::ops::Mul<f64> for Vec2 {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self {
        Self { x: self.x * rhs, y: self.y * rhs }
    }
}

// ---------------------------------------------------------------------------
// Rect
// ---------------------------------------------------------------------------

/// An axis-aligned rectangle.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl Rect {
    pub const fn new(x: f64, y: f64, w: f64, h: f64) -> Self {
        Self { x, y, w, h }
    }

    /// Does the rect contain a point (inclusive of edges on all four sides)?
    pub fn contains(&self, point: Vec2) -> bool {
        point.x >= self.x
            && point.x <= self.x + self.w
            && point.y >= self.y
            && point.y <= self.y + self.h
    }

    /// Center point of the rectangle.
    pub fn center(&self) -> Vec2 {
        Vec2 {
            x: self.x + self.w / 2.0,
            y: self.y + self.h / 2.0,
        }
    }
}

// ---------------------------------------------------------------------------
// Camera
// ---------------------------------------------------------------------------

/// A 2D camera that maps between world and screen coordinates.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Camera {
    pub position: Vec2,
    pub zoom: f64,
    pub rotation: f64,
    pub viewport_width: f64,
    pub viewport_height: f64,
}

impl Camera {
    /// Create a new camera centered at the origin with zoom 1.0 and no rotation.
    pub fn new(viewport_width: f64, viewport_height: f64) -> Self {
        Self {
            position: Vec2::zero(),
            zoom: 1.0,
            rotation: 0.0,
            viewport_width,
            viewport_height,
        }
    }

    /// Transform a world-space point to screen space.
    ///
    /// The camera's position is the world-space point that maps to the
    /// centre of the viewport.
    pub fn world_to_screen(&self, world_pos: Vec2) -> Vec2 {
        let dx = world_pos.x - self.position.x;
        let dy = world_pos.y - self.position.y;

        let (sin, cos) = self.rotation.sin_cos();
        let rx = dx * cos - dy * sin;
        let ry = dx * sin + dy * cos;

        Vec2 {
            x: rx * self.zoom + self.viewport_width / 2.0,
            y: ry * self.zoom + self.viewport_height / 2.0,
        }
    }

    /// Transform a screen-space point to world space.
    pub fn screen_to_world(&self, screen_pos: Vec2) -> Vec2 {
        let dx = screen_pos.x - self.viewport_width / 2.0;
        let dy = screen_pos.y - self.viewport_height / 2.0;

        let inv_z = 1.0 / self.zoom;
        let ux = dx * inv_z;
        let uy = dy * inv_z;

        let (sin, cos) = (-self.rotation).sin_cos();
        let rx = ux * cos - uy * sin;
        let ry = ux * sin + uy * cos;

        Vec2 {
            x: rx + self.position.x,
            y: ry + self.position.y,
        }
    }

    /// Returns `(top_left, bottom_right)` of the visible world rectangle.
    pub fn visible_bounds(&self) -> (Vec2, Vec2) {
        let tl = self.screen_to_world(Vec2::new(0.0, 0.0));
        let br = self.screen_to_world(Vec2::new(self.viewport_width, self.viewport_height));
        (tl, br)
    }

    /// Returns `true` when `pos` (in world coords) is visible within the
    /// viewport, padded by `margin` world units.
    pub fn is_visible(&self, pos: Vec2, margin: f64) -> bool {
        let (tl, br) = self.visible_bounds();

        let left = tl.x.min(br.x) - margin;
        let right = tl.x.max(br.x) + margin;
        let top = tl.y.min(br.y) - margin;
        let bottom = tl.y.max(br.y) + margin;

        pos.x >= left && pos.x <= right && pos.y >= top && pos.y <= bottom
    }

    pub fn set_position(&mut self, pos: Vec2) {
        self.position = pos;
    }

    pub fn set_zoom(&mut self, zoom: f64) {
        self.zoom = zoom;
    }

    pub fn set_rotation(&mut self, rot: f64) {
        self.rotation = rot;
    }

    /// Move the camera so `target` is at the centre of the viewport.
    pub fn center_on(&mut self, target: Vec2) {
        self.position = target;
    }
}

// ---------------------------------------------------------------------------
// CameraFollow
// ---------------------------------------------------------------------------

/// Smoothly follows a target position with deadzone and look-ahead.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraFollow {
    pub target: Option<Vec2>,
    pub follow_speed: f64,
    pub deadzone: Rect,
    pub look_ahead: f64,
    /// Velocity estimate for look-ahead.
    #[serde(skip)]
    prev_target: Option<Vec2>,
}

impl CameraFollow {
    /// Create a new follower with the given speed and deadzone.
    pub fn new(follow_speed: f64, deadzone: Rect, look_ahead: f64) -> Self {
        Self {
            target: None,
            follow_speed,
            deadzone,
            look_ahead,
            prev_target: None,
        }
    }

    pub fn set_target(&mut self, pos: Vec2) {
        self.target = Some(pos);
    }

    pub fn clear_target(&mut self) {
        self.target = None;
        self.prev_target = None;
    }

    /// Advance the follower by `dt` seconds, smoothly moving the camera.
    ///
    /// If the target is inside the deadzone the camera stays still.
    /// Otherwise it moves toward the target (with velocity-based look-ahead)
    /// at `follow_speed`.
    pub fn update(&mut self, camera: &mut Camera, dt: f64) {
        let Some(target) = self.target else {
            self.prev_target = None;
            return;
        };

        let offset = Vec2 {
            x: target.x - camera.position.x,
            y: target.y - camera.position.y,
        };

        if self.deadzone.contains(offset) {
            self.prev_target = Some(target);
            return;
        }

        let look = if let Some(prev) = self.prev_target {
            let vx = (target.x - prev.x) / dt.max(1e-9);
            let vy = (target.y - prev.y) / dt.max(1e-9);
            let speed = (vx * vx + vy * vy).sqrt();
            if speed > 1.0 {
                let factor = (speed / 100.0).min(1.0);
                Vec2 {
                    x: (vx / speed) * self.look_ahead * factor,
                    y: (vy / speed) * self.look_ahead * factor,
                }
            } else {
                Vec2::zero()
            }
        } else {
            Vec2::zero()
        };

        let desired = Vec2 {
            x: target.x + look.x,
            y: target.y + look.y,
        };

        let t = 1.0 - (-self.follow_speed * dt * 5.0).exp();
        camera.position = camera.position.lerp(desired, t);

        self.prev_target = Some(target);
    }
}

// ---------------------------------------------------------------------------
// CameraShake
// ---------------------------------------------------------------------------

/// A screen-shake effect that decays over time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraShake {
    pub intensity: f64,
    pub duration: f64,
    pub elapsed: f64,
    pub decay: f64,
}

impl CameraShake {
    pub fn new(intensity: f64, duration: f64) -> Self {
        Self {
            intensity,
            duration,
            elapsed: 0.0,
            decay: intensity / duration.max(1e-9),
        }
    }

    /// Advance the shake by `dt` and return the random offset to apply.
    pub fn update(&mut self, dt: f64) -> Vec2 {
        self.elapsed += dt;

        let current_intensity = (self.intensity - self.decay * self.elapsed).max(0.0);
        if self.elapsed >= self.duration || current_intensity <= 0.0 {
            return Vec2::zero();
        }

        let seed = (self.elapsed * 1000.0).floor();
        let ox = pseudo_noise(seed, 0) * 2.0 - 1.0;
        let oy = pseudo_noise(seed, 1) * 2.0 - 1.0;

        Vec2 {
            x: ox * current_intensity,
            y: oy * current_intensity,
        }
    }

    pub fn is_finished(&self) -> bool {
        self.elapsed >= self.duration || self.intensity <= 0.0
    }
}

/// Simple deterministic pseudo-random value in [0, 1).
fn pseudo_noise(seed: f64, channel: u64) -> f64 {
    let mut h = (seed as u64).wrapping_mul(0x9e3779b97f4a7c15);
    h = h.wrapping_add(channel);
    h ^= h >> 30;
    h = h.wrapping_mul(0xbf58476d1ce4e5b9);
    h ^= h >> 27;
    h = h.wrapping_mul(0x94d049bb133111eb);
    h ^= h >> 31;
    (h >> 11) as f64 / (1u64 << 53) as f64
}

// ---------------------------------------------------------------------------
// CameraState / Easing / CameraTransition
// ---------------------------------------------------------------------------

/// A snapshot of camera parameters.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CameraState {
    pub position: Vec2,
    pub zoom: f64,
    pub rotation: f64,
}

impl CameraState {
    pub fn from_camera(camera: &Camera) -> Self {
        Self {
            position: camera.position,
            zoom: camera.zoom,
            rotation: camera.rotation,
        }
    }

    pub fn apply(&self, camera: &mut Camera) {
        camera.position = self.position;
        camera.zoom = self.zoom;
        camera.rotation = self.rotation;
    }
}

/// Easing function for camera transitions.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Easing {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
}

impl Easing {
    /// Apply the easing curve to normalised time `t` in [0, 1].
    pub fn apply(&self, t: f64) -> f64 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Easing::Linear => t,
            Easing::EaseIn => t * t,
            Easing::EaseOut => 1.0 - (1.0 - t) * (1.0 - t),
            Easing::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(2) / 2.0
                }
            }
        }
    }
}

/// An animated transition between two camera states.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraTransition {
    pub from: CameraState,
    pub to: CameraState,
    pub duration: f64,
    pub elapsed: f64,
    pub easing: Easing,
}

impl CameraTransition {
    pub fn new(from: CameraState, to: CameraState, duration: f64, easing: Easing) -> Self {
        Self {
            from,
            to,
            duration,
            elapsed: 0.0,
            easing,
        }
    }

    /// Advance the transition by `dt` seconds and return the interpolated state.
    ///
    /// When the transition is complete the `to` state is returned unchanged.
    pub fn update(&mut self, dt: f64) -> CameraState {
        self.elapsed = (self.elapsed + dt).min(self.duration);
        let raw_t = if self.duration <= 0.0 { 1.0 } else { self.elapsed / self.duration };
        let t = self.easing.apply(raw_t);
        CameraState {
            position: self.from.position.lerp(self.to.position, t),
            zoom: self.from.zoom + (self.to.zoom - self.from.zoom) * t,
            rotation: self.from.rotation + (self.to.rotation - self.from.rotation) * t,
        }
    }

    pub fn is_complete(&self) -> bool {
        self.elapsed >= self.duration
    }

    /// Normalised progress in [0, 1].
    pub fn progress(&self) -> f64 {
        if self.duration <= 0.0 {
            return 1.0;
        }
        (self.elapsed / self.duration).clamp(0.0, 1.0)
    }
}

// ---------------------------------------------------------------------------
// CameraController
// ---------------------------------------------------------------------------

/// High-level controller that composes follow, shake, transitions and bounds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraController {
    pub camera: Camera,
    pub follow: CameraFollow,
    pub shake: Option<CameraShake>,
    pub transition: Option<CameraTransition>,
    pub bounds: Option<(Vec2, Vec2)>,
}

impl CameraController {
    pub fn new(viewport_w: f64, viewport_h: f64) -> Self {
        Self {
            camera: Camera::new(viewport_w, viewport_h),
            follow: CameraFollow::new(
                0.5,
                Rect::new(-10.0, -10.0, 20.0, 20.0),
                0.0,
            ),
            shake: None,
            transition: None,
            bounds: None,
        }
    }

    /// Start following `pos`.
    pub fn follow_target(&mut self, pos: Vec2) {
        self.follow.set_target(pos);
    }

    /// Trigger a screen shake.
    pub fn shake(&mut self, intensity: f64, duration: f64) {
        self.shake = Some(CameraShake::new(intensity, duration));
    }

    /// Begin a transition.
    pub fn transition_to(&mut self, state: CameraState, duration: f64, easing: Easing) {
        self.transition = Some(CameraTransition::new(
            CameraState::from_camera(&self.camera),
            state,
            duration,
            easing,
        ));
    }

    /// Set world-space bounds for the camera centre.
    pub fn constrain(&mut self, min: Vec2, max: Vec2) {
        self.bounds = Some((min, max));
    }

    /// Advance the controller by `dt` seconds.
    ///
    /// Order: follow -> transition -> shake -> bounds clamp.
    pub fn update(&mut self, dt: f64) {
        self.follow.update(&mut self.camera, dt);

        if let Some(ref mut tx) = self.transition {
            let state = tx.update(dt);
            state.apply(&mut self.camera);
            if tx.is_complete() {
                self.transition = None;
            }
        }

        if let Some(ref mut sh) = self.shake {
            let offset = sh.update(dt);
            self.camera.position = self.camera.position + offset;
            if sh.is_finished() {
                self.shake = None;
            }
        }

        if let Some((min, max)) = self.bounds {
            self.camera.position.x = self.camera.position.x.clamp(min.x, max.x);
            self.camera.position.y = self.camera.position.y.clamp(min.y, max.y);
        }
    }

    pub fn camera(&self) -> &Camera {
        &self.camera
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ---- Vec2 ----

    #[test]
    fn vec2_new() {
        let v = Vec2::new(3.0, 4.0);
        assert_eq!(v.x, 3.0);
        assert_eq!(v.y, 4.0);
    }

    #[test]
    fn vec2_zero() {
        let v = Vec2::zero();
        assert_eq!(v.x, 0.0);
        assert_eq!(v.y, 0.0);
    }

    #[test]
    fn vec2_distance() {
        let a = Vec2::new(0.0, 0.0);
        let b = Vec2::new(3.0, 4.0);
        assert!((a.distance(b) - 5.0).abs() < 1e-12);
    }

    #[test]
    fn vec2_lerp() {
        let a = Vec2::new(0.0, 0.0);
        let b = Vec2::new(10.0, 20.0);
        let mid = a.lerp(b, 0.5);
        assert!((mid.x - 5.0).abs() < 1e-12);
        assert!((mid.y - 10.0).abs() < 1e-12);
    }

    #[test]
    fn vec2_lerp_clamped() {
        let a = Vec2::new(0.0, 0.0);
        let b = Vec2::new(10.0, 10.0);
        assert_eq!(a.lerp(b, 1.5), b);
        assert_eq!(a.lerp(b, -0.5), a);
    }

    #[test]
    fn vec2_add() {
        assert_eq!(
            Vec2::new(1.0, 2.0) + Vec2::new(3.0, 4.0),
            Vec2::new(4.0, 6.0)
        );
    }

    #[test]
    fn vec2_sub() {
        assert_eq!(
            Vec2::new(5.0, 7.0) - Vec2::new(2.0, 3.0),
            Vec2::new(3.0, 4.0)
        );
    }

    #[test]
    fn vec2_mul() {
        assert_eq!(Vec2::new(2.0, 3.0) * 2.0, Vec2::new(4.0, 6.0));
    }

    // ---- Rect ----

    #[test]
    fn rect_contains() {
        let r = Rect::new(0.0, 0.0, 100.0, 100.0);
        assert!(r.contains(Vec2::new(50.0, 50.0)));
        assert!(r.contains(Vec2::new(0.0, 0.0)));
        assert!(r.contains(Vec2::new(100.0, 100.0)));
        assert!(!r.contains(Vec2::new(-1.0, 50.0)));
        assert!(!r.contains(Vec2::new(50.0, 101.0)));
    }

    #[test]
    fn rect_center() {
        let r = Rect::new(0.0, 0.0, 100.0, 200.0);
        assert_eq!(r.center(), Vec2::new(50.0, 100.0));
    }

    // ---- Camera ----

    #[test]
    fn camera_new_center() {
        let cam = Camera::new(800.0, 600.0);
        assert_eq!(cam.position, Vec2::zero());
        assert!((cam.zoom - 1.0).abs() < 1e-9);
        assert_eq!(cam.viewport_width, 800.0);
        assert_eq!(cam.viewport_height, 600.0);
    }

    #[test]
    fn camera_world_to_screen_identity() {
        let cam = Camera::new(800.0, 600.0);
        let screen = cam.world_to_screen(Vec2::zero());
        assert!((screen.x - 400.0).abs() < 1e-9);
        assert!((screen.y - 300.0).abs() < 1e-9);
    }

    #[test]
    fn camera_roundtrip() {
        let cam = Camera::new(800.0, 600.0);
        let world = Vec2::new(123.456, -789.012);
        let screen = cam.world_to_screen(world);
        let back = cam.screen_to_world(screen);
        assert!((back.x - world.x).abs() < 1e-9);
        assert!((back.y - world.y).abs() < 1e-9);
    }

    #[test]
    fn camera_roundtrip_nonzero_zoom_and_rotation() {
        let mut cam = Camera::new(1024.0, 768.0);
        cam.set_zoom(2.0);
        cam.set_rotation(0.5);
        let world = Vec2::new(100.0, -200.0);
        let screen = cam.world_to_screen(world);
        let back = cam.screen_to_world(screen);
        assert!((back.x - world.x).abs() < 1e-9);
        assert!((back.y - world.y).abs() < 1e-9);
    }

    #[test]
    fn camera_visible_bounds() {
        let cam = Camera::new(800.0, 600.0);
        let (tl, br) = cam.visible_bounds();
        assert!((tl.x - (-400.0)).abs() < 1e-9);
        assert!((tl.y - (-300.0)).abs() < 1e-9);
        assert!((br.x - 400.0).abs() < 1e-9);
        assert!((br.y - 300.0).abs() < 1e-9);
    }

    #[test]
    fn camera_is_visible() {
        let cam = Camera::new(800.0, 600.0);
        assert!(cam.is_visible(Vec2::new(0.0, 0.0), 0.0));
        assert!(!cam.is_visible(Vec2::new(500.0, 0.0), 0.0));
        assert!(cam.is_visible(Vec2::new(500.0, 0.0), 200.0));
    }

    #[test]
    fn camera_center_on() {
        let mut cam = Camera::new(800.0, 600.0);
        cam.center_on(Vec2::new(100.0, 200.0));
        assert_eq!(cam.position, Vec2::new(100.0, 200.0));
    }

    #[test]
    fn camera_setters() {
        let mut cam = Camera::new(800.0, 600.0);
        cam.set_position(Vec2::new(10.0, 20.0));
        cam.set_zoom(3.0);
        cam.set_rotation(1.5);
        assert_eq!(cam.position, Vec2::new(10.0, 20.0));
        assert!((cam.zoom - 3.0).abs() < 1e-9);
        assert!((cam.rotation - 1.5).abs() < 1e-9);
    }

    // ---- CameraFollow ----

    #[test]
    fn camera_follow_does_nothing_without_target() {
        let mut cam = Camera::new(800.0, 600.0);
        let mut follow = CameraFollow::new(0.5, Rect::new(-5.0, -5.0, 10.0, 10.0), 0.0);
        follow.update(&mut cam, 1.0);
        assert_eq!(cam.position, Vec2::zero());
    }

    #[test]
    fn camera_follow_moves_toward_target() {
        let mut cam = Camera::new(800.0, 600.0);
        let mut follow =
            CameraFollow::new(0.9, Rect::new(-1.0, -1.0, 2.0, 2.0), 0.0);
        follow.set_target(Vec2::new(100.0, 0.0));
        follow.update(&mut cam, 1.0);
        assert!(cam.position.x > 0.0);
        assert!(cam.position.x < 100.0);
    }

    #[test]
    fn camera_follow_deadzone() {
        let mut cam = Camera::new(800.0, 600.0);
        let mut follow =
            CameraFollow::new(0.9, Rect::new(-100.0, -100.0, 200.0, 200.0), 0.0);
        follow.set_target(Vec2::new(50.0, 0.0));
        follow.update(&mut cam, 1.0);
        assert!((cam.position.x).abs() < 1e-9);
    }

    #[test]
    fn camera_follow_clear_target() {
        let mut cam = Camera::new(800.0, 600.0);
        let mut follow = CameraFollow::new(0.5, Rect::new(-1.0, -1.0, 2.0, 2.0), 0.0);
        follow.set_target(Vec2::new(100.0, 0.0));
        follow.clear_target();
        follow.update(&mut cam, 1.0);
        assert_eq!(cam.position, Vec2::zero());
    }

    // ---- CameraShake ----

    #[test]
    fn camera_shake_initial() {
        let sh = CameraShake::new(10.0, 1.0);
        assert!(!sh.is_finished());
        assert!((sh.intensity - 10.0).abs() < 1e-9);
    }

    #[test]
    fn camera_shake_decays_to_zero() {
        let mut sh = CameraShake::new(10.0, 1.0);
        for _ in 0..100 {
            sh.update(0.01);
        }
        assert!(sh.is_finished());
    }

    #[test]
    fn camera_shake_zero_intensity() {
        let mut sh = CameraShake::new(0.0, 1.0);
        let offset = sh.update(0.5);
        assert_eq!(offset, Vec2::zero());
        assert!(sh.is_finished());
    }

    // ---- Easing ----

    #[test]
    fn easing_linear() {
        assert!((Easing::Linear.apply(0.5) - 0.5).abs() < 1e-9);
        assert!((Easing::Linear.apply(0.0) - 0.0).abs() < 1e-9);
        assert!((Easing::Linear.apply(1.0) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn easing_ease_in_out_symmetry() {
        let half = Easing::EaseInOut.apply(0.5);
        assert!((half - 0.5).abs() < 1e-9, "at t=0.5, ease-in-out should be 0.5");
        let a = Easing::EaseInOut.apply(0.25);
        let b = Easing::EaseInOut.apply(0.75);
        assert!((a + b - 1.0).abs() < 1e-9, "symmetry: F(0.25) + F(0.75) = 1");
    }

    // ---- CameraTransition ----

    #[test]
    fn camera_transition_linear() {
        let from = CameraState {
            position: Vec2::zero(),
            zoom: 1.0,
            rotation: 0.0,
        };
        let to = CameraState {
            position: Vec2::new(100.0, 0.0),
            zoom: 2.0,
            rotation: std::f64::consts::FRAC_PI_2,
        };
        let mut tx = CameraTransition::new(from, to, 2.0, Easing::Linear);
        let mid = tx.update(1.0);
        assert!((mid.position.x - 50.0).abs() < 1e-9);
        assert!((mid.zoom - 1.5).abs() < 1e-9);
        assert!((mid.rotation - std::f64::consts::FRAC_PI_4).abs() < 1e-9);
        assert!(!tx.is_complete());
        assert!((tx.progress() - 0.5).abs() < 1e-9);
    }

    #[test]
    fn camera_transition_completes() {
        let from = CameraState {
            position: Vec2::zero(),
            zoom: 1.0,
            rotation: 0.0,
        };
        let to = CameraState {
            position: Vec2::new(100.0, 0.0),
            zoom: 2.0,
            rotation: 0.0,
        };
        let mut tx = CameraTransition::new(from, to, 1.0, Easing::Linear);
        let final_state = tx.update(2.0);
        assert!(tx.is_complete());
        assert!((tx.progress() - 1.0).abs() < 1e-9);
        assert!((final_state.position.x - 100.0).abs() < 1e-9);
    }

    #[test]
    fn camera_transition_zero_duration() {
        let from = CameraState {
            position: Vec2::zero(),
            zoom: 1.0,
            rotation: 0.0,
        };
        let to = CameraState {
            position: Vec2::new(50.0, 50.0),
            zoom: 3.0,
            rotation: 1.0,
        };
        let mut tx = CameraTransition::new(from, to, 0.0, Easing::Linear);
        let state = tx.update(0.0);
        assert!(tx.is_complete());
        assert!((state.position.x - 50.0).abs() < 1e-9);
    }

    // ---- CameraController ----

    #[test]
    fn controller_new() {
        let ctrl = CameraController::new(800.0, 600.0);
        assert_eq!(ctrl.camera.viewport_width, 800.0);
        assert_eq!(ctrl.camera.viewport_height, 600.0);
        assert!(ctrl.shake.is_none());
        assert!(ctrl.transition.is_none());
        assert!(ctrl.bounds.is_none());
    }

    #[test]
    fn controller_follow_and_transition() {
        let mut ctrl = CameraController::new(800.0, 600.0);
        ctrl.follow_target(Vec2::new(200.0, 0.0));
        ctrl.update(1.0);
        assert!(ctrl.camera.position.x > 0.0);
    }

    #[test]
    fn controller_shake() {
        let mut ctrl = CameraController::new(800.0, 600.0);
        ctrl.shake(5.0, 2.0);
        assert!(ctrl.shake.is_some());
        ctrl.update(0.5);
    }

    #[test]
    fn controller_bounds() {
        let mut ctrl = CameraController::new(800.0, 600.0);
        ctrl.constrain(Vec2::new(-100.0, -100.0), Vec2::new(100.0, 100.0));
        ctrl.follow_target(Vec2::new(9999.0, 9999.0));
        ctrl.update(1.0);
        assert!(ctrl.camera.position.x <= 100.0);
        assert!(ctrl.camera.position.y <= 100.0);
        assert!(ctrl.camera.position.x >= -100.0);
        assert!(ctrl.camera.position.y >= -100.0);
    }

    #[test]
    fn controller_transition_to() {
        let mut ctrl = CameraController::new(800.0, 600.0);
        let target = CameraState {
            position: Vec2::new(300.0, 200.0),
            zoom: 2.5,
            rotation: 0.3,
        };
        ctrl.transition_to(target, 2.0, Easing::EaseInOut);
        assert!(ctrl.transition.is_some());

        ctrl.update(1.0);
        assert!(ctrl.camera.position.x > 0.0);
        assert!(ctrl.camera.position.x < 300.0);

        ctrl.update(2.0);
        assert!(ctrl.transition.is_none());
    }

    #[test]
    fn controller_camera_accessor() {
        let ctrl = CameraController::new(1024.0, 768.0);
        let cam = ctrl.camera();
        assert_eq!(cam.viewport_width, 1024.0);
        assert_eq!(cam.viewport_height, 768.0);
    }

    // ---- Serde roundtrip ----

    #[test]
    fn serde_camera_roundtrip() {
        let cam = Camera::new(800.0, 600.0);
        let json = serde_json::to_string(&cam).unwrap();
        let back: Camera = serde_json::from_str(&json).unwrap();
        assert!((back.zoom - 1.0).abs() < 1e-9);
        assert_eq!(back.viewport_width, 800.0);
        assert_eq!(back.viewport_height, 600.0);
    }

    #[test]
    fn serde_camera_state_roundtrip() {
        let state = CameraState {
            position: Vec2::new(100.0, 200.0),
            zoom: 2.5,
            rotation: 0.3,
        };
        let json = serde_json::to_string(&state).unwrap();
        let back: CameraState = serde_json::from_str(&json).unwrap();
        assert_eq!(back.position.x, 100.0);
        assert_eq!(back.position.y, 200.0);
        assert!((back.zoom - 2.5).abs() < 1e-9);
        assert!((back.rotation - 0.3).abs() < 1e-9);
    }

    #[test]
    fn serde_camera_shake_roundtrip() {
        let sh = CameraShake::new(5.0, 2.0);
        let json = serde_json::to_string(&sh).unwrap();
        let back: CameraShake = serde_json::from_str(&json).unwrap();
        assert!((back.intensity - 5.0).abs() < 1e-9);
        assert!((back.duration - 2.0).abs() < 1e-9);
        assert!((back.elapsed - 0.0).abs() < 1e-9);
    }

    #[test]
    fn serde_vec2_roundtrip() {
        let v = Vec2::new(123.0, 456.0);
        let json = serde_json::to_string(&v).unwrap();
        let back: Vec2 = serde_json::from_str(&json).unwrap();
        assert!((back.x - 123.0).abs() < 1e-12);
        assert!((back.y - 456.0).abs() < 1e-12);
    }
}

