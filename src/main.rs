// TODO: Deconnexion
// TODO: Prefix on client sending message

use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, AsyncReadExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::task;
use tokio::sync::Mutex;
use std::sync::Arc;
use std::collections::HashMap;
use std::str::from_utf8;

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut listener = TcpListener::bind("127.0.0.1:3000").await?;
    let client_send: Arc<Mutex<HashMap<Arc<u32>, Arc<Mutex<io::WriteHalf<TcpStream>>>>>> = Arc::new(Mutex::new(HashMap::new()));
    let mut last_id: u32 = 0;

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("Accepted connection from: {:?}", addr);
        let client_send = client_send.clone();
        let id = Arc::new(last_id);
        last_id += 1;
        task::spawn(async move {
            let mut name: &mut [u8] = &mut [0;10];
            println!("{:?}", id);
            let (mut rcv, mut send) = io::split(socket);
            
            send.write_all(b"Your name: ").await;
            let bytes = rcv.read(&mut name).await;
            let name = &name[..bytes.unwrap()-1];
            let name = from_utf8(&name).unwrap();
            let msg = format!("{:?} has joined the server\n", name);
            println!("{}", msg);
            for (key, value) in client_send.lock().await.iter() {
                if *key != id {
                    if let Err(e) = value.clone().lock().await.write_all(msg.as_bytes()).await {
                        eprintln!("{:?}", e);
                    }
                }
                
            }
            client_send.lock().await.insert(id.clone(), Arc::new(Mutex::new(send)));
            // if let Some(value) = client_send.lock().await.get(&id) {
            //     if let Err(e) = value.clone().lock().await.write_all(b"Your message: ").await {
            //         panic!("{:?}", e);
            //     };
            // }
            let buf_reader = io::BufReader::new(rcv);
            let mut lines = buf_reader.lines();

            while let Ok(result) = lines.next_line().await {
                match result {
                    Some(mut line) => {
                        println!("Received line from {:?}: {:?}", name, line);
                        line.push_str("\n");
                        let mut msg = format!("{} said: ", name);
                        msg.push_str(&line);
                        for (key, value) in client_send.lock().await.iter() {
                            if *key != id {
                                if let Err(e) = value.clone().lock().await.write_all(msg.as_bytes()).await {
                                    eprintln!("{:?}", e);
                                }
                            }
                            
                        }
                        // if let Some(value) = client_send.lock().await.get(&id) {
                        //     if let Err(e) = value.clone().lock().await.write_all(b"Your message: ").await {
                        //         panic!("{:?}", e);
                        //     };
                        // }
                    },
                    None => {}
                }
            }
        });
    }
}