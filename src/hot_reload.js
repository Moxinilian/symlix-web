(function () {
    const socket = new WebSocket('ws://{{ addr }}');
    setInterval(() => socket.send('?'), 5000);

    socket.addEventListener('message', (event) => {
        if (event.data == '!') {
            location.reload();
        }
    });
})();
