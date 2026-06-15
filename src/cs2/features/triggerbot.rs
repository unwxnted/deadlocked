use std::time::{Duration, Instant};

use glam::Vec2;
use rand::rng;

use crate::{
    config::{Config, aim::TriggerbotConfig},
    cs2::{
        CS2,
        bones::Bones,
        entity::{player::Player, weapon_class::WeaponClass},
    },
    math::angles_to_fov,
    os::uinput::InputDevice,
};

#[derive(Debug, Default)]
pub struct Triggerbot {
    shot_start: Option<Instant>,
    shot_end: Option<Instant>,
    pub active: bool,
    magnet_active: bool,
}

impl CS2 {
    pub fn triggerbot(&mut self, root_config: &Config, input_device: &mut InputDevice) -> bool {
        let hotkey = root_config.aim.triggerbot_hotkey;
        let config = self.triggerbot_config(root_config);

        if !config.enabled {
            self.deactivate_magnet_trigger();
            return false;
        }

        if !Self::check_hotkey(&self.input, config.mode, hotkey, &mut self.trigger.active) {
            self.deactivate_magnet_trigger();
            return false;
        }

        let Some(local_player) = Player::local_player(self) else {
            self.deactivate_magnet_trigger();
            return false;
        };

        if config.magnet_enabled {
            return self.magnet_trigger(root_config, config, &local_player, input_device);
        }

        self.deactivate_magnet_trigger();

        if self.trigger.has_pending_shot() {
            return false;
        }

        if config.flash_check && local_player.is_flashed(self) {
            return false;
        }

        if config.scope_check
            && local_player.weapon_class(self) == WeaponClass::Sniper
            && !local_player.is_scoped(self)
        {
            return false;
        }

        if config.velocity_check && local_player.velocity(self).length() > config.velocity_threshold
        {
            return false;
        }

        let Some(player) = local_player.crosshair_entity(self) else {
            return false;
        };

        if !self.is_ffa() && player.team(self) == local_player.team(self) {
            return false;
        }

        if config.head_only {
            let head = player.bone_position(self, Bones::Head.u64());

            let target_angle = self.angle_to_target(&local_player, &head, &Vec2::ZERO);
            let view_angles = local_player.view_angles(self);
            let fov = angles_to_fov(&view_angles, &target_angle);

            let head_radius_fov =
                3.5 / (local_player.position(self) - player.position(self)).length() * 100.0;

            if fov > head_radius_fov {
                return false;
            }
        }

        self.trigger.queue_shot(config);
        false
    }

    fn magnet_trigger(
        &mut self,
        config: &Config,
        trigger_config: &TriggerbotConfig,
        local_player: &Player,
        input_device: &mut InputDevice,
    ) -> bool {
        let aimbot_config = self.aimbot_config(config);
        if !self.aimbot_weapon_allowed(local_player) {
            self.deactivate_magnet_trigger();
            return false;
        }

        if aimbot_config.flash_check && local_player.is_flashed(self) {
            self.deactivate_magnet_trigger();
            return false;
        }

        if trigger_config.scope_check
            && local_player.weapon_class(self) == WeaponClass::Sniper
            && !local_player.is_scoped(self)
        {
            self.deactivate_magnet_trigger();
            return false;
        }

        if trigger_config.velocity_check
            && local_player.velocity(self).length() > trigger_config.velocity_threshold
        {
            self.deactivate_magnet_trigger();
            return false;
        }

        let Some(target) = self.active_target() else {
            self.deactivate_magnet_trigger();
            return false;
        };

        if aimbot_config.visibility_check && !target.visible(self, local_player) {
            self.deactivate_magnet_trigger();
            return false;
        }

        let Some(target_angle) = self.target_angle_from_bones(aimbot_config, local_player, &target)
        else {
            self.deactivate_magnet_trigger();
            return false;
        };

        let view_angles = local_player.view_angles(self);
        let max_fov = self.aimbot_max_fov(aimbot_config, self.target.distance);
        let target_fov = angles_to_fov(&view_angles, &target_angle);
        if target_fov > max_fov {
            self.deactivate_magnet_trigger();
            return false;
        }

        if self.magnet_crosshair_locked(local_player, &target) {
            self.trigger.magnet_active = true;
            if !self.trigger.has_pending_shot() {
                self.trigger.queue_instant_shot(trigger_config);
            }
            return true;
        }

        self.apply_aimbot_step(
            aimbot_config,
            local_player,
            target_angle,
            max_fov,
            input_device,
        );
        self.recoil.previous = local_player.aim_punch(self);
        self.trigger.magnet_active = true;

        true
    }

    fn deactivate_magnet_trigger(&mut self) {
        if self.trigger.magnet_active {
            self.reset_aimbot_state();
            self.trigger.magnet_active = false;
        }
    }

    fn magnet_crosshair_locked(&self, local_player: &Player, target: &Player) -> bool {
        let Some(crosshair_entity) = local_player.crosshair_entity(self) else {
            return false;
        };

        if !self.is_ffa() && crosshair_entity.team(self) == local_player.team(self) {
            return false;
        }

        crosshair_entity.pawn == target.pawn
    }

    pub fn triggerbot_shoot(&mut self, input_device: &mut InputDevice) {
        let now = Instant::now();

        if let Some(shot_time) = self.trigger.shot_start
            && now >= shot_time
        {
            input_device.left_press();
            self.trigger.shot_start = None;
        }

        if let Some(shot_end) = self.trigger.shot_end
            && now >= shot_end
        {
            input_device.left_release();
            self.trigger.shot_end = None;
        }
    }
}

impl Triggerbot {
    fn has_pending_shot(&self) -> bool {
        self.shot_start.is_some() || self.shot_end.is_some()
    }

    fn queue_shot(&mut self, config: &TriggerbotConfig) {
        let mean = (*config.delay.start() + *config.delay.end()) as f32 / 2.0;
        let std_dev = (*config.delay.end() - *config.delay.start()) as f32 / 2.0;

        let delay = if std_dev <= f32::EPSILON {
            mean.max(0.0) as u64
        } else {
            let normal = rand_distr::Normal::new(mean, std_dev).unwrap();
            use rand_distr::Distribution as _;
            normal.sample(&mut rng()).max(0.0) as u64
        };

        let now = Instant::now();
        let delay = Duration::from_millis(delay);
        self.shot_start = Some(now + delay);
        self.shot_end = Some(now + delay + Duration::from_millis(config.shot_duration));
    }

    fn queue_instant_shot(&mut self, config: &TriggerbotConfig) {
        let now = Instant::now();
        self.shot_start = Some(now);
        self.shot_end = Some(now + Duration::from_millis(config.shot_duration));
    }
}
