use std::time::SystemTime;


pub fn inrange(elem: isize) -> bool {
    if (elem < 0) || (elem >= 8){
        return false;
    }
    return true;
}



#[derive(Clone)]
pub struct Timer {
    pub duration: u128,
    pub runtime: u128,
    pub start_time: SystemTime,
    pub percent_done: f32,
    pub active: bool,
    pub finished: bool
}

impl Timer {
    pub fn new(duration: u128) -> Self {
        Self {
            duration: duration,
            runtime: 0,
            start_time: SystemTime::now(),
            percent_done: 0.0,
            active: false,
            finished: false
        }
    }
    pub fn reset(&mut self) {
        self.deactivate();
        self.finished = false;
        self.start_time = SystemTime::now();
        self.runtime = 0;
    }
    pub fn activate(&mut self) {
        self.active = true;
        self.finished = false;
        self.percent_done = 0.0;
        self.start_time = SystemTime::now();
    }
    pub fn deactivate(&mut self) {
        self.active = false;
        self.start_time = SystemTime::now();
    }
    pub fn update(&mut self) {
        let current_time = SystemTime::now();
        self.runtime = current_time.duration_since(self.start_time).unwrap().as_millis();
        self.percent_done = self.runtime as f32 / self.duration as f32;
        if self.runtime >= self.duration {
            self.finished = true;
            self.deactivate();
        }
    }
}