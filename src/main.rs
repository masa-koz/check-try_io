use std::time::Duration;

#[cfg(target_family = "unix")]
use std::os::unix::io::AsRawFd;
#[cfg(target_family = "windows")]
use std::os::windows::prelude::AsRawSocket;

#[tokio::main]
async fn main() {
    let task_recv = tokio::spawn(async move {
        let udp = tokio::net::UdpSocket::bind("0.0.0.0:3456").await.unwrap();
        let mut buf = vec![0; 1500];

        loop {
            let _ = udp.readable().await;
            let res = udp.try_io(tokio::io::Interest::READABLE, || {
                let res = unsafe {
                    #[cfg(target_family = "unix")]
                    let sock = udp.as_raw_fd();
                    #[cfg(target_family = "windows")]
                    let sock =  udp.as_raw_socket();

                    libc::recv(
                        sock,
                        buf.as_mut_ptr() as *mut libc::c_void,
                        buf.len(),
                        0
                    )
                };
                if res < 0 {
                    Err(std::io::Error::last_os_error())
                } else {
                    Ok(res as usize)
                }
            });
            println!("res: {:?}", res);
        }
    });
    let task_send = tokio::spawn(async move {
        let udp = tokio::net::UdpSocket::bind("0.0.0.0:0").await.unwrap();
        loop {
            udp.send_to(&[0; 10], "127.0.0.1:3456")
                .await
                .expect("couldn't send data");
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });

    let _ = tokio::join!(task_recv, task_send);
}
