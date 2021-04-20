document.addEventListener("DOMContentLoaded", async function () {
    let steamid = new URLSearchParams(window.location.search).get('steamid64');

    if (steamid != null) {
        getData(steamid);
    }
});

async function getData(steamid64) {
    // send mod name to back-end
    let str = steamid64
    const data = { str }
    const options = {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify(data)
    };
  
    var response = await fetch('/author_api', options);
    let info = await response.json();
    console.log(info);
  
    if (response.status == 200) {
        const html = `${JSON.stringify(info)}`;
        document.body.innerHTML = html;
    }
}