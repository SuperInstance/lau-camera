# lau-camera

> 2D camera system for games — viewport, zoom, follow, shake, smooth transitions

## What This Does

2D camera system for games — viewport, zoom, follow, shake, smooth transitions. Part of the PLATO/LAU ecosystem — a mathematically rigorous framework for building educational agents that learn, teach, and evolve.

## The Key Idea

This crate implements the core abstractions needed for its domain, with a focus on correctness, composability, and conservation guarantees. Every public type is serializable (serde), every algorithm is tested, and every invariant is verified.

## Install

```bash
cargo add lau-camera
```

## Quick Start

See the API Reference below for complete usage. Key entry points:

```rust
use lau_camera::*;
// See types and methods below for complete usage
```

## API Reference

```rust
pub struct Vec2 
    pub fn distance(self, other: Self) -> f64 
    pub fn lerp(self, target: Self, t: f64) -> Self 
pub struct Rect 
    pub fn contains(&self, point: Vec2) -> bool 
    pub fn center(&self) -> Vec2 
pub struct Camera 
    pub fn new(viewport_width: f64, viewport_height: f64) -> Self 
    pub fn world_to_screen(&self, world_pos: Vec2) -> Vec2 
    pub fn screen_to_world(&self, screen_pos: Vec2) -> Vec2 
    pub fn visible_bounds(&self) -> (Vec2, Vec2) 
    pub fn is_visible(&self, pos: Vec2, margin: f64) -> bool 
    pub fn set_position(&mut self, pos: Vec2) 
    pub fn set_zoom(&mut self, zoom: f64) 
    pub fn set_rotation(&mut self, rot: f64) 
    pub fn center_on(&mut self, target: Vec2) 
pub struct CameraFollow 
    pub fn new(follow_speed: f64, deadzone: Rect, look_ahead: f64) -> Self 
    pub fn set_target(&mut self, pos: Vec2) 
    pub fn clear_target(&mut self) 
    pub fn update(&mut self, camera: &mut Camera, dt: f64) 
pub struct CameraShake 
    pub fn new(intensity: f64, duration: f64) -> Self 
    pub fn update(&mut self, dt: f64) -> Vec2 
    pub fn is_finished(&self) -> bool 
pub struct CameraState 
    pub fn from_camera(camera: &Camera) -> Self 
    pub fn apply(&self, camera: &mut Camera) 
pub enum Easing 
    pub fn apply(&self, t: f64) -> f64 
pub struct CameraTransition 
    pub fn new(from: CameraState, to: CameraState, duration: f64, easing: Easing) -> Self 
    pub fn update(&mut self, dt: f64) -> CameraState 
    pub fn is_complete(&self) -> bool 
    pub fn progress(&self) -> f64 
pub struct CameraController 
    pub fn new(viewport_w: f64, viewport_h: f64) -> Self 
    pub fn follow_target(&mut self, pos: Vec2) 
    pub fn shake(&mut self, intensity: f64, duration: f64) 
    pub fn transition_to(&mut self, state: CameraState, duration: f64, easing: Easing) 
    pub fn constrain(&mut self, min: Vec2, max: Vec2) 
    pub fn update(&mut self, dt: f64) 
    pub fn camera(&self) -> &Camera 
```

## How It Works

Read the source in `src/` for full implementation details. All algorithms are documented with inline comments explaining the mathematical foundations.

## The Math

This crate implements formal mathematical constructs. See the source documentation for theorem statements and proofs of correctness.

## Testing

**40 tests** covering construction, serialization, correctness properties, edge cases, and composability with other lau-* crates.

## License

MIT
