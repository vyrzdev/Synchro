# Synchro
A simulator and real world demonstration of best-effort consistency between sovereign systems designed for Multi-Channel E-Commerce.

## Installation
The project is written in Rust- as of right now you must build it from source.\
You can install rust using [rustup](rustup.rs)\
<br>
We use nightly features (mutable linked list cursors), so you must switch to the nightly branch.\
`rustup default nightly`\
<br>
From here- you can build the project by cd'ing into the Cargo.toml directory and running:\
`cargo build`\
<br>
The binary will (likely) be located in:\
`target/debug`\

## Usage
Syncho has both a simulation, and real-world mode. Each is configured differently.
To run simulations:
`synchro simulate <simulation_config>`
You can also supply a directory with many simulation configurations, in which case all will be run sequentially.

To run the real-world mode:
`synchro run <real_world_config>`
*note* to run the real-world mode, you must have a square developer account- see the attached [start guide](...)
