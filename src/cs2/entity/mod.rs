use glam::Vec3;
use serde::{Deserialize, Serialize};

use crate::{
    constants::cs2::class,
    cs2::{
        CS2,
        entity::{
            inferno::{Inferno, InfernoInfo},
            molotov::{Molotov, MolotovInfo},
            planted_c4::PlantedC4,
            player::Player,
            smoke::{Smoke, SmokeInfo},
            weapon::Weapon,
        },
    },
};

pub mod inferno;
pub mod molotov;
pub mod planted_c4;
pub mod player;
pub mod smoke;
pub mod weapon;
pub mod weapon_class;

#[derive(Clone)]
pub enum Entity {
    Weapon { weapon: Weapon, entity: usize },
    Inferno(Inferno),
    Smoke(Smoke),
    Molotov(Molotov),
    Flashbang(usize),
    HeGrenade(usize),
    Decoy(usize),
}

#[derive(Clone, Serialize, Deserialize)]
pub enum EntityInfo {
    Weapon {
        weapon: Weapon,
        position: Vec3,
        ammo: (i32, i32),
    },
    Inferno(InfernoInfo),
    Smoke(SmokeInfo),
    Molotov(MolotovInfo),
    Flashbang(GrenadeInfo),
    HeGrenade(GrenadeInfo),
    Decoy(GrenadeInfo),
}

#[derive(Clone, Serialize, Deserialize)]
pub struct GrenadeInfo {
    pub entity: usize,
    pub position: Vec3,
    pub name: String,
}

impl GrenadeInfo {
    pub fn new(entity: usize, name: &str, cs2: &CS2) -> Self {
        Self {
            entity,
            position: Player::entity(entity).position(cs2),
            name: name.to_owned(),
        }
    }
}

impl CS2 {
    pub fn cache_entities(&mut self) {
        self.players.clear();
        self.dead_players.clear();
        self.entities.clear();
        self.planted_c4 = None;

        let Some(local_player) = Player::local_player(self) else {
            return;
        };

        self.weapon = local_player.weapon(self);

        const NUM_BUCKETS: usize = 64;
        let bucket_pointers = self
            .process
            .read_vec(self.offsets.interface.entity, 0x8 * NUM_BUCKETS);
        for bucket_index in 0..64 {
            let bucket_pointer =
                *bytemuck::from_bytes(&bucket_pointers[bucket_index * 8..(bucket_index + 1) * 8]);
            self.get_entities_in_bucket(bucket_index, bucket_pointer, &local_player);
        }
    }

    fn get_entities_in_bucket(
        &mut self,
        bucket_index: usize,
        bucket_ptr: usize,
        local_player: &Player,
    ) {
        if bucket_ptr == 0 || bucket_ptr >> 48 != 0 {
            return;
        }
        const IDENTITIES_PER_BUCKET: usize = 512;
        let bucket = self.process.read_vec(
            bucket_ptr,
            IDENTITIES_PER_BUCKET * self.offsets.entity_identity.size as usize,
        );
        for index_in_bucket in 0..IDENTITIES_PER_BUCKET {
            let identity_offset = index_in_bucket * self.offsets.entity_identity.size as usize;

            let entity: usize = *bytemuck::from_bytes(&bucket[identity_offset..identity_offset + 8]);
            if entity == 0 {
                continue;
            }

            let handle_start = identity_offset + 0x10;
            let handle: u32 = *bytemuck::from_bytes(&bucket[handle_start..handle_start + 4]);
            let handle_index = handle & 0x7FFF;
            let entity_index =
                (bucket_index * IDENTITIES_PER_BUCKET + index_in_bucket) as u32;
            if entity_index != handle_index {
                continue;
            }

            let vtable: usize = self.process.read(entity);
            let rtti: usize = self.process.read(vtable - 0x8);
            let name_ptr: usize = self.process.read(rtti + 0x8);
            let name = self.process.read_string(name_ptr);

            let name_str = name.as_str();
            match name_str {
                s if s == *class::PLAYER_CONTROLLER => {
                    let Some(player) = Player::from_controller(entity, self) else {
                        continue;
                    };

                    if !player.is_valid(self) {
                        self.dead_players.push(player);
                        continue;
                    }

                    if player == *local_player {
                        self.target.local_pawn_index = (handle as usize & 0x7FFF) - 1;
                    } else {
                        self.players.push(player);
                    }
                }
                s if s == *class::PLANTED_C4 => {
                    let planted_c4 = PlantedC4::new(entity);
                    if planted_c4.is_relevant(self) {
                        self.planted_c4 = Some(planted_c4)
                    }
                }
                s if s == *class::INFERNO => {
                    self.entities.push(Entity::Inferno(Inferno::new(entity)));
                }
                s if s == *class::SMOKE => {
                    self.entities.push(Entity::Smoke(Smoke::new(entity)));
                }
                s if s == *class::MOLOTOV => {
                    self.entities.push(Entity::Molotov(Molotov::new(entity)));
                }
                s if s == *class::FLASHBANG => {
                    self.entities.push(Entity::Flashbang(entity));
                }
                s if s == *class::HE_GRENADE => {
                    self.entities.push(Entity::HeGrenade(entity));
                }
                s if s == *class::DECOY => {
                    self.entities.push(Entity::Decoy(entity));
                }
                _ => {
                    let entity_identity: usize = self.process.read(entity + 0x10);
                    if entity_identity == 0 {
                        continue;
                    }

                    let name_pointer = self.process.read(entity_identity + 0x20);
                    if name_pointer == 0 {
                        continue;
                    }

                    let name = self.process.read_string(name_pointer);

                    if name.starts_with(&crate::obfstr!("weapon_").decrypt()) {
                        if self.entity_has_owner(entity) {
                            continue;
                        }

                        let weapon = Weapon::from_entity(entity, self);

                        self.entities.push(Entity::Weapon { weapon, entity });
                    }
                }
            }
        }
    }
}
