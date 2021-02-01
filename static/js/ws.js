let socket = new WrapperWebSocket();
socket.send("腾");

function WrapperWebSocket() {
    if ("WebSocket" in window) {
        let ws = new WebSocket("ws://127.0.0.1:8080/ws/");
        this.waitForConnection = function (callback, interval) {
            if (ws.readyState === 1) {
                callback();
            } else {
                let that = this;
                setTimeout(function (){
                    that.waitForConnection(callback, interval)
                }, interval);
            }
        };
        this.send = function (message, callback) {
            this.waitForConnection(function (){
                ws.send(message);
                if (typeof callback != 'undefined') {
                    callback();
                }
            }, 1000);
        };
        ws.onmessage = function (messageEvent){
            const chat = document.querySelector("#chat");
            chat.innerHTML += messageEvent.data;
            alert(messageEvent.data);
        };
    }
}