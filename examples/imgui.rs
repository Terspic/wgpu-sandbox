use std::time::Duration;
use wgpu_sandbox::prelude::*;

#[derive(Debug)]
pub struct Example {}

impl AppInstance for Example {
    fn create(_gpu: &Gpu) -> Self {
        Self {}
    }

    fn on_imgui(&mut self, ui: &imgui::Ui, _gpu: &Gpu, _dt: Duration) {
        ui.show_demo_window(&mut true);
    }
}

fn main() {
    AppBuilder::new()
        .with_name("hello_world")
        .with_dimension(1280, 720)
        .build()
        .run::<Example>();
}
