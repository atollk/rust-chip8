extern crate sfml;

use super::emulator::vm::VMInterface;
use crate::emulator::basics::{SCREEN_HEIGHT, SCREEN_WIDTH};
use sfml::graphics::{Color, RectangleShape, RenderTarget, RenderWindow, Shape, Transformable};
use sfml::system::Vector2f;
use sfml::window::{ContextSettings, Event, Style, VideoMode};
use std::iter;
use std::sync::{Arc, Condvar, Mutex};

const SCALE: usize = 8;

pub struct Visualizer {
    setup_done: Arc<(Mutex<bool>, Condvar)>,
}

impl Visualizer {
    pub fn new(vm_interface: Arc<Mutex<VMInterface>>) -> Visualizer {
        let setup_done = Arc::new((Mutex::new(false), Condvar::new()));
        let setup_done2 = setup_done.clone();
        let vis = Visualizer { setup_done };
        let thread = std::thread::spawn(move || {
            let mut window = init_window();
            let pixels = init_pixels();
            {
                let (mutex, condvar) = &*setup_done2;
                *mutex.lock().unwrap() = true;
                condvar.notify_all();
            }
            run(&mut window, &pixels, vm_interface);
        });
        vis
    }

    pub fn wait_for_init(&self) {
        let (mutex, condvar) = &*self.setup_done;
        let guard = mutex.lock().unwrap();
        if !*guard {
            condvar.wait(guard);
        }
    }
}

fn init_window() -> RenderWindow {
    let video_mode = VideoMode::new(
        SCREEN_WIDTH as u32 * SCALE as u32,
        SCREEN_HEIGHT as u32 * SCALE as u32,
        32,
    );
    RenderWindow::new(
        video_mode,
        "Chip 8 Emulator",
        Style::CLOSE,
        &ContextSettings::default(),
    )
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
    while window.is_open() {
        // Handle events
        while let Some(event) = window.poll_event() {
            match event {
                Event::Closed => window.close(),
                _ => { /* do nothing */ }
            }
        }

        // Draw
        window.clear(Color::BLACK);
        for x in 0..SCREEN_WIDTH as usize {
            for y in 0..SCREEN_HEIGHT as usize {
                if vm_interface.lock().unwrap().display[x][y] {
                    window.draw(&pixels[x][y]);
                }
            }
        }
        window.display()
    }
}
