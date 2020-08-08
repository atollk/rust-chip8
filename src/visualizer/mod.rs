extern crate sfml;

use sfml::system::Vector2f;
use sfml::window::{ContextSettings, VideoMode, Event, Style};
use sfml::graphics::{RenderWindow, RenderTarget, RectangleShape, Color, Transformable, Shape};
use super::emulator::vm::VMInterface;
use std::sync::{Mutex, Arc};
use crate::emulator::basics::{SCREEN_HEIGHT, SCREEN_WIDTH};
use std::iter;

const SCALE: usize = 8;

fn init_window() -> RenderWindow {
    let video_mode = VideoMode::new(SCREEN_WIDTH as u32*SCALE as u32, SCREEN_HEIGHT as u32*SCALE as u32, 32);
    RenderWindow::new(video_mode,
        "Chip 8 Emulator",
        Style::CLOSE,
        &ContextSettings::default())
}

fn init_pixels() -> [[RectangleShape<'static>; SCREEN_HEIGHT as usize]; SCREEN_WIDTH as usize] {
    let mut pixels: [[RectangleShape; SCREEN_HEIGHT as usize]; SCREEN_WIDTH as usize] = 
        iter::repeat(
            iter::repeat(RectangleShape::new()).collect::<arrayvec::ArrayVec<_>>().into_inner().unwrap()
        ).collect::<arrayvec::ArrayVec<_>>().into_inner().unwrap();
    for x in 0..SCREEN_WIDTH as usize {
        for y in 0..SCREEN_HEIGHT as usize {
            let pixel = &mut pixels[x][y];
            pixel.set_size(Vector2f::new(SCALE as f32, SCALE as f32));
            pixel.set_position(Vector2f::new((SCALE*x) as f32, (SCALE*y) as f32));
            pixel.set_fill_color(Color::WHITE);
        }
    }
    pixels
}

pub fn main(vm_interface: Arc<Mutex<VMInterface>>) {
    let mut window = init_window();
    let pixels = init_pixels();

    while window.is_open() {
        // Handle events
        while let Some(event) = window.poll_event() {
            match event {
                Event::Closed => window.close(),
                _             => {/* do nothing */}
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