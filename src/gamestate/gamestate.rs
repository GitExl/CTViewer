use sdl2::event::Event;
use crate::l10n::L10n;
use crate::software_renderer::surface::Surface;

pub trait GameStateTrait {
    fn tick(&mut self, delta: f64);
    fn render(&mut self, lerp: f64, target_surface: &mut Surface);
    fn get_title(&self, l10n: &L10n) -> String;
    fn event(&mut self, event: &Event);
    fn dump(&mut self);
}
