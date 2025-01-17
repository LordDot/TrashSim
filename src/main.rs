use std::{
    ops::Sub,
    thread::sleep,
    time::{Duration, Instant},
};

use drawer::Drawer;
use log::{error, info, trace};
use player::Player;
use sdl2::event::Event;
use world::{World, WorldPos};

mod drawer;
mod player;
mod world;

fn main() -> Result<(), String> {
    env_logger::init();
    error!("Logger initalized");

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("TrashSim", 800, 600)
        .position_centered()
        .resizable()
        .build()
        .map_err(|e| e.to_string())?;

    let canvas = window
        .into_canvas()
        .software()
        .build()
        .map_err(|e| e.to_string())?;

    let mut events = sdl_context.event_pump()?;
    let mut drawer = Drawer::new(canvas, 800, 600, 50);
    let mut world = World::new();
    let mut player = Player::new(WorldPos(0, 0));

    let mut waited_time = Duration::from_secs(0);
    let mut last_time = Instant::now();
    'main_loop: loop {
        let mouse_button_pressed = events
            .mouse_state()
            .is_mouse_button_pressed(sdl2::mouse::MouseButton::Left);
        for event in events.poll_iter() {
            match event {
                Event::Quit { .. } => break 'main_loop,
                Event::MouseWheel { x, y, .. } => drawer.context.adjust_scale(x + y),
                Event::MouseMotion { xrel, yrel, .. } => {
                    if mouse_button_pressed {
                        drawer.context.offset(-xrel, -yrel)
                    }
                }
                Event::Window { win_event, .. } => match win_event {
                    sdl2::event::WindowEvent::SizeChanged(x, y) => drawer.context.resize(x, y),
                    _ => {}
                },
                _ => {}
            }
        }

        drawer.clear();

        let view_rect = drawer.view_rect();
        world.ensure_rect(view_rect.0, view_rect.1);

        world.draw(&mut drawer)?;
        player.draw(&mut drawer)?;

        drawer.present();

        waited_time = waited_time + last_time.elapsed();
        last_time = Instant::now();
        trace!("Waited: {}", waited_time.as_millis());

        let frame_time: u64 = 200;
        if waited_time.as_millis() > frame_time.into() {
            waited_time = waited_time.sub(Duration::from_millis(frame_time));

            world.update();
            player.next_move(&mut world);
        }
    }

    Ok(())
}
