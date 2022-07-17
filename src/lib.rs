// imports
mod params;
mod editor;

use fundsp::hacker::*;
use num_derive::FromPrimitive;
use params:: {Parameter, Parameters};
use std::{convert::TryFrom, sync::Arc, time::Duration, ops::RangeInclusive};
use vst::prelude::*;
use wmidi::{Note, Velocity};

// define the plugin struct
struct Synthy {
    // look into what this line does O.O
    // dynamic dispatched audio graph
    audio: Box<dyn AudioUnit64 + Send>,
    // add a thread-safe parameters field
    parameters: Arc<Parameters>,
    // store note as an option
    note: Option<(Note, Velocity)>,
    enabled: bool,
    sample_rate: f32,
    time: Duration,
    editor: Option<editor::PluginEditor>,
}

impl Plugin for Synthy {
    // called on init
    fn init(&mut self) {
        // Set up logs, adapted from code from DGriffin91
        // MIT: https://github.com/DGriffin91/egui_baseview_test_vst2/blob/main/LICENSE
        let Info {
            name,
            version,
            unique_id,
            ..
        } = self.get_info();
        let home = dirs::home_dir().unwrap().join("tmp");
        let id_string = format!("{name}-{version}-{unique_id}-log.txt");
        let log_file = std::fs::File::create(home.join(id_string)).unwrap();
        let log_config = ::simplelog::ConfigBuilder::new()
            .set_time_to_local(true)
            .build();
        simplelog::WriteLogger::init(simplelog::LevelFilter::Info, log_config, log_file).ok();
        log_panics::init();
        log::info!("init");
    }

    #[allow(clippy::precendence)]
    fn new(_host: HostCallback) -> Self {
        let Parameters { modulation } = Parameters::default();
        let freq = || tag(Tag::Freq as i64, 440.);
        let modulation = || tag(Tag::Modulation as i64, modulation.get() as f64);
        // generate envelope
        let offset = || tag(Tag::NoteOn as i64, 0.);
        let env = || offset() >> envelope2(|t, offset| downarc((t - offset) * 2.));
        // create a new audio graph description using the env, freq and modulation
        let audio_graph = freq() 
        >> sine() * freq() * modulation() + freq() 
        >> env() * sine() 
        >> declick()
        >> split::<U2>();
        let params: Arc<Parameters> = Arc::new(Default::default());
        // notice the implicit return here
        Self {
            audio: Box::new(audio_graph) as Box<dyn AudioUnit64 + Send>,
            parameters: params.clone(),
            note: None,
            time: Duration::default(),
            sample_rate: 40_000f32,
            enabled: false,
            editor: Some(editor::PluginEditor {
                params,
                window_handle: None,
                is_open: false,
            })
        }
    }

    fn get_editor(&mut self) -> Option<Box<dyn vst::editor::Editor>> {
        if let Some(editor) = self.editor.take() {
            Some(Box::new(editor) as Box<dyn vst::editor::Editor>)
        } else {
            None
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
            parameters: 1,
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

                self.set_tag_with_param(Tag::Modulation, Parameter::Modulation, 0f64..=10f64);

                if let Some((note, ..)) = self.note {
                    self.set_tag(Tag::Freq, note.to_freq_f64())
                }

                if self.enabled {
                    // do timekeeping stuff and process audio buffer
                    self.time += Duration::from_secs_f32(MAX_BUFFER_SIZE as f32 / self.sample_rate);
                    self.audio.process(
                        MAX_BUFFER_SIZE,
                        &[],
                        &mut [&mut left_buffer, &mut right_buffer],
                    );
                }


                for (chunk, output) in left_chunk.iter_mut().zip(left_buffer.iter()) {
                    *chunk = *output as f32;
                }

                for (chunk, output) in right_chunk.iter_mut().zip(right_buffer.iter()) {
                    *chunk = *output as f32;
                }
            }
        }
    }

    // handle midi events
    fn process_events(&mut self, events: &vst::api::Events) {
        for event in events.events() {
            if let vst::event::Event::Midi(midi) = event {
                if let Ok(midi) = wmidi::MidiMessage::try_from(midi.data.as_slice()) {
                    // actually process midi events here
                    match midi {
                        wmidi::MidiMessage::NoteOn(_channel, note, velocity) => {
                            // set NoteOn time tag and enable synth
                            self.set_tag(Tag::NoteOn, self.time.as_secs_f64());
                            self.note = Some((note, velocity));
                            self.enabled = true;
                        }
                        wmidi::MidiMessage::NoteOff(_channel, note, _velocity) => {
                            if let Some((current_note, ..)) = self.note {
                                if current_note == note {
                                    self.note = None;
                                }
                            }
                        },
                        _ => ()
                    }
                }
            }
        }
    }

    // set sample rate
    fn set_sample_rate(&mut self, rate: f32) {
        self.sample_rate = rate;
        self.time = Duration::default();
        self.audio.reset(Some(rate as f64));
    }
}

impl Synthy {
    #[inline(always)]
    fn set_tag(&mut self, tag: Tag, value: f64) {
        self.audio.set(tag as i64, value);
    }

    #[inline(always)]
    fn set_tag_with_param(&mut self, tag: Tag, param: Parameter, range: RangeInclusive<f64>) {
        let value = self.parameters.get_parameter(param as i32) as f64;
        let mapped_value = (value - range.start()) * (range.end() - range.start()) + range.start();
        self.set_tag(tag, mapped_value);
    }
}

// Tag enum
#[derive(FromPrimitive, Clone, Copy)]
pub enum Tag {
    Freq = 0,
    Modulation = 1,
    NoteOn = 2,
}

// actually build the plugin
vst::plugin_main!(Synthy);