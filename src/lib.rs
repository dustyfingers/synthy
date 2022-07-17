// imports
mod params;

use fundsp::hacker::*;
use params::{Parameter, Parameters};
use std::sync::Arc;
use vst::prelude::*;

const FREQ_SCALAR: f64 = 1000.;

// define the plugin struct
struct Synthy {
    // look into what this line does O.O
    // dynamic dispatched audio graph
    audio: Box<dyn AudioUnit64 + Send>,
    // add a thread-safe parameters field
    parameters: Arc<Parameters>,
}

impl Plugin for Synthy {
    #[allow(clippy::precendence)]
    fn new(_host: HostCallback) -> Self {
        let Parameters { freq, modulation } = Parameters::default();
        let hz = freq.get() as f64 * FREQ_SCALAR;
        let freq = || tag(Parameter::Freq as i64, hz);
        let modulation = || tag(Parameter::Modulation as i64, modulation.get() as f64);
        // create a new audio graph description using the hz, freq and modulation
        let audio_graph = freq() >> sine() * freq() * modulation() + freq() >> sine() >> split::<U2>();
        // notice the implicit return here
        Self {
            audio: Box::new(audio_graph) as Box<dyn AudioUnit64 + Send>,
            parameters: Default::default(),
        }
    }

    // plugin info - this struct is how the daw gets info about our plugin
    fn get_info(&self) -> Info {
        Info {
            name: "synthy".into(),
            vendor: "louie".into(),
            unique_id: 183295682,
            category: Category::Synth,
            inputs: 0,
            outputs: 2,
            parameters: 2,
            ..Info::default()
        }
    }

    // housekeeping
    fn get_parameter_object(&mut self) -> Arc<dyn PluginParameters> {
        Arc::clone(&self.parameters) as Arc<dyn PluginParameters>
    }

    // modify the audio buffer
    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        let (_, mut outputs) = buffer.split();

        // this is where we will use fundsp to process the audio buffer
        // this is where we are actually modifying the audio graph
        if outputs.len() == 2 {
            let (left, right) = (outputs.get_mut(0), outputs.get_mut(1));

            for (left_chunk, right_chunk) in left
                .chunks_mut(MAX_BUFFER_SIZE)
                .zip(right.chunks_mut(MAX_BUFFER_SIZE))
            {
                let mut right_buffer = [0f64; MAX_BUFFER_SIZE];
                let mut left_buffer = [0f64; MAX_BUFFER_SIZE];

                self.audio.set(
                    Parameter::Modulation as i64,
                    self.parameters.get_parameter(Parameter::Modulation as i32) as f64,
                );

                self.audio.set(
                    Parameter::Freq as i64,
                    self.parameters.get_parameter(Parameter::Freq as i32) as f64 * FREQ_SCALAR,
                );

                self.audio.process(
                    MAX_BUFFER_SIZE,
                    &[],
                    &mut [&mut left_buffer, &mut right_buffer],
                );

                for (chunk, output) in left_chunk.iter_mut().zip(left_buffer.iter()) {
                    *chunk = *output as f32;
                }

                for (chunk, output) in right_chunk.iter_mut().zip(right_buffer.iter()) {
                    *chunk = *output as f32;
                }
            }
        }
    }
}

// actually build the plugin
vst::plugin_main!(Synthy);