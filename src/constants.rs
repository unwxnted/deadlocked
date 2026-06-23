use std::sync::LazyLock;

pub mod cs2 {
    use crate::cs2::entity::weapon::Weapon;
    use std::sync::LazyLock;

    pub fn process_name() -> &'static str {
        &PROCESS_NAME
    }
    pub fn client_lib() -> &'static str {
        &CLIENT_LIB
    }
    pub fn engine_lib() -> &'static str {
        &ENGINE_LIB
    }
    pub fn tier0_lib() -> &'static str {
        &TIER0_LIB
    }
    pub fn input_lib() -> &'static str {
        &INPUT_LIB
    }
    pub fn sdl_lib() -> &'static str {
        &SDL_LIB
    }
    pub fn schema_lib() -> &'static str {
        &SCHEMA_LIB
    }

    pub static PROCESS_NAME: LazyLock<String> = LazyLock::new(|| crate::obfstr!("cs2").decrypt());
    pub static CLIENT_LIB: LazyLock<String> =
        LazyLock::new(|| crate::obfstr!("libclient.so").decrypt());
    pub static ENGINE_LIB: LazyLock<String> =
        LazyLock::new(|| crate::obfstr!("libengine2.so").decrypt());
    pub static TIER0_LIB: LazyLock<String> =
        LazyLock::new(|| crate::obfstr!("libtier0.so").decrypt());
    pub static INPUT_LIB: LazyLock<String> =
        LazyLock::new(|| crate::obfstr!("libinputsystem.so").decrypt());
    pub static SDL_LIB: LazyLock<String> =
        LazyLock::new(|| crate::obfstr!("libSDL3.so.0").decrypt());
    pub static SCHEMA_LIB: LazyLock<String> =
        LazyLock::new(|| crate::obfstr!("libschemasystem.so").decrypt());

    pub fn libs() -> [&'static str; 6] {
        [
            client_lib(),
            engine_lib(),
            tier0_lib(),
            input_lib(),
            sdl_lib(),
            schema_lib(),
        ]
    }

    pub const TEAM_T: u8 = 2;
    pub const TEAM_CT: u8 = 3;

    pub fn weapon_unknown() -> &'static str {
        &WEAPON_UNKNOWN
    }
    pub static WEAPON_UNKNOWN: LazyLock<String> =
        LazyLock::new(|| crate::obfstr!("unknown").decrypt());

    pub const DEFAULT_FOV: u32 = 90;

    pub const SOUND_ESP_FOOTSTEP_DIAMETER_DEFAULT: f32 = 2000.0;
    pub const SOUND_ESP_GUNSHOT_DIAMETER_DEFAULT: f32 = 3000.0;
    pub const SOUND_ESP_WEAPON_DIAMETER_DEFAULT: f32 = 1000.0;

    pub const GRENADES: &[Weapon] = &[
        Weapon::Decoy,
        Weapon::Flashbang,
        Weapon::HeGrenade,
        Weapon::Incendiary,
        Weapon::Molotov,
        Weapon::Smoke,
    ];

    pub mod class {
        use std::sync::LazyLock;

        pub fn player_controller() -> &'static str {
            &PLAYER_CONTROLLER
        }
        pub fn planted_c4() -> &'static str {
            &PLANTED_C4
        }
        pub fn inferno() -> &'static str {
            &INFERNO
        }
        pub fn smoke() -> &'static str {
            &SMOKE
        }
        pub fn molotov() -> &'static str {
            &MOLOTOV
        }
        pub fn flashbang() -> &'static str {
            &FLASHBANG
        }
        pub fn he_grenade() -> &'static str {
            &HE_GRENADE
        }
        pub fn decoy() -> &'static str {
            &DECOY
        }

        static PLAYER_CONTROLLER: LazyLock<String> =
            LazyLock::new(|| crate::obfstr!("19CCSPlayerController").decrypt());
        static PLANTED_C4: LazyLock<String> =
            LazyLock::new(|| crate::obfstr!("11C_PlantedC4").decrypt());
        static INFERNO: LazyLock<String> = LazyLock::new(|| crate::obfstr!("9C_Inferno").decrypt());
        static SMOKE: LazyLock<String> =
            LazyLock::new(|| crate::obfstr!("24C_SmokeGrenadeProjectile").decrypt());
        static MOLOTOV: LazyLock<String> =
            LazyLock::new(|| crate::obfstr!("19C_MolotovProjectile").decrypt());
        static FLASHBANG: LazyLock<String> =
            LazyLock::new(|| crate::obfstr!("21C_FlashbangProjectile").decrypt());
        static HE_GRENADE: LazyLock<String> =
            LazyLock::new(|| crate::obfstr!("21C_HEGrenadeProjectile").decrypt());
        static DECOY: LazyLock<String> =
            LazyLock::new(|| crate::obfstr!("17C_DecoyProjectile").decrypt());
    }
}

pub mod elf {
    pub const PROGRAM_HEADER_OFFSET: u64 = 0x20;
    pub const PROGRAM_HEADER_ENTRY_SIZE: u64 = 0x36;
    pub const PROGRAM_HEADER_NUM_ENTRIES: u64 = 0x38;

    pub const SECTION_HEADER_OFFSET: u64 = 0x28;
    pub const SECTION_HEADER_ENTRY_SIZE: u64 = 0x3A;
    pub const SECTION_HEADER_NUM_ENTRIES: u64 = 0x3C;

    pub const DYNAMIC_SECTION_PHT_TYPE: u64 = 0x02;
}

pub fn grenade_file_name() -> &'static str {
    &GRENADE_FILE_NAME
}
pub static GRENADE_FILE_NAME: LazyLock<String> =
    LazyLock::new(|| crate::obfstr!("grenades.json").decrypt());
