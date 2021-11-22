use tokio::{io::{AsyncBufReadExt, AsyncWriteExt, BufReader}, net::{TcpListener}, sync::broadcast};

const LOCAL_SERVER: &str = "0.0.0.0:8888";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!(" hello im !");

    let listener = TcpListener::bind(LOCAL_SERVER).await?;

    let (tx, _rx) = broadcast::channel(12);

    loop {
        let (mut socket, addr) = listener.accept().await?;

        println!("{} connected", addr);

        let tx = tx.clone();
        let mut rx = tx.subscribe();

        tokio::spawn(async move {
            let (reader, mut writer) = socket.split();
            let mut reader = BufReader::new(reader);
            let mut msg = String::new();

            loop {
                tokio::select! {
                    result = reader.read_line(&mut msg) => {
                        if result.unwrap() == 0 {
                            break;
                        }

                        println!(" msg: {}", msg);

                        tx.send((msg.clone(), addr)).unwrap();
                        msg.clear();
                    }

                    result = rx.recv() => {
                        let (msg_str, o_addr) = result.unwrap();
                        if addr != o_addr {
                            println!("send {} to {}", &msg_str, o_addr);
                            writer.write_all(msg.as_bytes()).await.unwrap();
                        }
                    }
                }
            }
        });
    }
}