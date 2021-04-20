document.addEventListener("DOMContentLoaded", async function () {
    getData();
});

async function getData() {
    // send mod name to back-end
    const options = {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
    };
  
    var response = await fetch('/list_api', options);
    let info = await response.json();
    console.log(info);
  
    if (response.status == 200) {
        const html = `${JSON.stringify(info)}`;
        document.body.innerHTML = html;
    }
}