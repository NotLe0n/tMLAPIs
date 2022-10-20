# tMLAPIs
tMLAPIs adds multiple apis to get data for tModLoader mods.\
It has support for both tModLoader version 1.3 and 1.4.

This Project is built in rust using the rocket, reqwest, scraper, and once_cell crates and uses the Steam API to get data for 1.4 mods.
Data for 1.3 mods are scraped from http://javid.ddns.net/tModLoader.

## How to Run
**Prerequisites**:
* Install [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)

**Running**:
1. Clone the repository
2. Set the STEAM_API_KEY environment variable to your Steam API key. Go [here](https://steamcommunity.com/dev/apikey) to get one.
3. Run with `cargo run`

## Documentation
For api documentation, see the wiki page.