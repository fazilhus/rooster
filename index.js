console.log("Querying /api/search")

fetch("/api/search", {
    method: 'POST',
    headers: {'Content-Type': 'text/plain'},
    body: "bind texture to buffer opengl 4",
}).then((response) => console.log(response))