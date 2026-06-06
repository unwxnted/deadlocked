use std::time::{Duration, Instant};

use glam::Vec2;

use crate::{
    config::{Config, aim::KeyMode, movement::AutostrafeConfig},
    cs2::{CS2, entity::player::Player, input::Input, key_codes::KeyCode},
    os::uinput::InputDevice,
};

#[derive(Debug, Default)]
struct ActivationState {
    active: bool,
}

impl ActivationState {
    fn sync(&mut self, input: &Input, hotkey: KeyCode, mode: KeyMode, ignore_toggle: bool) -> bool {
        if hotkey == KeyCode::None {
            self.active = false;
            return false;
        }

        match mode {
            KeyMode::Hold => {
                self.active = input.is_key_pressed(hotkey);
            }
            KeyMode::Toggle => {
                if !ignore_toggle && input.key_just_pressed(hotkey) {
                    self.active = !self.active;
                }
            }
        }

        self.active
    }

    fn set(&mut self, active: bool) -> bool {
        self.active = active;
        active
    }

    fn clear(&mut self) {
        self.active = false;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StrafeSide {
    Left,
    Right,
}

#[derive(Debug, Default)]
struct BunnyhopState {
    jump_pressed: bool,
    jump_pressed_at: Option<Instant>,
    last_attempt: Option<Instant>,
    attempted_this_ground_contact: bool,
    last_on_ground: bool,
}

impl BunnyhopState {
    const JUMP_PULSE: Duration = Duration::from_millis(20);
    const GROUND_RETRY_COOLDOWN: Duration = Duration::from_millis(50);

    fn release_if_needed(&mut self, input_device: &mut InputDevice, now: Instant) {
        if self.jump_pressed
            && let Some(jump_pressed_at) = self.jump_pressed_at
            && now.duration_since(jump_pressed_at) >= Self::JUMP_PULSE
        {
            input_device.space_release();
            self.jump_pressed = false;
            self.jump_pressed_at = None;
        }
    }

    fn reset(&mut self, input_device: &mut InputDevice) {
        if self.jump_pressed {
            input_device.space_release();
        }
        self.jump_pressed = false;
        self.jump_pressed_at = None;
        self.last_attempt = None;
        self.attempted_this_ground_contact = false;
        self.last_on_ground = false;
    }

    fn press(&mut self, input_device: &mut InputDevice, now: Instant) {
        input_device.space_press();
        self.jump_pressed = true;
        self.jump_pressed_at = Some(now);
        self.last_attempt = Some(now);
        self.attempted_this_ground_contact = true;
    }

    fn mark_ground_state(&mut self, on_ground: bool) {
        if self.last_on_ground && !on_ground {
            self.attempted_this_ground_contact = false;
        }
        self.last_on_ground = on_ground;
    }
}

#[derive(Debug, Default)]
struct AutostrafeState {
    current_side: Option<StrafeSide>,
    filtered_intent: f32,
    previous_yaw: Option<f32>,
}

impl AutostrafeState {
    const MIN_SPEED: f32 = 1.0;
    const MOUSE_DEADZONE: f32 = 0.15;
    const VELOCITY_DEADZONE: f32 = 0.75;

    fn reset(&mut self, input_device: &mut InputDevice) {
        self.release(input_device);
        self.filtered_intent = 0.0;
        self.previous_yaw = None;
    }

    fn release(&mut self, input_device: &mut InputDevice) {
        match self.current_side.take() {
            Some(StrafeSide::Left) => input_device.a_release(),
            Some(StrafeSide::Right) => input_device.d_release(),
            None => {}
        }
    }

    fn sync(
        &mut self,
        input_device: &mut InputDevice,
        velocity: Vec2,
        view_yaw: f32,
        config: &AutostrafeConfig,
    ) {
        let speed = velocity.length();
        let yaw_delta = self
            .previous_yaw
            .map(|previous| normalize_yaw_delta(view_yaw - previous))
            .unwrap_or(0.0);
        self.previous_yaw = Some(view_yaw);

        if speed < Self::MIN_SPEED {
            self.release(input_device);
            self.filtered_intent = 0.0;
            return;
        }

        let velocity_yaw = velocity.y.atan2(velocity.x).to_degrees();
        let velocity_delta = normalize_yaw_delta(view_yaw - velocity_yaw);

        let desired_intent = intent_from_delta(yaw_delta, Self::MOUSE_DEADZONE)
            .or_else(|| intent_from_delta(velocity_delta, Self::VELOCITY_DEADZONE))
            .unwrap_or_else(|| {
                self.current_side
                    .map(StrafeSide::intent)
                    .unwrap_or(velocity_delta.signum())
                    .clamp(-1.0, 1.0)
            });

        let smooth = config.smooth.clamp(0.0, 1.0);
        let response = (1.0 - smooth).clamp(0.08, 1.0);
        self.filtered_intent += (desired_intent - self.filtered_intent) * response;

        let switch_threshold = 0.0001 + smooth * 0.25;
        let target_side = if self.filtered_intent > switch_threshold {
            Some(StrafeSide::Left)
        } else if self.filtered_intent < -switch_threshold {
            Some(StrafeSide::Right)
        } else {
            self.current_side.or({
                if desired_intent >= 0.0 {
                    Some(StrafeSide::Left)
                } else {
                    Some(StrafeSide::Right)
                }
            })
        };

        if let Some(target_side) = target_side {
            self.set_side(target_side, input_device);
        }
    }

    fn set_side(&mut self, side: StrafeSide, input_device: &mut InputDevice) {
        if self.current_side == Some(side) {
            return;
        }

        self.release(input_device);

        match side {
            StrafeSide::Left => input_device.a_press(),
            StrafeSide::Right => input_device.d_press(),
        }

        self.current_side = Some(side);
    }
}

impl StrafeSide {
    fn intent(self) -> f32 {
        match self {
            Self::Left => 1.0,
            Self::Right => -1.0,
        }
    }
}

#[derive(Debug, Default)]
pub struct MovementState {
    master_activation: ActivationState,
    bhop_activation: ActivationState,
    autostrafe_activation: ActivationState,
    bhop: BunnyhopState,
    autostrafe: AutostrafeState,
    synthetic_space_guard_until: Option<Instant>,
}

impl MovementState {
    const SYNTHETIC_SPACE_GUARD: Duration = Duration::from_millis(40);

    fn guard_space_toggle(&mut self, now: Instant) {
        self.synthetic_space_guard_until = Some(now + Self::SYNTHETIC_SPACE_GUARD);
    }

    fn space_toggle_guard_active(&self, now: Instant) -> bool {
        self.synthetic_space_guard_until
            .is_some_and(|until| now < until)
    }

    fn reset_runtime(&mut self, input_device: &mut InputDevice) {
        self.bhop.reset(input_device);
        self.autostrafe.reset(input_device);
    }

    fn sync_master_activation(&mut self, input: &Input, config: &Config, now: Instant) -> bool {
        if !config.movement.master_enabled {
            self.master_activation.clear();
            return false;
        }

        self.master_activation.sync(
            input,
            config.movement.master_hotkey,
            config.movement.master_mode,
            config.movement.master_hotkey == KeyCode::Space
                && config.movement.master_mode == KeyMode::Toggle
                && self.space_toggle_guard_active(now),
        )
    }

    fn sync_bhop_activation(
        &mut self,
        input: &Input,
        config: &Config,
        master_active: bool,
        now: Instant,
    ) -> bool {
        let bhop = &config.movement.bhop;
        if !bhop.enabled {
            self.bhop_activation.clear();
            return false;
        }

        if bhop.use_master_activation {
            self.bhop_activation.set(master_active)
        } else {
            self.bhop_activation.sync(
                input,
                bhop.hotkey,
                bhop.mode,
                bhop.hotkey == KeyCode::Space
                    && bhop.mode == KeyMode::Toggle
                    && self.space_toggle_guard_active(now),
            )
        }
    }

    fn sync_autostrafe_activation(
        &mut self,
        input: &Input,
        config: &Config,
        master_active: bool,
        now: Instant,
    ) -> bool {
        let autostrafe = &config.movement.autostrafe;
        if !autostrafe.enabled {
            self.autostrafe_activation.clear();
            return false;
        }

        if autostrafe.use_master_activation {
            self.autostrafe_activation.set(master_active)
        } else {
            self.autostrafe_activation.sync(
                input,
                autostrafe.hotkey,
                autostrafe.mode,
                autostrafe.hotkey == KeyCode::Space
                    && autostrafe.mode == KeyMode::Toggle
                    && self.space_toggle_guard_active(now),
            )
        }
    }

    pub fn master_active(&self) -> bool {
        self.master_activation.active
    }

    pub fn bhop_active(&self) -> bool {
        self.bhop_activation.active
    }

    pub fn autostrafe_active(&self) -> bool {
        self.autostrafe_activation.active
    }
}

impl CS2 {
    pub fn movement(&mut self, config: &Config, input_device: &mut InputDevice) {
        let now = Instant::now();
        self.movement_state
            .bhop
            .release_if_needed(input_device, now);

        let master_active = self
            .movement_state
            .sync_master_activation(&self.input, config, now);
        let bhop_active =
            self.movement_state
                .sync_bhop_activation(&self.input, config, master_active, now);
        let autostrafe_active =
            self.movement_state
                .sync_autostrafe_activation(&self.input, config, master_active, now);

        let Some(local_player) = Player::local_player(self) else {
            self.movement_state.reset_runtime(input_device);
            return;
        };

        let on_ground = local_player.is_on_ground(self);
        self.movement_state.bhop.mark_ground_state(on_ground);

        if bhop_active {
            if on_ground && !self.movement_state.bhop.jump_pressed {
                let can_retry = self
                    .movement_state
                    .bhop
                    .last_attempt
                    .is_some_and(|last_attempt| {
                        now.duration_since(last_attempt) >= BunnyhopState::GROUND_RETRY_COOLDOWN
                    });

                if !self.movement_state.bhop.attempted_this_ground_contact || can_retry {
                    self.movement_state.bhop.press(input_device, now);
                    self.movement_state.guard_space_toggle(now);
                }
            }
        } else {
            self.movement_state.bhop.reset(input_device);
        }

        if autostrafe_active && !on_ground {
            let velocity = local_player.velocity(self);
            self.movement_state.autostrafe.sync(
                input_device,
                Vec2::new(velocity.x, velocity.y),
                local_player.view_angles(self).y,
                &config.movement.autostrafe,
            );
        } else {
            self.movement_state.autostrafe.reset(input_device);
        }
    }

    pub fn reset_movement(&mut self, input_device: &mut InputDevice) {
        self.movement_state.reset_runtime(input_device);
    }

    pub fn movement_master_active(&self, config: &Config) -> bool {
        config.movement.master_enabled && self.movement_state.master_active()
    }

    pub fn bhop_enabled(&self, config: &Config) -> bool {
        config.movement.bhop.enabled && self.movement_state.bhop_active()
    }

    pub fn autostrafe_enabled(&self, config: &Config) -> bool {
        config.movement.autostrafe.enabled && self.movement_state.autostrafe_active()
    }
}

fn intent_from_delta(delta: f32, deadzone: f32) -> Option<f32> {
    if delta.abs() <= deadzone {
        None
    } else {
        Some(delta.signum())
    }
}

fn normalize_yaw_delta(delta: f32) -> f32 {
    (delta + 180.0).rem_euclid(360.0) - 180.0
}
