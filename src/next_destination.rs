use crate::destination::Destination;

pub struct NextDestination {
    pub destination: Option<Destination>,
    pub fade_in: bool,
}

impl NextDestination {
    pub fn new() -> NextDestination {
        NextDestination {
            destination: None,
            fade_in: true,
        }
    }

    pub fn set(&mut self, destination: Destination, fade_in: bool) {
        if self.destination.is_some() {
            return;
        }
        self.destination = Some(destination);
        self.fade_in = fade_in;
    }

    pub fn clear(&mut self) {
        self.destination = None;
    }
}
