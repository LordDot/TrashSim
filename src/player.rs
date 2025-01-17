use log::{debug, info};
use sdl2::pixels::Color;

use crate::{
    drawer::Drawer,
    world::{Block, World, WorldPos},
};

pub struct Player {
    pos: WorldPos,
    load: i32,
    current_target: Option<WorldPos>,
}

impl Player {
    pub fn new(pos: WorldPos) -> Self {
        Player {
            pos,
            load: 0,
            current_target: None,
        }
    }

    pub fn draw(&self, drawer: &mut Drawer) -> Result<(), String> {
        drawer.draw_rect(self.pos, 1, 1, Color::RGB(199, 116, 8))?;
        Ok(())
    }

    pub fn next_move(&mut self, world: &mut World) {
        if self.current_target.is_none() {
            if self.load < 2 {
                if world.force_block(self.pos.add_y(-1)) == Block::Trash {
                    info!("Load");
                    self.load(world);
                } else {
                    info!("Find trash");
                    self.current_target = Some(self.find_nearest_trash(world));
                }
            } else {
                let unload_target = self.find_unload_target(world);
                if self.pos.0 == unload_target.0 {
                    info!("Unload");
                    self.unload(world);
                } else {
                    info!("Move to tower");
                    self.current_target = Some(unload_target);
                }
            }
        }
        self.move_towards_target(world);
    }

    fn move_towards_target(&mut self, world: &World) {
        if let Some(target) = self.current_target {
            info!(
                "Go to ({},{}) from ({}, {})...",
                self.current_target.unwrap().0,
                self.current_target.unwrap().1,
                self.pos.0,
                self.pos.1
            );
            let inc = (target.0 - self.pos.0).signum();
            if inc == 0 {
                self.current_target = None;
                return;
            }
            let target = world.get_highest_block(self.pos.0 + inc).add_y(1);
            info!("Next setp ({},{})...", target.0, target.1);
            if target.1 == self.pos.1 {
                info!("Walk");
                self.pos = target;
            } else if target.1 < self.pos.1 {
                if world.get_highest_block(self.pos.0).1 == self.pos.1 - 1 {
                    info!("Climb down");
                    self.pos = self.pos.add_x(inc).add_y(-1);
                } else {
                    info!("Falling");
                    self.pos = self.pos.add_y(-1);
                }
            } else if target.1 == self.pos.1 + 1 {
                info!("Done Climbing up");
                self.pos = self.pos.add_x(inc).add_y(1);
            } else {
                info!("Climb up");
                self.pos = self.pos.add_y(1);
            }

            let inc = (target.0 - self.pos.0).signum();
            if inc == 0 {
                self.current_target = None;
                return;
            }
        }
    }

    fn load(&mut self, world: &mut World) {
        world.set_block(self.pos.add_y(-1), Block::Air);
        self.pos = self.pos.add_y(-1);
        self.load += 1;
    }

    fn unload(&mut self, world: &mut World) {
        world.set_block(self.pos, Block::Compacted);
        self.pos = self.pos.add_y(1);
        self.load = 0;
    }

    fn find_unload_target(&self, world: &mut World) -> WorldPos {
        let inc = -self.pos.0.signum();
        if inc == 0 {
            return self.pos.add_y(-1);
        }

        let mut current_height = self.pos.1 - 1;
        for i in 0.. {
            let new_x = self.pos.0 + i * inc;
            let new_height = world.get_highest_block(new_x).1;
            debug!("new: {}, current: {}", new_height, current_height);
            if new_height - current_height > 3 {
                return WorldPos(new_x - inc, current_height);
            }
            if new_x == 0 {
                return WorldPos(0, new_height);
            }
            current_height = new_height;
        }
        panic!("We broke out of infinite loop?")
    }

    fn find_nearest_trash(&self, world: &mut World) -> WorldPos {
        for i in 1.. {
            let right_block = world.get_highest_block(self.pos.0 + i);
            if world.force_block(right_block) == Block::Trash {
                return right_block;
            }
            let left_block = world.get_highest_block(self.pos.0 - i);
            if world.force_block(left_block) == Block::Trash {
                return left_block;
            }
        }
        panic!("WTF? The world should be infinite...")
    }
}
