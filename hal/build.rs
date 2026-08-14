fn main() {
    let num_enabled_device_features = std::env::vars()
        .map(|(a, _)| a)
        .filter(|x| x.starts_with("CARGO_FEATURE_MSP430FR"))
        .count();

    match num_enabled_device_features {
        0 => panic!("\x1b[31;1m No device feature enabled. Enable the feature that matches the part number of your device. \x1b[0m"),
        1 => (),
        _ => panic!("\x1b[31;1m Multiple device features enabled. Only use the device feature that matches the part number of your device. \x1b[0m"),
    };
    
    panic!("\x1b[31;1m The `msp430fr2x5x-hal` project has been renamed to `msp430-hal`. This old crate is now deprecated, the last functional release is 0.7.0. \n Please update your cargo.toml to use the new `msp430-hal` crate instead. \x1b[0m");
}
