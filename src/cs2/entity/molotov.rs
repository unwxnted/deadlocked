use glam::Vec3;
use serde::{Deserialize, Serialize};

use crate::cs2::{CS2, entity::player::Player};

#[derive(Clone, PartialEq, Serialize)]
pub struct Molotov {
    controller: usize,
}

impl Molotov {
    pub fn new(controller: usize) -> Self {
        Self { controller }
    }

    pub fn info(&self, cs2: &CS2) -> MolotovInfo {
        MolotovInfo {
            entity: self.controller,
            position: Player::entity(self.controller).position(cs2),
            is_incendiary: self.is_incendiary(cs2),
        }
    }

    pub fn is_incendiary(&self, cs2: &CS2) -> bool {
        cs2.process
            .read::<u8>(self.controller + cs2.offsets.molotov.is_incendiary)
            != 0
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct MolotovInfo {
    pub entity: usize,
    pub position: Vec3,
    pub is_incendiary: bool,
}

impl MolotovInfo {
    pub fn grenade(&self) -> super::GrenadeInfo {
        super::GrenadeInfo {
            entity: self.entity,
            position: self.position,
            name: if self.is_incendiary {
                "Incendiary".to_owned()
            } else {
                "Molotov".to_owned()
            },
        }
    }
}
