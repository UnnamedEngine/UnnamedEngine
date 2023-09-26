pub struct Engine {
    running: bool,
    title: String,
}

impl Engine {
    pub fn new(title: String) -> Self {
        Engine {
            running: false,
            title: title
        }
    }

    // Starts the engine
    // This method is the only that should be called from the application
    pub fn start(&mut self, start_f: impl FnOnce(), update_f: impl FnMut(), render_f: impl FnMut()) {
        start_f();

        self.running = true;
        self.run(update_f, render_f);
    }

    // // Stops the engine
    // // Should be called for a graceful shutdown of the engine
    // fn stop(&mut self, stop_f: impl FnOnce()) {
    //     stop_f();
    //     self.running = false;
    // }

    // Starts running the engine
    fn run(&self, mut update_f: impl FnMut(), mut render_f: impl FnMut()) {
        while self.running {
            self.update(&mut update_f);
            self.render(&mut render_f);
        }
    }

    // Logical update that runs at each iteration of the engine
    fn update(&self, mut update_f: impl FnMut()) {
        update_f();
    }

    // Rendering that runs at each iteration of the engine
    fn render(&self, mut render_f: impl FnMut()) {
        render_f();
    }
}
