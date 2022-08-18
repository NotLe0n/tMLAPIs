# TMLAPIS

# OUTDATED README! QUERIES AND RESPONSES HAVE CHANGED! NAVIGATE THE MENU IN ROOT ('http://127.0.0.1:8000/')

TMLAPIS provides useful tModLoader json apis for making dynamic websites. This API was made by [@NotLe0n](https://github.com/NotLe0n) and [@bafto](https://github.com/bafto)

## Usage
## Mod Info API
Making a GET request on `https://tmlapis.repl.co/modInfo?modname=<internal mod name>` you recieve 
- DisplayName
- InternalName
- Author
- Homepage
- Description
- Icon
- Version
- TModLoaderVersion
- LastUpdated
- ModDependencies
- ModSide
- DownloadLink
- DownloadsTotal
- DownloadsYesterday

Example nodejs code:
```js
const fetch = require('node-fetch');

let url = "https://tmlapis.repl.co/modInfo?modname=BetterChests";

let settings = { method: "Get" };

fetch(url, settings)
    .then(res => res.json())
    .then((json) => {
        console.log(json);
    });
```

## Author API

Making a GET request on `https://tmlapis.repl.co/author_api/<steam64id>` you recieve 
- the steam name of the user
- the Total Downloads Combined
- Yesterdays Total Downloads Combined
- a list of all mods the given user has made.

Example nodejs code:
```js
const fetch = require('node-fetch');

let url = "https://tmlapis.repl.co/author_api/76561198278789341";

let settings = { method: "Get" };

fetch(url, settings)
    .then(res => res.json())
    .then((json) => {
        console.log(json);
    });
```

## Mod List API

Making a GET request on `https://tmlapis.repl.co/modList` you recieve a list of all mods that exist in tML.

Example nodejs code:
```js
const fetch = require('node-fetch');

let url = "https://tmlapis.repl.co/modList";

let settings = { method: "Get" };

fetch(url, settings)
    .then(res => res.json())
    .then((json) => {
        console.log(json);
    });
```

## Item Image API

You can get the image of any Item with this url: `https://tmlapis.repl.co/img/Item_<item id>.png`.<br>
Items which have a sprite sheet are cut down to a singular frame of the animation.
