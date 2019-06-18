#![feature(async_await)]

use futures::{StreamExt, future::try_join, FutureExt};
use std::{net::SocketAddr, io};
use tokio::runtime::current_thread;
use tokio_tcp::{TcpListener, TcpStream};

mod split;
mod copy;

use split::split;
use copy::copy;

fn main() -> io::Result<()> {
    let incoming_addr = "127.0.0.1:3556".parse().unwrap();
    let proxy_addr = "172.217.11.46:80".parse().unwrap();

    println!("Listening on: {}", incoming_addr);
    println!("Proxying to: {}", proxy_addr);

    current_thread::block_on_all(proxy(incoming_addr, proxy_addr))
}

async fn proxy(addr: SocketAddr, proxy_addr: SocketAddr) -> io::Result<()> {
    let mut incoming = TcpListener::bind(&addr).unwrap().incoming();

    while let Some(Ok(inbound)) = incoming.next().await {
        let transfer = transfer(inbound, proxy_addr).map(|r| if let Err(e) = r {
            println!("Error: {}", e);
        });
        
        tokio::spawn(transfer);
    }

    Ok(())
}

async fn transfer(inbound: TcpStream, proxy_addr: SocketAddr) -> io::Result<()> {
    let outbound = TcpStream::connect(&proxy_addr).await?;

    let (mut ri, mut wi) = split(inbound);
    let (mut ro, mut wo) = split(outbound);
    
    let client_to_server = copy(&mut ri, &mut wo);
    let server_to_client = copy(&mut ro, &mut wi);

    try_join(client_to_server, server_to_client).await?;

    Ok(())
}

