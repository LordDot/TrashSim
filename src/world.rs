use std::collections::HashMap;

use sdl2::pixels::Color;

use crate::drawer::Drawer;

#[derive(Clone, Copy, Debug)]
pub struct WorldPos(pub i32, pub i32);
impl WorldPos {
    pub fn add_x(&self, i: i32) -> WorldPos {
        WorldPos(self.0 + i, self.1)
    }
    pub fn add_y(&self, i: i32) -> WorldPos {
        WorldPos(self.0, self.1 + i)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
struct ChunkPos(i32, i32);
impl From<WorldPos> for ChunkPos {
    fn from(value: WorldPos) -> Self {
        Self(negative_int_div(value.0, 16), negative_int_div(value.1, 16))
    }
}

fn negative_int_div(l: i32, r: i32) -> i32 {
    if l >= 0 {
        l / r
    } else {
        -((-l - 1) / r) - 1
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct InChunkPos(i32, i32);
impl From<WorldPos> for InChunkPos {
    fn from(value: WorldPos) -> Self {
        Self(negative_mod(value.0, 16), negative_mod(value.1, 16))
    }
}

fn negative_mod(l: i32, r: i32) -> i32 {
    if l >= 0 {
        l % r
    } else {
        (r + (l % r)) % r
    }
}

pub struct World {
    chunks: HashMap<ChunkPos, Chunk>,
}

impl World {
    pub fn new() -> Self {
        let mut world = Self {
            chunks: HashMap::new(),
        };
        world.ensure_block(WorldPos(0, 0));
        world.ensure_block(WorldPos(0, -1));
        world.ensure_block(WorldPos(-1, 0));
        world.ensure_block(WorldPos(-1, -1));

        for y in 0..6 {
            for x in 0..5 + y {
                world.set_block(WorldPos(x, y), Block::Air);
                world.set_block(WorldPos(-x - 1, y), Block::Air);
            }
        }

        world
    }

    pub fn draw(&self, drawer: &mut Drawer) -> Result<(), String> {
        for (&pos, chunk) in self.chunks.iter() {
            chunk.draw(drawer, pos)?;
        }
        Ok(())
    }

    pub fn update(&mut self) {
        let updates: Vec<_> = self
            .chunks
            .iter()
            .flat_map(|(&pos, chunk)| chunk.update(pos, self))
            .collect();
        for u in updates {
            match u {
                UpdateCommand::Move(from, to) => {
                    let from_block = self.force_block(from);
                    self.set_block(to, from_block);
                    self.set_block(from, Block::Air);
                    self.ensure_around(from);
                    self.ensure_around(to);
                }
            }
        }
    }

    fn ensure_around(&mut self, pos: WorldPos) {
        if pos.0 == 0 {
            self.ensure_block(pos.add_x(-1));
        }
        if pos.0 == 15 {
            self.ensure_block(pos.add_x(1));
        }
    }

    pub fn ensure_rect(&mut self, top_right: WorldPos, bottom_left: WorldPos) {
        let top_right_chunk: ChunkPos = top_right.into();
        let bottom_left_chunk: ChunkPos = bottom_left.into();

        for x in top_right_chunk.0..=bottom_left_chunk.0 {
            for y in bottom_left_chunk.1..=top_right_chunk.1 {
                self.ensure_chunk(ChunkPos(x, y));
            }
        }
    }

    pub fn ensure_block(&mut self, pos: WorldPos) {
        self.ensure_chunk(pos.into());
    }

    fn ensure_chunk(&mut self, pos: ChunkPos) {
        if self.chunks.contains_key(&pos) {
            return;
        }
        let chunk = Chunk::generate(pos);
        self.chunks.insert(pos, chunk);
    }

    pub fn set_block(&mut self, pos: WorldPos, block: Block) {
        self.ensure_block(pos);
        let chunk = self
            .chunks
            .get_mut(&pos.into())
            .expect("We just generated that");
        chunk.set_block(pos.into(), block);
    }

    pub fn get_block(&self, pos: WorldPos) -> Option<Block> {
        self.chunks
            .get(&pos.into())
            .map(|c| c.get_block(pos.into()))
    }

    pub fn force_block(&mut self, pos: WorldPos) -> Block {
        self.ensure_block(pos);
        self.get_block(pos).expect("We just generated that?")
    }

    pub fn get_highest_block(&self, x: i32) -> WorldPos {
        let chunk_x = negative_int_div(x, 16);
        let in_chunk_x = negative_mod(x, 16);
        let mut candidates: Vec<_> = self.chunks.iter().filter(|(k, _)| k.0 == chunk_x).collect();
        candidates.sort_by_key(|(&k, _)| -k.1);
        let y = candidates
            .iter()
            .find_map(|(k, chunk)| {
                (0..16)
                    .rev()
                    .find_map(|y| {
                        if chunk.get_block(InChunkPos(in_chunk_x, y)) != Block::Air {
                            Some(y)
                        } else {
                            None
                        }
                    })
                    .map(|in_chunk_y| k.1 * 16 + in_chunk_y)
            })
            .unwrap_or(5);
        WorldPos(x, y)
    }
}

struct Chunk {
    blocks: [Block; 16 * 16],
}

impl Chunk {
    pub fn generate(pos: ChunkPos) -> Self {
        let mut blocks = [Block::Air; 16 * 16];
        for x in 0..16 {
            for y in 0..16 {
                let world_y = Self::world_pos(pos, InChunkPos(x, y)).1;
                let block = if world_y < 0 {
                    Block::Ground
                } else if world_y < 6 {
                    Block::Trash
                } else {
                    Block::Air
                };
                blocks[(x * 16 + y) as usize] = block;
            }
        }
        Self { blocks }
    }

    fn world_pos(chunk: ChunkPos, pos: InChunkPos) -> WorldPos {
        WorldPos(chunk.0 * 16 + pos.0, chunk.1 * 16 + pos.1)
    }

    pub fn set_block(&mut self, pos: InChunkPos, block: Block) -> () {
        self.blocks[(pos.0 * 16 + pos.1) as usize] = block;
    }

    pub fn get_block(&self, pos: InChunkPos) -> Block {
        self.blocks[(pos.0 * 16 + pos.1) as usize]
    }

    pub fn draw(&self, drawer: &mut Drawer, pos: ChunkPos) -> Result<(), String> {
        for x in 0..16 {
            for y in 0..16 {
                let in_chunk = InChunkPos(x, y);
                self.get_block(in_chunk)
                    .draw(drawer, Self::world_pos(pos, in_chunk))?;
            }
        }
        // drawer.frame_rect_color(
        //     Self::world_pos(pos, InChunkPos(0, 0)),
        //     16,
        //     16,
        //     Color::RGB(255, 0, 0),
        // )?;
        Ok(())
    }

    pub fn update(&self, pos: ChunkPos, world: &World) -> Vec<UpdateCommand> {
        let mut commands = vec![];
        for y in (0..16).rev() {
            for x in 0..16 {
                let this = Self::world_pos(pos, InChunkPos(x, y));
                if world.get_block(this).unwrap() != Block::Trash {
                    continue;
                }
                let bottom_left = this.add_x(-1).add_y(-1);
                if world.get_block(bottom_left).unwrap_or(Block::Ground) == Block::Air {
                    commands.push(UpdateCommand::Move(this, bottom_left));
                }
                let bottom_right = this.add_x(1).add_y(-1);
                if world.get_block(bottom_right).unwrap_or(Block::Ground) == Block::Air {
                    commands.push(UpdateCommand::Move(this, bottom_right));
                }
            }
        }
        commands
    }
}

pub enum UpdateCommand {
    Move(WorldPos, WorldPos),
}

#[derive(Clone, Copy, PartialEq)]
pub enum Block {
    Air,
    Ground,
    Trash,
    Compacted,
}

impl Block {
    fn draw(self, drawer: &mut Drawer, pos: WorldPos) -> Result<(), String> {
        if let Block::Air = self {
            return Ok(());
        }

        let color = match self {
            Block::Air => panic!("How did we get here?"),
            Block::Ground => Color::RGB(30, 15, 0),
            Block::Trash => Color::RGB(80, 40, 0),
            Block::Compacted => Color::RGB(60, 30, 0),
        };
        drawer.draw_rect(pos, 1, 1, color)?;
        drawer.frame_rect(pos, 1, 1)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_to_chunk() {
        let data = vec![
            (WorldPos(0, 0), ChunkPos(0, 0)),
            (WorldPos(15, 15), ChunkPos(0, 0)),
            (WorldPos(16, 16), ChunkPos(1, 1)),
            (WorldPos(-1, -1), ChunkPos(-1, -1)),
            (WorldPos(-16, -16), ChunkPos(-1, -1)),
            (WorldPos(-17, -17), ChunkPos(-2, -2)),
        ];
        for (input, expected) in data {
            let chunk: ChunkPos = input.into();
            assert_eq!(chunk, expected);
        }
    }

    #[test]
    fn world_to_in_chunk() {
        let data = vec![
            (WorldPos(0, 0), InChunkPos(0, 0)),
            (WorldPos(15, 15), InChunkPos(15, 15)),
            (WorldPos(16, 16), InChunkPos(0, 0)),
            (WorldPos(-1, -1), InChunkPos(15, 15)),
            (WorldPos(-16, -16), InChunkPos(0, 0)),
            (WorldPos(-17, -17), InChunkPos(15, 15)),
        ];
        for (input, expected) in data {
            let chunk: InChunkPos = input.into();
            assert_eq!(chunk, expected);
        }
    }
}
