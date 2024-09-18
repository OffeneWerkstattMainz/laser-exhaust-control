laser-exhaust-control
=====================

Arduino based laser exhaust control with configurable delayed shutdown. 

Built with Rust, targeting the _Arduino Uno_ or compatible boards.

## Build Instructions
1. Install prerequisites as described in the [`avr-hal` README] (`avr-gcc`, `avr-libc`, `avrdude`, [`ravedude`]).
2. There are two configurable parameters that can be set at build time:
   - `EXHAUST_RUNTIME`: The time in seconds the exhaust should run after the control signal is removed. Defaults to 20 seconds.
   - `INITIAL_IGNORE_PERIOD`: The time in seconds to ignore inputs after power on. This is useful on controllers that produce spurious control output signals during startup. Defaults to 0 seconds.

   These can be set by passing them as environment variables to the `build` and `run` command. For example:
   ```sh
   EXHAUST_RUNTIME=60 INITIAL_IGNORE_PERIOD=10 cargo build
   EXHAUST_RUNTIME=60 INITIAL_IGNORE_PERIOD=10 cargo run
   ```

2. Run `cargo build` to build the firmware.

3. Run `cargo run` to flash the firmware to a connected board.  If `ravedude`
   fails to detect your board, check its documentation at
   <https://crates.io/crates/ravedude>.

4. `ravedude` will open a console session after flashing where you can interact
   with the UART console of your board.

[`avr-hal` README]: https://github.com/Rahix/avr-hal#readme
[`ravedude`]: https://crates.io/crates/ravedude

## Usage

This project assumes a circuit similar to the following:

<picture>
   <source media="(prefers-color-scheme: dark)" srcset="doc/circuit-d.svg"/>
   <source media="(prefers-color-scheme: light)" srcset="doc/circuit-b.svg"/>
   <img alt="Circuit diagram" />
</picture>

([Diagram source](https://crcit.net/c/1c4770f90524432d8ed3a7e588cca58c))

- Pin 2: Control signal input from the laser controller.
   
  The shown circuit assumes a 12V control signal and contains an appropriate voltage divider to bring the signal down to around 4V. The resistor values chosen allow an input voltage range of 10 - 16V
- Pin 7: Relay control output. This assumes an active-low 5V relay.

## License
Licensed under the [MIT license](./LICENSE)
