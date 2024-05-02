console.log("Querying ...")

// fetch("/api/search", {
//     method: "POST",
//     headers: {
//         "Content-Type": "application/json"
//     },
//     body: JSON.stringify({
//         "query": "bind texture to buffer"
//     })
// }).then((response) => console.log(response))

fetch("/api/search", {
    method: "POST",
    headers: {
        "Content-Type": "text/plain"
    },
    body: "bind texture to buffer",
}).then((response) => console.log(response))
