use crate::emulator::executor::Executor;
use crate::emulator::vm::VirtualMachine;
use crate::visualizer::Visualizer;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::{fs::File, io::Read, time::Duration};

const TIMER_INTERVAL: Duration = Duration::from_micros(16667);

struct Config {
    filename: &'static str,
    display_fade: u32,
    instruction_sleep: Duration,
    keymap: HashMap<u8, sfml::window::Key>,
}

lazy_static! {
    static ref DEFAULT_KEYMAP: HashMap<u8, sfml::window::Key> = vec![
        (0, sfml::window::Key::Num0),
        (1, sfml::window::Key::Num1),
        (2, sfml::window::Key::Num2),
        (3, sfml::window::Key::Num3),
        (4, sfml::window::Key::Num4),
        (5, sfml::window::Key::Num5),
        (6, sfml::window::Key::Num6),
        (7, sfml::window::Key::Num7),
        (8, sfml::window::Key::Num8),
        (9, sfml::window::Key::Num9),
        (10, sfml::window::Key::A),
        (11, sfml::window::Key::B),
        (12, sfml::window::Key::C),
        (13, sfml::window::Key::D),
        (14, sfml::window::Key::E),
        (15, sfml::window::Key::F),
    ]
    .into_iter()
    .collect();

    static ref TABLE_KEYMAP: HashMap<u8, sfml::window::Key> = vec![
        (0, sfml::window::Key::X),
        (1, sfml::window::Key::Num1),
        (2, sfml::window::Key::Num2),
        (3, sfml::window::Key::Num3),
        (4, sfml::window::Key::Q),
        (5, sfml::window::Key::W),
        (6, sfml::window::Key::E),
        (7, sfml::window::Key::A),
        (8, sfml::window::Key::S),
        (9, sfml::window::Key::D),
        (10, sfml::window::Key::Y),
        (11, sfml::window::Key::C),
        (12, sfml::window::Key::Num4),
        (13, sfml::window::Key::R),
        (14, sfml::window::Key::F),
        (15, sfml::window::Key::V),
    ]
    .into_iter()
    .collect();
}

