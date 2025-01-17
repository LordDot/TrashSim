use sdl2::{pixels::Color, rect::Rect, render::Canvas, video::Window};

use crate::world::WorldPos;

#[derive(Debug, Clone, Copy)]
pub struct DrawerContext {
    pub scale: u32,
    pub x_center: i32,
    pub y_center: i32,
    pub window_size: (i32, i32),
}

impl DrawerContext {
    pub fn offset(&mut self, x: i32, y: i32) {
        self.x_center += x;
        self.y_center += y;
    }

    pub fn adjust_scale(&mut self, scale_diff: i32) {
        let new_scale = self.scale as i32 + scale_diff;
        if new_scale < 10 {
            self.scale = 10;
        } else if new_scale > 1000 {
            self.scale = 1000
        } else {
            self.scale = new_scale as u32;
        }
    }

    pub fn resize(&mut self, x: i32, y: i32) {
        self.window_size = (x, y);
    }
}

pub struct Drawer {
    canvas: Canvas<Window>,
    pub context: DrawerContext,
}

impl Drawer {
    pub fn new(canvas: Canvas<Window>, x_size: i32, y_size: i32, scale: u32) -> Self {
        return Self {
            canvas,
            context: DrawerContext {
                scale,
                x_center: 0,
                y_center: 0,
                window_size: (x_size, y_size),
            },
        };
    }

    pub fn view_rect(&self) -> (WorldPos, WorldPos) {
        let context = self.context;
        let window_size = context.window_size;
        (
            WorldPos(
                (context.x_center - window_size.0 / 2) / context.scale as i32,
                (-context.y_center + window_size.1 / 2) / context.scale as i32,
            ),
            WorldPos(
                (context.x_center + window_size.0 / 2) / context.scale as i32,
                (-context.y_center - window_size.1 / 2) / context.scale as i32,
            ),
        )
    }

    pub fn draw_rect(
        &mut self,
        pos: WorldPos,
        width: u32,
        height: u32,
        c: Color,
    ) -> Result<(), String> {
        let window_size: (i32, i32) = self.context.window_size;
        let x = pos.0 * self.context.scale as i32 + window_size.0 / 2;
        let y = (-pos.1 - 1) * self.context.scale as i32 + window_size.1 / 2;
        let y = y - ((height - 1) * self.context.scale) as i32;
        self.canvas.set_draw_color(c);
        self.canvas.fill_rect(Rect::new(
            x - self.context.x_center,
            y - self.context.y_center,
            width * self.context.scale,
            height * self.context.scale,
        ))?;
        Ok(())
    }

    pub fn frame_rect(&mut self, pos: WorldPos, width: u32, height: u32) -> Result<(), String> {
        self.frame_rect_color(pos, width, height, Color::RGB(0, 0, 0))
    }

    pub fn frame_rect_color(
        &mut self,
        pos: WorldPos,
        width: u32,
        height: u32,
        c: Color,
    ) -> Result<(), String> {
        let window_size: (i32, i32) = self.context.window_size;
        let x = pos.0 * self.context.scale as i32 + window_size.0 / 2;
        let y = (-pos.1 - 1) * self.context.scale as i32 + window_size.1 / 2;
        let y = y - ((height - 1) * self.context.scale) as i32;
        self.canvas.set_draw_color(c);
        self.canvas.draw_rect(Rect::new(
            x - self.context.x_center,
            y - self.context.y_center,
            width * self.context.scale,
            height * self.context.scale,
        ))?;
        Ok(())
    }

    pub fn clear(&mut self) {
        self.canvas.set_draw_color(Color::RGB(0, 127, 255));
        self.canvas.clear();
    }

    pub fn present(&mut self) {
        self.canvas.present();
    }
}
