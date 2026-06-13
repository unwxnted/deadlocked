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
struct JitterLayer {
    offset: Vec2,
    target: Vec2,
    last_update: Option<Instant>,
}

#[derive(Debug, Default)]
pub struct Aimbot {
    pub active: bool,
    inertia: Vec2,
    jitter: JitterLayer,
    micro_jitter: JitterLayer,
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
        let max_fov = config.fov
            * if config.distance_adjusted_fov {
                self.distance_scale(self.target.distance)
            } else {
                1.0
            };

        if angles_to_fov(&view_angles, &target_angle) > max_fov {
            self.aim.reset_jitter();
            return false;
        }

        let jitter_offset = self.aim.jitter(config);
        let mut jittered_target_angle = target_angle + jitter_offset;
        vec2_clamp(&mut jittered_target_angle);

        let mut aim_angles = view_angles - jittered_target_angle;
        if aim_angles.y < -180.0 {
            aim_angles.y += 360.0
        }
        vec2_clamp(&mut aim_angles);

        let sensitivity = self.get_sensitivity() * local_player.fov_multiplier(self);
        let smooth_divisor = self.aim.smooth_divisor(config, aim_angles, max_fov);

        let mouse_angles = vec2(
            aim_angles.y / sensitivity * 45.45,
            -aim_angles.x / sensitivity * 45.45,
        ) / smooth_divisor;

        let alpha = 1.0 - config.inertia.clamp(0.0, 1.0) * 0.5;
        self.aim.inertia += (mouse_angles - self.aim.inertia) * alpha;
        input_device.move_rel(self.aim.inertia);

        self.recoil.previous = local_player.aim_punch(self);

        true
    }
}

impl Aimbot {
    const JITTER_UPDATE_INTERVAL: Duration = Duration::from_millis(40);
    const MICRO_JITTER_UPDATE_INTERVAL: Duration = Duration::from_millis(12);

    fn reset_jitter(&mut self) {
        self.jitter.reset();
        self.micro_jitter.reset();
    }

    fn jitter(&mut self, config: &AimbotConfig) -> Vec2 {
        if !config.jitter.enabled {
            self.reset_jitter();
            return Vec2::ZERO;
        }

        let amount = config.jitter.amount.max(0.0);
        let micro_amount = config.jitter.micro_amount.max(0.0);

        let main_offset = if amount <= f32::EPSILON {
            self.jitter.reset();
            Vec2::ZERO
        } else {
            self.jitter.step(
                amount,
                Self::jitter_smooth_response(config.jitter.smooth),
                Self::JITTER_UPDATE_INTERVAL,
            )
        };

        let micro_offset = if micro_amount <= f32::EPSILON {
            self.micro_jitter.reset();
            Vec2::ZERO
        } else {
            self.micro_jitter.step(
                micro_amount,
                Self::jitter_smooth_response(config.jitter.micro_smooth),
                Self::MICRO_JITTER_UPDATE_INTERVAL,
            )
        };

        main_offset + micro_offset
    }

    fn smooth_divisor(&self, config: &AimbotConfig, aim_delta: Vec2, max_fov: f32) -> f32 {
        let base = self.base_smooth_divisor(config, aim_delta, max_fov);
        if !config.humanization_enabled {
            return base;
        }

        let strength = config.humanization_strength.clamp(0.0, 1.0);
        if strength <= f32::EPSILON {
            return base;
        }

        let error_ratio = Self::error_ratio(aim_delta, max_fov);
        let curve = 1.0 - (1.0 - error_ratio).powi(2);
        let near_scale = 1.0 + strength * 0.65;
        let far_scale = 1.0 - strength * 0.45;
        let dynamic_scale = near_scale + (far_scale - near_scale) * curve;
        (base * dynamic_scale).clamp(0.35, 20.0)
    }

    fn base_smooth_divisor(&self, config: &AimbotConfig, aim_delta: Vec2, max_fov: f32) -> f32 {
        let base = (config.smooth + 1.0).clamp(1.0, 20.0);
        if !config.distance_adjusted_fov {
            return base;
        }

        let error_ratio = Self::error_ratio(aim_delta, max_fov);
        let curve = 1.0 - (1.0 - error_ratio).powi(2);
        let near_scale = 1.12;
        let far_scale = 0.88;
        let dynamic_scale = near_scale + (far_scale - near_scale) * curve;
        (base * dynamic_scale).clamp(0.35, 20.0)
    }

    fn jitter_smooth_response(smooth: f32) -> f32 {
        (1.0 / (smooth + 1.0).clamp(1.0, 20.0)).clamp(0.02, 1.0)
    }

    fn error_ratio(aim_delta: Vec2, max_fov: f32) -> f32 {
        (aim_delta.length() / max_fov.max(0.1)).clamp(0.0, 1.0)
    }
}

impl JitterLayer {
    fn reset(&mut self) {
        self.offset = Vec2::ZERO;
        self.target = Vec2::ZERO;
        self.last_update = None;
    }

    fn step(&mut self, amount: f32, response: f32, update_interval: Duration) -> Vec2 {
        let now = Instant::now();
        if self
            .last_update
            .is_none_or(|last| now.duration_since(last) >= update_interval)
        {
            self.target = vec2(
                rng().random_range(-amount..=amount),
                rng().random_range(-amount..=amount),
            );
            self.last_update = Some(now);
        }

        self.offset += (self.target - self.offset) * response.clamp(0.0, 1.0);
        self.offset
    }
}
