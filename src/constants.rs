pub mod cs2 {
    use crate::cs2::entity::weapon::Weapon;
    use std::sync::LazyLock;

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

    pub static LIBS: LazyLock<[String; 6]> = LazyLock::new(|| {
        [
            CLIENT_LIB.clone(),
            ENGINE_LIB.clone(),
            TIER0_LIB.clone(),
            INPUT_LIB.clone(),
            SDL_LIB.clone(),
            SCHEMA_LIB.clone(),
        ]
    });

    pub const TEAM_T: u8 = 2;
    pub const TEAM_CT: u8 = 3;

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

        pub(crate) static PLAYER_CONTROLLER: LazyLock<String> =
            LazyLock::new(|| crate::obfstr!("19CCSPlayerController").decrypt());
        pub(crate) static PLANTED_C4: LazyLock<String> =
            LazyLock::new(|| crate::obfstr!("11C_PlantedC4").decrypt());
        pub(crate) static INFERNO: LazyLock<String> =
            LazyLock::new(|| crate::obfstr!("9C_Inferno").decrypt());
        pub(crate) static SMOKE: LazyLock<String> =
            LazyLock::new(|| crate::obfstr!("24C_SmokeGrenadeProjectile").decrypt());
        pub(crate) static MOLOTOV: LazyLock<String> =
            LazyLock::new(|| crate::obfstr!("19C_MolotovProjectile").decrypt());
        pub(crate) static FLASHBANG: LazyLock<String> =
            LazyLock::new(|| crate::obfstr!("21C_FlashbangProjectile").decrypt());
        pub(crate) static HE_GRENADE: LazyLock<String> =
            LazyLock::new(|| crate::obfstr!("21C_HEGrenadeProjectile").decrypt());
        pub(crate) static DECOY: LazyLock<String> =
            LazyLock::new(|| crate::obfstr!("17C_DecoyProjectile").decrypt());
    }
}

pub mod elf {
    pub const PROGRAM_HEADER_OFFSET: usize = 0x20;
    pub const PROGRAM_HEADER_ENTRY_SIZE: usize = 0x36;
    pub const PROGRAM_HEADER_NUM_ENTRIES: usize = 0x38;

    pub const SECTION_HEADER_OFFSET: usize = 0x28;
    pub const SECTION_HEADER_ENTRY_SIZE: usize = 0x3A;
    pub const SECTION_HEADER_NUM_ENTRIES: usize = 0x3C;

    pub const DYNAMIC_SECTION_PHT_TYPE: usize = 0x02;
}

pub static GRENADE_FILE_NAME: std::sync::LazyLock<String> =
    std::sync::LazyLock::new(|| crate::obfstr!("grenades.json").decrypt());
