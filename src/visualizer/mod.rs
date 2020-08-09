extern crate sfml;

use super::emulator::vm::VMInterface;
use crate::emulator::basics::{SCREEN_HEIGHT, SCREEN_WIDTH};
use crate::emulator::vm::Display;
use sfml::graphics::{Color, RectangleShape, RenderTarget, RenderWindow, Shape, Transformable};
use sfml::system::Vector2f;
use sfml::window::{ContextSettings, Event, Style, VideoMode};
use std::iter;
use std::{
    sync::{Arc, Condvar, Mutex},
    thread::JoinHandle,
};

const SCALE: usize = 16;
const KEYS: [sfml::window::Key; 16] = [
    sfml::window::Key::Num1,
    sfml::window::Key::Num2,
    sfml::window::Key::Num3,
    sfml::window::Key::Num4,
    sfml::window::Key::Q,
    sfml::window::Key::W,
    sfml::window::Key::E,
    sfml::window::Key::R,
    sfml::window::Key::A,
    sfml::window::Key::S,
    sfml::window::Key::D,
    sfml::window::Key::F,
    sfml::window::Key::Y,
    sfml::window::Key::X,
    sfml::window::Key::C,
    sfml::window::Key::V,
];

pub struct Visualizer {
    setup_done: Arc<(Mutex<bool>, Condvar)>,
    join_handle: JoinHandle<()>,
}

impl Visualizer {
    pub fn new(vm_interface: Arc<Mutex<VMInterface>>) -> Visualizer {
        let setup_done = Arc::new((Mutex::new(false), Condvar::new()));
        let setup_done2 = setup_done.clone();
        let join_handle = std::thread::spawn(move || {
            vm_interface.lock().unwrap().display = Box::new(BufferedDisplay::new());
            let mut window = init_window();
            let pixels = init_pixels();
            {
                let (mutex, condvar) = &*setup_done2;
                *mutex.lock().unwrap() = true;
                condvar.notify_all();
            }
            run(&mut window, &pixels, vm_interface);
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

struct BufferedDisplay {
    true_display: [[bool; SCREEN_HEIGHT as usize]; SCREEN_WIDTH as usize],
    buffered_display: [[bool; SCREEN_HEIGHT as usize]; SCREEN_WIDTH as usize],
}

impl BufferedDisplay {
    pub fn new() -> BufferedDisplay {
        BufferedDisplay {
            true_display: [[false; SCREEN_HEIGHT as usize]; SCREEN_WIDTH as usize],
            buffered_display: [[false; SCREEN_HEIGHT as usize]; SCREEN_WIDTH as usize],
        }
    }
}

impl Display for BufferedDisplay {
    fn clear(&mut self) {
        for column in self.true_display.iter_mut() {
            for pixel in column.iter_mut() {
                *pixel = false;
            }
        }
        for column in self.buffered_display.iter_mut() {
            for pixel in column.iter_mut() {
                *pixel = false;
            }
        }
    }

    fn draw_pixels(&mut self, pixels: &[(u8, u8)]) {
        for (x, y) in pixels {
            self.buffered_display[*x as usize][*y as usize] = true;
            let true_pixel = &mut self.true_display[*x as usize][*y as usize];
            *true_pixel = !*true_pixel;
        }
    }

    fn get(&self, x: u8, y: u8) -> &bool {
        &self.buffered_display[x as usize][y as usize]
    }

    fn frame(&mut self) {
        self.buffered_display = self.true_display;
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

fn run(
    window: &mut RenderWindow,
    pixels: &[[RectangleShape; SCREEN_HEIGHT as usize]; SCREEN_WIDTH as usize],
    vm_interface: Arc<Mutex<VMInterface>>,
) {
    let mut keys_pressed = [false; 16];
    while window.is_open() {
        // Handle events
        while let Some(event) = window.poll_event() {
            match event {
                Event::Closed => window.close(),
                Event::KeyPressed { code, .. } => {
                    if let Some((i, _)) = KEYS.iter().enumerate().find(|(i, k)| **k == code) {
                        keys_pressed[i] = true;
                    }
                }
                Event::KeyReleased { code, .. } => {
                    if let Some((i, _)) = KEYS.iter().enumerate().find(|(i, k)| **k == code) {
                        keys_pressed[i] = false;
                    }
                }
                _ => { /* do nothing */ }
            }
        }

        // Update keys in VM.
        {
            let key_down = &mut vm_interface.lock().unwrap().key_down;
            *key_down = None;
            for (i, k) in keys_pressed.iter().enumerate() {
                if *k {
                    *key_down = Some(i as u8);
                }
            }
        }

        // Draw
        window.clear(Color::BLACK);
        for x in 0..SCREEN_WIDTH {
            for y in 0..SCREEN_HEIGHT {
                if *vm_interface.lock().unwrap().display.get(x, y) {
                    window.draw(&pixels[x as usize][y as usize]);
                }
            }
        }
        vm_interface.lock().unwrap().display.frame();
        window.display()
    }
}
