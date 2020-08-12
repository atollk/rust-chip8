extern crate sfml;

use super::emulator::vm::VMInterface;
use crate::emulator::basics::{SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::emulator::vm::Display;
use sfml::audio::{Sound, SoundBuffer, SoundSource};
use sfml::graphics::{Color, RectangleShape, RenderTarget, RenderWindow, Shape, Transformable};
use sfml::system::{SfBox, Vector2f};
use sfml::window::{ContextSettings, Event, Style, VideoMode};
use std::iter;
use std::{
    collections::HashMap,
    sync::{Arc, Condvar, Mutex},
    thread::JoinHandle,
};

const SCALE: usize = 16;
const SOUND_FILENAME: &str = "final-fantasy-viii-sound-effects-cursor-move.ogg";

pub struct Visualizer {
    setup_done: Arc<(Mutex<bool>, Condvar)>,
    join_handle: JoinHandle<()>,
}

struct VisualizerInternals<'a> {
    window: RenderWindow,
    pixels: [[RectangleShape<'a>; SCREEN_HEIGHT as usize]; SCREEN_WIDTH as usize],
    vm_interface: &'a Mutex<VMInterface>,
    sound_buffer: SfBox<SoundBuffer>,
    keymap: HashMap<u8, sfml::window::Key>,
}

impl<'a> VisualizerInternals<'a> {
    fn new(
        vm_interface: &'a Mutex<VMInterface>,
        keymap: HashMap<u8, sfml::window::Key>,
    ) -> VisualizerInternals<'a> {
        VisualizerInternals {
            window: VisualizerInternals::init_window(),
            pixels: VisualizerInternals::init_pixels(),
            vm_interface,
            sound_buffer: SoundBuffer::from_file(SOUND_FILENAME).unwrap(),
            keymap,
        }
    }

    fn init_window() -> RenderWindow {
        let video_mode = VideoMode::new(
            SCREEN_WIDTH as u32 * SCALE as u32,
            SCREEN_HEIGHT as u32 * SCALE as u32,
            32,
        );
        let mut window = RenderWindow::new(
            video_mode,
            "Chip 8 Emulator",
            Style::CLOSE,
            &ContextSettings::default(),
        );
        window.set_framerate_limit(60);
        window
    }

    fn init_pixels() -> [[RectangleShape<'static>; SCREEN_HEIGHT as usize]; SCREEN_WIDTH as usize] {
        let mut pixels: [[RectangleShape; SCREEN_HEIGHT as usize]; SCREEN_WIDTH as usize] =
            iter::repeat(
                iter::repeat(RectangleShape::new())
                    .collect::<arrayvec::ArrayVec<_>>()
                    .into_inner()
                    .unwrap(),
            )
            .collect::<arrayvec::ArrayVec<_>>()
            .into_inner()
            .unwrap();
        for x in 0..SCREEN_WIDTH as usize {
            for y in 0..SCREEN_HEIGHT as usize {
                let pixel = &mut pixels[x][y];
                pixel.set_size(Vector2f::new(SCALE as f32, SCALE as f32));
                pixel.set_position(Vector2f::new((SCALE * x) as f32, (SCALE * y) as f32));
                pixel.set_fill_color(Color::WHITE);
            }
        }
        pixels
    }
}

impl Visualizer {
    pub fn new(
        vm_interface: Arc<Mutex<VMInterface>>,
        display_fade: u32,
        keymap: HashMap<u8, sfml::window::Key>,
    ) -> Visualizer {
        let setup_done = Arc::new((Mutex::new(false), Condvar::new()));
        let setup_done2 = setup_done.clone();
        let join_handle = std::thread::spawn(move || {
            vm_interface.lock().unwrap().display = Box::new(FadeDisplay::new(display_fade));
            let mut internals = VisualizerInternals::new(&*vm_interface, keymap);
            {
                let (mutex, condvar) = &*setup_done2;
                *mutex.lock().unwrap() = true;
                condvar.notify_all();
            }
            run(&mut internals);
        });
        Visualizer {
            setup_done,
            join_handle,
        }
    }

    pub fn wait_for_init(&self) {
        let (mutex, condvar) = &*self.setup_done;
        let guard = mutex.lock().unwrap();
        if !*guard {
            condvar.wait(guard).unwrap();
        }
    }

    pub fn wait_for_close(self) {
        self.join_handle.join().unwrap();
    }
}

struct FadeDisplay {
    fade_duration: u32,
    display: [[u32; SCREEN_HEIGHT as usize]; SCREEN_WIDTH as usize],
    true_display: [[bool; SCREEN_HEIGHT as usize]; SCREEN_WIDTH as usize],
}

impl FadeDisplay {
    pub fn new(fade_duration: u32) -> FadeDisplay {
        FadeDisplay {
            fade_duration,
            display: [[0; SCREEN_HEIGHT as usize]; SCREEN_WIDTH as usize],
            true_display: [[false; SCREEN_HEIGHT as usize]; SCREEN_WIDTH as usize],
        }
    }
}

impl Display for FadeDisplay {
    fn clear(&mut self) {
        for column in self.true_display.iter_mut() {
            for pixel in column.iter_mut() {
                *pixel = false;
            }
        }
        for column in self.display.iter_mut() {
            for pixel in column.iter_mut() {
                *pixel = 0;
            }
        }
    }

    fn draw_pixels(&mut self, pixels: &[(u8, u8)]) {
        for (x, y) in pixels {
            let true_pixel = &mut self.true_display[*x as usize][*y as usize];
            if *true_pixel {
                *true_pixel = false;
            } else {
                *true_pixel = true;
                self.display[*x as usize][*y as usize] = self.fade_duration;
            }
        }
    }

    fn get(&self, x: u8, y: u8) -> u8 {
        (self.display[x as usize][y as usize] * 255 / self.fade_duration) as u8
    }

    fn frame(&mut self) {
        for x in 0..SCREEN_WIDTH as usize {
            for y in 0..SCREEN_HEIGHT as usize {
                if !self.true_display[x][y] && self.display[x][y] > 0 {
                    self.display[x][y] -= 1;
                }
            }
        }
    }
}

fn run(internals: &mut VisualizerInternals) {
    let mut keys_pressed = [false; 16];
    let mut sound = Sound::with_buffer(&*internals.sound_buffer);
    sound.set_volume(10.0);
    sound.set_pitch(100.0);

    while internals.window.is_open() {
        // Handle events
        while let Some(event) = internals.window.poll_event() {
            match event {
                Event::Closed => internals.window.close(),
                Event::KeyPressed { code, .. } => {
                    if let Some((i, _)) = internals
                        .keymap
                        .iter()
                        .find(|(_, k)| **k == code)
                    {
                        keys_pressed[*i as usize] = true;
                    }
                }
                Event::KeyReleased { code, .. } => {
                    if let Some((i, _)) = internals
                        .keymap
                        .iter()
                        .find(|(_, k)| **k == code)
                    {
                        keys_pressed[*i as usize] = false;
                    }
                }
                _ => { /* do nothing */ }
            }
        }

        // Update keymap in VM.
        {
            let key_down = &mut internals.vm_interface.lock().unwrap().key_down;
            *key_down = None;
            for (i, k) in keys_pressed.iter().enumerate() {
                if *k {
                    *key_down = Some(i as u8);
                }
            }
        }

        // Sound
        if internals.vm_interface.lock().unwrap().sound_timer.0 > 0 {
            sound.play();
        }

        // Draw
        internals.window.clear(Color::BLACK);
        for x in 0..SCREEN_WIDTH {
            for y in 0..SCREEN_HEIGHT {
                let pixel = &mut internals.pixels[x as usize][y as usize];
                let alpha = internals.vm_interface.lock().unwrap().display.get(x, y);
                pixel.set_fill_color(Color::rgba(255, 255, 255, alpha));
                internals.window.draw(pixel);
            }
        }
        internals.vm_interface.lock().unwrap().display.frame();
        internals.window.display()
    }
}
