# tMLAPIs
[![build and release](https://github.com/NotLe0n/tMLAPIs/actions/workflows/build-and-release.yml/badge.svg)](https://github.com/NotLe0n/tMLAPIs/actions/workflows/build-and-release.yml)

tMLAPIs adds multiple apis to get data for tModLoader mods.\
It has support for both tModLoader version 1.3 and 1.4.

This Project is built in rust using the rocket, reqwest and scraper crates.
Data for 1.3 mods are scraped from http://javid.ddns.net/tModLoader and data for 1.4 mods is reqwested from the Steam API.

## How to use
You can host tMLAPIs yourself by compiling this repo and running it on your own machine. How to build this repo is explained [in the next section](#how-to-build).
Or you can use the API using the mirror: https://tmlapis.le0n.dev/

## How to build
### Prerequisites:
* Install [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)

### Running
**Bare metal:**
1. Clone the repository
2. Set the STEAM_API_KEY environment variable to your Steam API key. Go [here](https://steamcommunity.com/dev/apikey) to get one.
3. Run with `cargo run --release`

**Docker:**
1. Clone the repository
2. Build the image: `docker build -t tmlapis .`
3. Run the image with the STEAM_API_KEY environment variable included:
```
docker run --name tmlapis \
	-e STEAM_API_KEY=**** \
	-dp 127.0.0.1:8000:8000 \
	tmlapis
```

## Documentation
For api documentation, see the [wiki](https://github.com/NotLe0n/tMLAPIs/wiki) page.
