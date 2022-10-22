# tMLAPIs
tMLAPIs adds multiple apis to get data for tModLoader mods.\
It has support for both tModLoader version 1.3 and 1.4.

This Project is built in rust using the rocket, reqwest, scraper, and once_cell crates and uses the Steam API to get data for 1.4 mods.
Data for 1.3 mods are scraped from http://javid.ddns.net/tModLoader.

## How to use
You can host tMLAPIs yourself by compiling this repo yourself and running it on your own machine. How to build this repo is explained [in the next section](#how-to-build).
Or you can use the API using these mirrors:
* https://tmlapis.repl.co/
* https://tmlapi.tomat.dev/

## How to build
**Prerequisites**:
* Install [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)

**Running**:
1. Clone the repository
2. Set the STEAM_API_KEY environment variable to your Steam API key. Go [here](https://steamcommunity.com/dev/apikey) to get one.
3. Run with `cargo run --release`

## Documentation
For api documentation, see the [wiki](https://github.com/NotLe0n/tMLAPIs/wiki) page.