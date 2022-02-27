
(async function() {
  const webSocket = new WebSocket('ws://localhost:8080');

  webSocket.onopen = function (event) {
    webSocket.send("Hello!");
  };

  webSocket.onmessage = function (event) {
    console.log(event);
  }
})()