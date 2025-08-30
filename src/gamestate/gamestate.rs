use sdl3::event::Event;
use crate::{Context, GameEvent};

pub trait GameStateTrait {
    fn tick(&mut self, ctx: &mut Context, delta: f64) -> Option<GameEvent>;
    fn render(&mut self, ctx: &mut Context, lerp: f64);
    fn get_title(&self, ctx: &Context) -> String;
    fn event(&mut self, ctx: &mut Context, event: &Event);
    fn mouse_motion(&mut self, ctx: &Context, x: i32, y: i32);
    fn dump(&mut self, ctx: &Context);
}
