// imports
use fundsp::hacker::*;
use vst::buffer::AudioBuffer;
use vst::prelude::*;

// define the plugin struct
struct Synthy {
    // look into what this line does O.O
    // dynamic dispatched audio graph
    audio: Box<dyn AudioUnit64 + Send>,
}

// we use fundsup to process the audio buffer

impl Plugin for Synthy {
    fn new(_host: HostCallback) -> Self {
        // create a new audio graph description
        let audio_graph = noise() >> split::<U2>();
        // notice the implicit return here
        
        Self {
            audio: Box::new(audio_graph) as Box<dyn AudioUnit64 + Send>,
        }
    }

    // plugin info
    fn get_info(&self) -> Info {
        Info {
            name: "synthy".into(),
            vendor: "louie".into(),
            unique_id: 183295682,
            category: Category::Synth,
            inputs: 0,
            outputs: 2,
            parameters: 0,
            ..Info::default()
        }
    }

    // modify the audio buffer
    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        let (_, mut outputs) = buffer.split();

        // this is where we will use fundsp to process the audio buffer

        if outputs.len() == 2 {
            let (left, right) = (outputs.get_mut(0), outputs.get_mut(1));

            for (left_chunk, right_chunk) in left
                .chunks_mut(MAX_BUFFER_SIZE)
                .zip(right.chunks_mut(MAX_BUFFER_SIZE))
            {
                let mut right_buffer = [0f64; MAX_BUFFER_SIZE];
                let mut left_buffer = [0f64; MAX_BUFFER_SIZE];

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