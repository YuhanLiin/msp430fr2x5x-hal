# Change Log

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/)
and this project adheres to [Semantic Versioning](http://semver.org/).

## [v0.5.0] - 2025-XX-XX

- Add implementations for embedded-hal 1.0 traits (including embedded-io and embedded-hal-nb).
- Expose functionality from the traits dropped between eh-0.2.7 and eh-1.0 (ADC, timers, RTC, watchdog, etc.) as methods on structs instead
- Gate embedded-hal 0.2.7 implementations behind `embedded-hal-02` feature.
- Add additional generic delay implementations for eh-0.2.7 DelayMs trait (u8, u32, i32)
- Derive `Debug` and `Copy` for `RecvError`
- Ensure crate builds successfully back to `nightly-2022-03-01`
- Rename I2C, SPI, Delay structs to avoid confusion with new embedded-hal trait names.
- Support for delays when using sub-1MHz clock sources (e.g. ACLK, VLOCLK).
- Replace public references to `void::Void` with `core::convert::Infallible`
- Add methods to enable the internal voltage reference and temperature sensor

## [v0.4.1] - 2025-01-25

- Fix doc.rs build issue

## [v0.4.0] - 2025-01-22

- Add support for ADC interface
- Add support for SPI interface
- Add support for I2C interface
- Add `Delay` object, allowing for millisecond delays
- Change `ClockConfig::freeze` to return `Delay` in addition to its other return values
- Mitigate issue of device hanging after clock configuration by adding NOPs

## [v0.3.3] - 2022-12-24

- Bump `msp430fr2355` to v0.5.2 to ensure atomic PAC operations are single-instruction

## [v0.3.2] - 2022-10-26

- Bump `msp430fr2355` to v0.5.1
- Fix documentation generation on doc.rs

## [v0.3.1] - 2022-10-26

- Remove erroneous line in docs

## [v0.3.0] - 2022-10-26

- Bump `msp430fr2355` to v0.5
- Add CI pipeline
- Update dependencies to latest ecosystem versions
