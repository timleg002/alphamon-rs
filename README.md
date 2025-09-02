# alphamon-rs

Library for monitoring Alpha Outback-type UPSes via a serial interface. Works for Alpha Outback UPSes using the Alphamon software. Based on the official protocol published by Alpha Outback (`protocol/Continuity-Plus-Series-Communication-protocol-en.pdf`).

## Platforms
 
Tested on Windows (x64) and Linux (aarch64 and armv7). Also running in production for these platforms.
 
## Usage
 
Examples are provided in the `examples` folder. The [alphamon-cli-rs](https://github.com/timleg002/alphamon-cli-rs) crate is based on this library and contains a full implementation of all query commands in this library.