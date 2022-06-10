// imports
use rand::Rng;
use std::borrow::BorrowMut;
use vst::prelude::*;

// define the plugin struct
struct Synthy;

impl Plugin for Synthy {
    fn new(_host: HostCallback) -> Self {
        // notice the implicit return here
        Synthy
    }

    // plugin info
    fn get_info(&self) -> Info {
        Info {
            name: "synthy".into(),
            vendor: "rusty".into(),
            unique_id: 183295682,
            category: Category::Synth,
            inputs: 0,
            outputs: 2,
            parameters: 0,
            ..Info::default()
        }
    }

    // modify the ausio buffer
    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        let (_, mut outputs) = buffer.split();

        for output in outputs.borrow_mut() {
            rand::thread_rng().fill(output);
        }
    }
}

// actually build the plugin
vst::plugin_main!(Synthy);