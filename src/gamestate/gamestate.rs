use sdl3::event::Event;
use crate::l10n::L10n;
use crate::renderer::Renderer;

pub trait GameStateTrait {
    fn tick(&mut self, delta: f64);
    fn render(&mut self, lerp: f64, renderer: &mut Renderer);
    fn get_title(&self, l10n: &L10n) -> String;
    fn event(&mut self, event: &Event);
    fn mouse_motion(&mut self, x: i32, y: i32);
    fn dump(&mut self);
}
