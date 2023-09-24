pub struct Application {
    pub running: bool,
    on_start: Option<Box<dyn FnMut()>>,
    on_stop: Option<Box<dyn FnMut()>>,
    on_update: Option<Box<dyn FnMut()>>,
}

impl Application {
    pub fn new() -> Self {
        Application {
            running: false,
            on_start: None,
            on_stop: None,
            on_update: None
        }
    }

    // Starts the engine
    pub fn start(&mut self) {
        // Sets the values
        self.running = true;

        // Calls on_start if it exists
        if let Some(ref mut on_start) = self.on_start {
            on_start();
        }

        // Start the running state
        self.run();
    }

    // Stops the engine
    fn stop(&mut self) {
        // Calls on_stop if it exists
        if let Some(ref mut on_stop) = self.on_stop {
            on_stop();
        }

        self.running = false;
    }

    // Running state
    fn run(&mut self) {
        while self.running {
            self.main_loop();
        }
    }

    // Main loop
    fn main_loop(&mut self) {
        // Calls on_update if it exists
        if let Some(ref mut on_update) = self.on_update {
            on_update();
        }
    }

    pub fn set_on_start(&mut self, on_start: impl FnMut() + 'static) {
        self.on_start = Some(Box::new(on_start));
    }

    pub fn set_on_stop(&mut self, on_stop: impl FnMut() + 'static) {
        self.on_stop = Some(Box::new(on_stop));
    }

    pub fn set_on_update(&mut self, on_update: impl FnMut() + 'static) {
        self.on_update = Some(Box::new(on_update));
    }
}