lazy_static! {
static ref ROM_MAP: HashMap<&'static str, Config> = vec![
    ("15puzzle" , Config { 
        filename: "roms/15PUZZLE",
        display_fade: 1,
        instruction_sleep: Duration::from_micros(100),
        keymap: TABLE_KEYMAP.clone()
    }),
    ("blinky" , Config {
        filename: "roms/BLINKY",
        display_fade: 1,
        instruction_sleep: Duration::from_millis(1),
        keymap: vec![
            (3, sfml::window::Key::Up),
            (6, sfml::window::Key::Down),
            (7, sfml::window::Key::Left),
            (8, sfml::window::Key::Right),
        ]
        .into_iter()
        .collect()
    }),
    ("blitz" , Config { // todo
        filename: "roms/BLITZ",
        display_fade: 3,
        instruction_sleep: Duration::from_millis(2),
        keymap: DEFAULT_KEYMAP.clone(),
    }),
    ("brix" , Config { // todo
        filename: "roms/BRIX",
        display_fade: 3,
        instruction_sleep: Duration::from_millis(2),
        keymap: DEFAULT_KEYMAP.clone(),
    }),
    ("connect4" , Config { // todo
        filename: "roms/CONNECT4",
        display_fade: 3,
        instruction_sleep: Duration::from_millis(15),
        keymap: vec![
            (4, sfml::window::Key::Left),
            (5, sfml::window::Key::Down),
            (6, sfml::window::Key::Right),
        ]
        .into_iter()
        .collect()
    }),
    ("guess" , Config { // todo
        filename: "roms/GUESS",
        display_fade: 3,
        instruction_sleep: Duration::from_millis(2),
        keymap: DEFAULT_KEYMAP.clone(),
    }),
    ("hidden" , Config { // todo
        filename: "roms/HIDDEN",
        display_fade: 3,
        instruction_sleep: Duration::from_millis(2),
        keymap: DEFAULT_KEYMAP.clone(),
    }),
    ("invaders" , Config { // todo
        filename: "roms/INVADERS",
        display_fade: 3,
        instruction_sleep: Duration::from_millis(2),
        keymap: DEFAULT_KEYMAP.clone(),
    }),
    ("kaleid" , Config { // todo
        filename: "roms/KALEID",
        display_fade: 3,
        instruction_sleep: Duration::from_millis(2),
        keymap: DEFAULT_KEYMAP.clone(),
    }),
    ("maze" , Config { // todo
        filename: "roms/MAZE",
        display_fade: 3,
        instruction_sleep: Duration::from_millis(2),
        keymap: DEFAULT_KEYMAP.clone(),
    }),
    ("merlin" , Config { // todo
        filename: "roms/MERLIN",
        display_fade: 3,
        instruction_sleep: Duration::from_millis(2),
        keymap: DEFAULT_KEYMAP.clone(),
    }),
    ("missile" , Config { // todo
        filename: "roms/MISSILE",
        display_fade: 3,
        instruction_sleep: Duration::from_millis(2),
        keymap: DEFAULT_KEYMAP.clone(),
    }),
    ("pong" , Config { // todo
        filename: "roms/PONG",
        display_fade: 3,
        instruction_sleep: Duration::from_millis(2),
        keymap: DEFAULT_KEYMAP.clone(),
    }),
    ("pong2" , Config { // todo
        filename: "roms/PONG2",
        display_fade: 3,
        instruction_sleep: Duration::from_millis(2),
        keymap: DEFAULT_KEYMAP.clone(),
    }),
    ("puzzle" , Config { // todo
        filename: "roms/PUZZLE",
        display_fade: 3,
        instruction_sleep: Duration::from_millis(1),
        keymap: DEFAULT_KEYMAP.clone(),
    }),
    ("syzygy" , Config { // todo
        filename: "roms/SYZYGY",
        display_fade: 3,
        instruction_sleep: Duration::from_millis(2),
        keymap: DEFAULT_KEYMAP.clone(),
    }),
    ("tank" , Config { // todo
        filename: "roms/TANK",
        display_fade: 3,
        instruction_sleep: Duration::from_millis(2),
        keymap: DEFAULT_KEYMAP.clone(),
    }),
    ("tetris" , Config { // todo
        filename: "roms/TETRIS",
        display_fade: 3,
        instruction_sleep: Duration::from_millis(2),
        keymap: DEFAULT_KEYMAP.clone(),
    }),
    ("tictac" , Config { // todo
        filename: "roms/TICTAC",
        display_fade: 3,
        instruction_sleep: Duration::from_millis(2),
        keymap: DEFAULT_KEYMAP.clone(),
    }),
    ("ufo" , Config { // todo
        filename: "roms/UFO",
        display_fade: 3,
        instruction_sleep: Duration::from_millis(2),
        keymap: DEFAULT_KEYMAP.clone(),
    }),
    ("vbrix" , Config { // todo
        filename: "roms/VBRIX",
        display_fade: 3,
        instruction_sleep: Duration::from_millis(2),
        keymap: DEFAULT_KEYMAP.clone(),
    }),
    ("vers" , Config { // todo
        filename: "roms/VERS",
        display_fade: 3,
        instruction_sleep: Duration::from_millis(2),
        keymap: DEFAULT_KEYMAP.clone(),
    }),
    ("wipeoff" , Config { // todo
        filename: "roms/WIPEOFF",
        display_fade: 3,
        instruction_sleep: Duration::from_millis(2),
        keymap: DEFAULT_KEYMAP.clone(),
    }),
].into_iter().collect();
}

fn load_rom_file(filename: &str) -> Vec<u8> {
    let mut file = File::open(filename).unwrap();
    let mut raw_rom = Vec::new();
    file.read_to_end(&mut raw_rom).unwrap();
    raw_rom
}

pub fn load_rom(rom_name: &str) -> (Executor, Visualizer) {
    let config = &ROM_MAP[rom_name];
    let vm = VirtualMachine::new(&load_rom_file(config.filename));
    let visualizer = Visualizer::new(
        vm.interface.clone(),
        config.display_fade,
        config.keymap.clone(),
    );
    let executor = Executor::new(config.instruction_sleep, TIMER_INTERVAL, vm);
    (executor, visualizer)
}
