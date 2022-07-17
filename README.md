# ABOUT
Synthy is a simple FM synthesizer built using Rust. it is cross platform, but only compiles to a VST2 DLL right now.

# INSTRUCTIONS

to use synthy in a daw:
1. run `cargo build` to generate a DLL file in the `/target/release` folder
2. move that generated DLL file into your system VST folder
3. have your DAW re-scan your VST folder
4. voila! you can now use synthy!

to build synthy for a new feature, etc:
1. delete the `/target/release` folder if there is any
2. run `cargo build --release`
3. move the new generated DLL file into your system VST folder
4. have your DAW re-scan your VST folder