// setup
const express = require('express');
const path = require('path');
const app = express();
const axios = require('axios');
const cheerio = require('cheerio');

// fixes file paths
app.use(express.static(__dirname));
//enables json use
app.use(express.json());

//get request for the homepage
app.get('/', async (request, response) => {
  await response.sendFile(path.join(process.cwd(), 'index.html'));
});

// listening for requests
app.listen(3000, () => {
  console.log('server started');
});

//
app.post('/author_api', async (request, response) => {
  let id = request.body.str;
  console.log('Got a request: ' + id);

  const url = 'http://javid.ddns.net/tModLoader/tools/ranksbysteamid.php?steamid64=' + id;

  // scrape data
  const mods = await scrapeAuthorData(url)

  console.log(mods);

  // send data to frontend
  response.status(200).send(mods);
});

app.post('/list_api', async (request, response) => {
  console.log('Got a request for a list');
  
  const mods = await scrapeModList()

  // send data to frontend
  response.status(200).send(mods);
});

// returns a json array
async function scrapeAuthorData(url) {
  let mods = [];

  await axios(url)
    .then(site => {
      const html = site.data;
      const $ = cheerio.load(html);

      // find the *first* table
      const table = $('.primary')[0];
      // firstChild is the <tbody>, get its children
      const rows = table.firstChild.children;

      let RankTotal;
      let DisplayName;
      let DownloadsTotal;
      let DownloadsYesterday;

      // go trough all rows and grab the data
      for (let i = 1; i < rows.length; i++) {
        RankTotal = rows[i].children[0].children[0].data;
        DisplayName = rows[i].children[1].children[0].data;
        DownloadsTotal = rows[i].children[2].children[0].data;
        DownloadsYesterday = rows[i].children[3].children[0].data;
      
        // generate json
        mods.push({
          "DisplayName": DisplayName,
          "RankTotal": RankTotal,
          "DownloadsTotal": DownloadsTotal,
          "DownloadsYesterday": DownloadsYesterday
        });
      }
    }).catch(console.error);

  return mods;
}

async function scrapeModList() {
  let mods = [];

  await axios("http://javid.ddns.net/tModLoader/modmigrationprogress.php")
    .then(site => {
      const html = site.data;
      const $ = cheerio.load(html);

      // find the *first* table
      const table = $('.primary')[0];
      // firstChild is the <tbody>, get its children
      const rows = table.firstChild.children;

      let DisplayName;
      let DownloadsTotal;
      let DownloadsYesterday;
      let tModLoaderVersion;
      let InternalName;

      // go trough all rows and grab the data
      for (let i = 1; i < rows.length; i++) {
        DisplayName = rows[i].children[0].children[0].data;
        DownloadsTotal = rows[i].children[1].children[0].data;
        DownloadsYesterday = rows[i].children[2].children[0].data;
        tModLoaderVersion = rows[i].children[3].children[0].data;
        InternalName = rows[i].children[4].children[0].data;

        // generate json
        mods.push({
          "DisplayName": DisplayName,
          "DownloadsTotal": DownloadsTotal,
          "DownloadsYesterday": DownloadsYesterday,
          "tModLoaderVersion": tModLoaderVersion,
          "InternalName": InternalName
        });
      }
    }).catch(console.error);

  return mods;
}

//stuff to do on exit
process.on('exit', function () {
  console.log('About to close');
});