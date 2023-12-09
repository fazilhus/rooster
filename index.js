async function search(prompt) {
    const results = document.getElementById("results");
    results.innerHTML = "";

    const response = await fetch("/api/search", {
        method: 'POST',
        headers: {'Content-Type': 'text/plain'},
        body: prompt,
    });

    const json = await response.json();
    results.innerHTML = "";

    for ([path, rank] of json) {
        let item = document.createElement("span");
        item.appendChild(document.createTextNode(path));
        item.appendChild(document.createElement("br"));
        results.appendChild(item);
    }
}

let query = document.getElementById("query");
let current_search = Promise.resolve();
query.addEventListener("keypress", (e) => {
    if (e.key == "Enter") {
        current_search.then(() => search(query.value));
    }
});