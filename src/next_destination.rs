use crate::destination::Destination;

pub struct NextDestination {
    pub destination: Option<Destination>,
    pub fade_out: bool,
}

impl NextDestination {
    pub fn new() -> NextDestination {
        NextDestination {
            destination: None,
            fade_out: true,
        }
    }

    pub fn set(&mut self, destination: Destination, fade_out: bool) {
        if self.destination.is_some() {
            return;
        }
        self.destination = Some(destination);
        self.fade_out = fade_out;
    }

    pub fn clear(&mut self) {
        self.destination = None;
    }
}
