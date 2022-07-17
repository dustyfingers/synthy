use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::fmt::Display;
use vst::{plugin::PluginParameters, util::AtomicFloat};

// create the 'Parameters' struct
pub struct Parameters {
    pub modulation: AtomicFloat,
}

impl Default for Parameters {
    // set defaults
    fn default() -> Self {
        Self {
            modulation: AtomicFloat::new(1.),
        }
    }
}

impl PluginParameters for Parameters {
    // requred get and set methods
    fn get_parameter(&self, index: i32) -> f32 {
        match FromPrimitive::from_i32(index) {
            Some(Parameter::Modulation) => self.modulation.get(),
            _ => 0f32,
        }
    }

    fn set_parameter(&self, index: i32, value: f32) {
        match FromPrimitive::from_i32(index) {
            Some(Parameter::Modulation) => self.modulation.set(value),
            _ => (),
        }
    }

    fn get_parameter_name(&self, index: i32) -> String {
        let param: Option<Parameter> = FromPrimitive::from_i32(index);
        param
            .map(|f| f.to_string())
            .unwrap_or_else(|| "unknown".to_string())
    }
}

// tagged enum - to replace magic numbers
#[derive(FromPrimitive, Clone, Copy)]
pub enum Parameter {
    Modulation = 0,
}

// display to make things look nice
impl Display for Parameter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Parameter::Modulation => "modulation",
            }
        )
    }
}