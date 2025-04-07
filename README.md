# Synchro
A simulator and real world demonstration of best-effort consistency between sovereign systems designed for Multi-Channel E-Commerce.
"calibration_target": ["L0ZSXYY4THT9D", "J2OR6BTXG7TBVD7HOUV7TXFI"],

## Installation
The project is written in Rust- as of right now you must build it from source.\
You can install Rust through [rustup](rustup.rs).\
<br>
We use nightly features (mutable linked list cursors), so you must switch to the nightly branch.\
`rustup default nightly`\
<br>
From here- you can build the project by cd'ing into the Cargo.toml directory and running:\
`cargo build`\
<br>
The binary will (likely) be located in:\
`target/debug`\
<br>
## Usage
Syncho has both a simulation, and real-world mode. Each is configured differently.\
To run simulations:\
`synchro simulate <simulation_config>`\
You can also supply a directory with many simulation configurations, in which case all will be run sequentially.\
<br>
To run the real-world mode:\
`synchro run <real_world_config>`\
*note* to run the real-world mode, you must have a square developer account- see the attached [start guide](...)\
<br>
## Configuration
### Simulation
Here is an example simulation config- we include many test configs in `/scenarios`
```json
{
  "Simulation": [{
      "until": 100000, // Run Simulation Until (in seconds)
      "initial_value": 100, // Start with value
      "max_divergence_before_error": {
        "secs": 100,
        "nanos": 0
      },
      "platforms": {
        "Polling2": { // Name (String ID)
          "PollingSafe": { // Model (Record, PollingSafe, PollingUnsafe)
            "initial_value": 100,
            "network_params": {
              "size": 20.0, // Network latency average (PER DIRECTION)
              "scale":4.0 // Stability (lower == UNSTABLE)
            },
            "interface_params": {
              "interp": "Transition", // Poll Interpretation
              "backoff": {
                "secs": 0,
                "nanos": 200000000 // Time between polls.
              }
            },
            "user_params": {
              "until": 100000, // Do sales until (in seconds)
              "average_sales_per_hour": 5.0,
              "average_edits_per_day": 1.0,
              "edit_to": 100,
              "start_after": { // Start sales after <deviation backoff>
                "secs": 2,
                "nanos": 0
              }
            }
          }
        },
        ... Further Platforms
      }
    }
  ]}
```

### Real World
Here is an example real-world config. You will need to fill in these details with your own account:
```json
{
  "RealWorld": {
    "initial_value": 100,
    "platforms": [
      ["VendorA", {
        "Records": {
          "token": "...", // OAuth Access Token (Developer Console)
          "backoff": {
            "secs": 0,
            "nanos": 200000000 // Time between queries (Avoid rate limits)
          },
          "target": ["LH1G24AYK9WJT", "52HQGXFQZWQQ7AITOUFWFW3C"], // Location ID, Catalog Object ID - the product to sync
          "calibration_target": ["LH1G24AYK9WJT", "HUXUOSBQC4HS3DDPHEH3RTXP"] // Ditto- sacrificial product for deviation calculations.
        }
      }],
      ["VendorB", {
        "Polling": {
          "token":"...",
          "backoff": {
            "secs": 0,
            "nanos": 200000000
          },
          "target": ["L0ZSXYY4THT9D", "F5FDIG3YCSZQXEXIRRLZLI6M"],
          "interpretation": "..." // How to interpret polls- Transition, Mutation, or Assignment
        }
      }]
    ]
  }
}
```
# Project Structure
The structure of the project is as follows:
- `src/interpreter`: Core logic for the system, history and application.
- `src/simulation`: Simulation-specific models, configuration, etc.
- `src/real_world`: Real-World specific workers, configuration, etc.
- `scenarios`: Various simulation configurations to explore.
- `src`: Start logic, and common logic between all system components.

*Note:* While this branch was started a couple weeks ago- there were many prior experiments and implementations.\
I include these as partial artifacts for reference in the `archive` directory.

