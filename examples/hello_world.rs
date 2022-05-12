use wgpu_sandbox::prelude::*;

#[derive(Debug)]
pub struct Example {}

impl AppInstance for Example {
    fn create(_gpu: &Gpu) -> Self {
        Self {}
    }
}

fn main() {
    AppBuilder::new()
        .with_name("hello_world")
        .with_dimension(1280, 720)
        .build()
        .run::<Example>();
}
