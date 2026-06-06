use std::time::{Duration, Instant};

use glam::{Vec2, vec2};
use rand::{RngExt, rng};

use crate::{
    config::{Config, aim::AimbotConfig},
    cs2::{
        CS2,
        entity::{player::Player, weapon_class::WeaponClass},
    },
    math::{angles_to_fov, vec2_clamp},
    os::uinput::InputDevice,
};

#[derive(Debug, Default)]
pub struct Aimbot {
    pub active: bool,
    inertia: Vec2,
    jitter_offset: Vec2,
    jitter_target: Vec2,
    last_jitter_update: Option<Instant>,
}

impl CS2 {
    pub fn aimbot(&mut self, config: &Config, input_device: &mut InputDevice) -> bool {
        let hotkey = config.aim.aimbot_hotkey;
        let config = self.aimbot_config(config);

        if !config.enabled {
            self.aim.reset_jitter();
            return false;
        }

        if !Self::check_hotkey(&self.input, config.mode, hotkey, &mut self.aim.active) {
            self.aim.reset_jitter();
            return false;
        }

        let Some(target) = &self.target.player else {
            self.aim.reset_jitter();
            return false;
        };

        if !target.is_valid(self) {
            self.aim.reset_jitter();
            return false;
        }

        let Some(local_player) = Player::local_player(self) else {
            return false;
        };

        let weapon_class = local_player.weapon_class(self);
        let disallowed_weapons = [
            WeaponClass::Unknown,
            WeaponClass::Knife,
            WeaponClass::Grenade,
        ];
        if disallowed_weapons.contains(&weapon_class) {
            self.aim.reset_jitter();
            return false;
        }

        if config.flash_check && local_player.is_flashed(self) {
            self.aim.reset_jitter();
            return false;
        }

        if config.visibility_check && !target.visible(self, &local_player) {
            self.aim.reset_jitter();
            return false;
        }

        if local_player.shots_fired(self) < config.start_bullet {
            self.aim.reset_jitter();
            return false;
        }

        let target_angle = {
            let mut smallest_fov = 360.0;
            let mut smallest_angle = glam::Vec2::ZERO;
            for bone in &config.bones {
                let bone_pos = target.bone_position(self, bone.u64());
                let angle =
                    self.angle_to_target(&local_player, &bone_pos, &self.target.previous_aim_punch);
                let fov = angles_to_fov(&local_player.view_angles(self), &angle);
                if fov < smallest_fov {
                    smallest_fov = fov;
                    smallest_angle = angle;
                }
            }

            smallest_angle
        };

        let view_angles = local_player.view_angles(self);
        if angles_to_fov(&view_angles, &target_angle)
            > (config.fov
                * if config.distance_adjusted_fov {
                    self.distance_scale(self.target.distance)
                } else {
                    1.0
                })
        {
            self.aim.reset_jitter();
            return false;
        }

        let mut jittered_target_angle = target_angle + self.aim.jitter(config);
        vec2_clamp(&mut jittered_target_angle);

        let mut aim_angles = view_angles - jittered_target_angle;
        if aim_angles.y < -180.0 {
            aim_angles.y += 360.0
        }
        vec2_clamp(&mut aim_angles);

        let sensitivity = self.get_sensitivity() * local_player.fov_multiplier(self);

        let mouse_angles = vec2(
            aim_angles.y / sensitivity * 45.45,
            -aim_angles.x / sensitivity * 45.45,
        ) / (config.smooth + 1.0).clamp(1.0, 20.0);

        let alpha = 1.0 - config.inertia.clamp(0.0, 1.0) * 0.5;
        self.aim.inertia += (mouse_angles - self.aim.inertia) * alpha;
        input_device.move_rel(self.aim.inertia);

        self.recoil.previous = local_player.aim_punch(self);

        true
    }
}

impl Aimbot {
    const JITTER_UPDATE_INTERVAL: Duration = Duration::from_millis(40);
    const JITTER_RESPONSE: f32 = 0.28;

    fn reset_jitter(&mut self) {
        self.jitter_offset = Vec2::ZERO;
        self.jitter_target = Vec2::ZERO;
        self.last_jitter_update = None;
    }

    fn jitter(&mut self, config: &AimbotConfig) -> Vec2 {
        if !config.jitter.enabled {
            self.reset_jitter();
            return Vec2::ZERO;
        }

        let amount = config.jitter.amount.max(0.0);
        if amount <= f32::EPSILON {
            self.reset_jitter();
            return Vec2::ZERO;
        }

        let now = Instant::now();
        if self
            .last_jitter_update
            .is_none_or(|last| now.duration_since(last) >= Self::JITTER_UPDATE_INTERVAL)
        {
            self.jitter_target = vec2(
                rng().random_range(-amount..=amount),
                rng().random_range(-amount..=amount),
            );
            self.last_jitter_update = Some(now);
        }

        self.jitter_offset += (self.jitter_target - self.jitter_offset) * Self::JITTER_RESPONSE;
        self.jitter_offset
    }
}
