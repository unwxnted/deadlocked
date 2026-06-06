use std::time::{Duration, Instant};

use crate::{
    config::Config,
    config::aim::KeyMode,
    cs2::{CS2, entity::player::Player, input::Input, key_codes::KeyCode},
    os::uinput::InputDevice,
};

#[derive(Debug, Default)]
pub struct Bunnyhop {
    active: bool,
    jump_pressed: bool,
    jump_pressed_at: Option<Instant>,
    last_attempt: Option<Instant>,
    attempted_this_ground_contact: bool,
    ignore_toggle_until: Option<Instant>,
    last_on_ground: bool,
}

impl Bunnyhop {
    const JUMP_PULSE: Duration = Duration::from_millis(18);
    const GROUND_RETRY_COOLDOWN: Duration = Duration::from_millis(32);
    const SYNTHETIC_TOGGLE_GUARD: Duration = Duration::from_millis(40);

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

    fn release(&mut self, input_device: &mut InputDevice) {
        if self.jump_pressed {
            input_device.space_release();
        }
        self.jump_pressed = false;
        self.jump_pressed_at = None;
    }

    fn disarm(&mut self, input_device: &mut InputDevice) {
        self.active = false;
        self.release(input_device);
    }

    fn press(&mut self, input_device: &mut InputDevice, now: Instant) {
        input_device.space_press();
        self.jump_pressed = true;
        self.jump_pressed_at = Some(now);
        self.last_attempt = Some(now);
        self.attempted_this_ground_contact = true;
        self.ignore_toggle_until = Some(now + Self::SYNTHETIC_TOGGLE_GUARD);
    }

    fn mark_ground_state(&mut self, on_ground: bool) {
        if self.last_on_ground && !on_ground {
            self.attempted_this_ground_contact = false;
        }
        self.last_on_ground = on_ground;
    }

    fn should_ignore_toggle(&self, now: Instant) -> bool {
        self.ignore_toggle_until.is_some_and(|until| now < until)
    }

    fn sync_hotkey_state(
        &mut self,
        input: &Input,
        hotkey: KeyCode,
        mode: KeyMode,
        now: Instant,
    ) -> bool {
        if hotkey == KeyCode::None {
            self.active = false;
            return false;
        }

        match mode {
            KeyMode::Hold => {
                self.active = input.is_key_pressed(hotkey);
            }
            KeyMode::Toggle => {
                if input.key_just_pressed(hotkey)
                    && !(hotkey == KeyCode::Space && self.should_ignore_toggle(now))
                {
                    self.active = !self.active;
                }
            }
        }

        self.active
    }
}

impl CS2 {
    pub fn bhop(&mut self, config: &Config, input_device: &mut InputDevice) {
        let now = Instant::now();
        self.bhop.release_if_needed(input_device, now);

        let Some(local_player) = Player::local_player(self) else {
            self.bhop.disarm(input_device);
            self.bhop.mark_ground_state(false);
            return;
        };

        let on_ground = local_player.is_on_ground(self);
        self.bhop.mark_ground_state(on_ground);

        if !config.misc.bhop {
            self.bhop.disarm(input_device);
            return;
        }

        if !self.bhop.sync_hotkey_state(
            &self.input,
            config.misc.bhop_hotkey,
            config.misc.bhop_mode,
            now,
        ) {
            self.bhop.release(input_device);
            return;
        }

        if !on_ground || self.bhop.jump_pressed {
            return;
        }

        let can_retry = self.bhop.last_attempt.is_some_and(|last_attempt| {
            now.duration_since(last_attempt) >= Bunnyhop::GROUND_RETRY_COOLDOWN
        });

        if self.bhop.attempted_this_ground_contact && !can_retry {
            return;
        }

        self.bhop.press(input_device, now);
    }

    pub fn bhop_enabled(&self, config: &Config) -> bool {
        config.misc.bhop && self.bhop.active
    }
}
