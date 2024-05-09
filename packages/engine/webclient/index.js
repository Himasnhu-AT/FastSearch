async function search(prompt) {
    const results = document.getElementById("results")
    results.innerHTML = "";
    const response = await fetch("/api/search", {
        method: 'POST',
        headers: { 'Content-Type': 'text/plain' },
        body: prompt,
    });
    const json = await response.json();
    results.innerHTML = "";
    for ([path, rank] of json) {
        let item = document.createElement("div");
        item.style = "margin: 5px; padding: 10px;"
        item.appendChild(document.createTextNode(path));
        // item.appendChild(document.createElement("br"));
        results.appendChild(item);
    }
    console.log(results)
    console.warn(response)
}

let query = document.getElementById("query");
let currentSearch = Promise.resolve()

query.addEventListener("keypress", (e) => {
    if (e.key == "Enter") {
        currentSearch.then(() => search(query.value));
    }
})