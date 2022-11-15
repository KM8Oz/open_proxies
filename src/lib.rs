#![allow(dead_code)]
use async_std::future;
use async_std::io::{ReadExt, WriteExt};
use futures::channel::oneshot;
use futures::{stream, StreamExt};
use httparse::{Response, EMPTY_HEADER};
use rayon::prelude::*;
use regex::Regex;
use std::env;
use std::fs::File;
use std::io::{self, BufRead, Read, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::task::JoinHandle;
#[derive(Default, Clone, PartialEq, Debug)]
pub enum Proto {
    #[default]
    HTTP,
    HTTPS,
    SOCKS4,
    SOCKS5,
    UNKNOWN,
}
impl Proto {
    fn to_string(&self) -> String {
        format!("{:?}", self).to_lowercase()
    }
}
pub struct Port {
    num: u16,
    open: bool,
    proto: Proto,
}
#[derive(Default, Clone, Debug)]
pub struct Proxy {
    proto: Proto,
    host: String,
    port: u16,
}
impl Proxy {
    pub fn check_host(&self) -> bool {
        let re = Regex::new(r"^\d{3}.\d{3}.\d{3}.\d{3}$").unwrap();
        return re.is_match(&self.host);
    }
}
fn make_request(host: &str, port: u16) -> String {
    format!(
        "CONNECT {0}:{1} HTTP/1.1\r\n\
         Host: {0}:{1}\r\n\
         Proxy-Connection: Keep-Alive\r\n",
        host, port
    )
}
fn make_request_without_basic_auth(host: &str, port: u16) -> String {
    let mut request = make_request(host, port);
    request.push_str("\r\n");
    request
}

pub async fn compute_proxy(proxy: Proxy, timeout: u64, retrys: usize) -> (bool, Proto) {
    let dur = std::time::Duration::from_secs(timeout);
    match proxy.proto {
        Proto::HTTPS => {
            let mut res = (false, Proto::HTTPS);
            let connector = async_tls::TlsConnector::default();
            let addrs = format!("{}:{}", proxy.host.as_str(), proxy.port);
            if let Ok(Ok(socket)) =
                future::timeout(dur, async_std::net::TcpStream::connect(addrs.clone())).await
            {
                let _connector = connector.clone();
                if let Ok(Ok(mut stream_socket)) =
                    future::timeout(dur, _connector.connect(proxy.host.as_str(), socket)).await
                {
                    let hello = format!(
                        "CONNECT {0}:{1} HTTP/1.1\r\n\
                             Host: {0}:{1}\r\n\
                             Proxy-Connection: Keep-Alive\r\n",
                        proxy.host.as_str(),
                        proxy.port
                    );
                    let request = hello.as_bytes();
                    if let Ok(_) = future::timeout(Duration::from_millis(900), async {
                        stream_socket.write_all(&request.clone())
                    })
                    .await
                    {
                        let mut buf = [0; 4096];
                        if let Ok(_) = future::timeout(Duration::from_millis(900), async {
                            stream_socket.read(&mut buf)
                        })
                        .await
                        {
                            const MAXIMUM_RESPONSE_HEADERS: usize = 16;
                            let mut response_headers = [EMPTY_HEADER; MAXIMUM_RESPONSE_HEADERS];
                            let mut response = Response::new(&mut response_headers[..]);
                            if let Ok(_) = response.parse(&buf) {
                                if response.code == Some(200) {
                                    res = (true, Proto::HTTP);
                                }
                            }
                        }
                    }
                }
            }
            return res;
        }
        Proto::HTTP => {
            let mut res = (false, Proto::HTTP);
            for _ in 0..retrys {
                let addrs = format!("{}:{}", proxy.host.as_str(), proxy.port);
                if let Ok(Ok(mut socket)) =
                    future::timeout(dur, async { std::net::TcpStream::connect(addrs.clone()) })
                        .await
                {
                    let hello = format!(
                        "CONNECT {0}:{1} HTTP/1.1\r\n\
                         Host: {0}:{1}\r\n\
                         Proxy-Connection: Keep-Alive\r\n",
                        proxy.host.as_str(),
                        proxy.port
                    );
                    let request = hello.as_bytes();
                    if let Ok(_) = future::timeout(Duration::from_millis(900), async {
                        socket.write_all(&request.clone())
                    })
                    .await
                    {
                        let mut buf = [0; 4096];
                        if let Ok(_) = future::timeout(Duration::from_millis(900), async {
                            let _ = socket.set_ttl(255);
                            socket.read(&mut buf)
                        })
                        .await
                        {
                            const MAXIMUM_RESPONSE_HEADERS: usize = 16;
                            let mut response_headers = [EMPTY_HEADER; MAXIMUM_RESPONSE_HEADERS];
                            let mut response = Response::new(&mut response_headers[..]);
                            if let Ok(_) = response.parse(&buf) {
                                if response.code == Some(200) {
                                    res = (true, Proto::HTTP);
                                    break;
                                }
                            }
                        }
                    };
                }
            }
            return res;
        }
        Proto::SOCKS5 => {
            let mut res = (false, Proto::SOCKS5);
            for _ in 0..retrys {
                let addrs = format!("{}:{}", proxy.host.as_str(), proxy.port);
                if let Ok(Ok(socket)) =
                    future::timeout(dur, TcpStream::connect(addrs.clone())).await
                {
                    let packet_len = 3;
                    let packet = [
                        5, // protocol version
                        1, // method count
                        0, // method
                        0, // no auth (always offered)
                    ];
                    if let Ok(_) = future::timeout(Duration::from_millis(900), async {
                        let _ = socket.writable().await;
                        socket.try_write(&packet[..packet_len])
                    })
                    .await
                    {
                        let mut buf = [0; 2];
                        if let Ok(Ok(_)) = future::timeout(Duration::from_millis(900), async {
                            let _ = socket.readable().await;
                            let _ = socket.set_ttl(255); // linux
                            socket.try_read(&mut buf)
                        })
                        .await
                        {
                            let response_version = buf[0];
                            if response_version == 5 {
                                res = (true, Proto::SOCKS5);
                                break;
                            }
                        }
                    };
                }
            }
            return res;
        }
        Proto::SOCKS4 => {
            let mut res = (false, Proto::SOCKS4);
            for _ in 0..retrys {
                let addrs = format!("{}:{}", proxy.host.as_str(), proxy.port);
                if let Ok(Ok(socket)) =
                    future::timeout(dur, TcpStream::connect(addrs.clone())).await
                {
                    let packet_len = 3;
                    let packet = [
                        4, // protocol version
                        1, // method count
                        0, // method
                        0, // no auth (always offered)
                    ];
                    if let Ok(_) = future::timeout(Duration::from_millis(900), async {
                        let _ = socket.writable().await;
                        socket.try_write(&packet[..packet_len])
                    })
                    .await
                    {
                        let mut buf = [0; 2];
                        if let Ok(Ok(_)) = future::timeout(Duration::from_millis(900), async {
                            let _ = socket.readable().await;
                            let _ = socket.set_ttl(255);
                            socket.try_read(&mut buf)
                        })
                        .await
                        {
                            let response_version = buf[0];
                            if response_version == 5 {
                                res = (true, Proto::SOCKS4);
                                break;
                            }
                        }
                    };
                }
            }
            return res;
        }
        Proto::UNKNOWN => {
            let (tx1, rx1) = oneshot::channel::<bool>();
            let (tx2, rx2) = oneshot::channel::<bool>();
            let (tx3, rx3) = oneshot::channel::<bool>();
            let (tx4, rx4) = oneshot::channel::<bool>();
            let host = proxy.host.clone();
            let mut handlers: Vec<JoinHandle<()>> = vec![];
            handlers.push(tokio::spawn(async move {
                let mut _retu = false;
                for _ in 0..retrys {
                    let addrs = format!("{}:{}", host.as_str(), proxy.port);
                    if let Ok(Ok(socket)) = future::timeout(
                        Duration::from_millis(800),
                        TcpStream::connect(addrs.clone()),
                    )
                    .await
                    {
                        let packet_len = 3;
                        let packet = [
                            4, // protocol version
                            1, // method count
                            0, // method
                            0, // no auth (always offered)
                        ];
                        if let Ok(_) = future::timeout(Duration::from_millis(900), async {
                            let _ = socket.writable().await;
                            socket.try_write(&packet[..packet_len])
                        })
                        .await
                        {
                            let mut buf = [0; 2];
                            if let Ok(Ok(_)) = future::timeout(Duration::from_millis(900), async {
                                let _ = socket.readable().await;
                                let _ = socket.set_ttl(255);
                                socket.try_read(&mut buf)
                            })
                            .await
                            {
                                let response_version = buf[0];
                                if response_version == 5 {
                                    _retu = true;
                                    break;
                                }
                            }
                        };
                    }
                }
                let _ = tx1.send(_retu);
            }));
            let host = proxy.host.clone();
            handlers.push(tokio::spawn(async move {
                let mut _retu = false;
                for _ in 0..retrys {
                    let addrs = format!("{}:{}", host.as_str(), proxy.port);
                    if let Ok(Ok(socket)) =
                        future::timeout(dur, TcpStream::connect(addrs.clone())).await
                    {
                        let packet_len = 3;
                        let packet = [
                            5, // protocol version
                            1, // method count
                            0, // method
                            0, // no auth (always offered)
                        ];
                        if let Ok(_) = future::timeout(Duration::from_millis(900), async {
                            let _ = socket.writable().await;
                            socket.try_write(&packet[..packet_len])
                        })
                        .await
                        {
                            let mut buf = [0; 2];
                            if let Ok(Ok(_)) = future::timeout(Duration::from_millis(900), async {
                                let _ = socket.readable().await;
                                let _ = socket.set_ttl(255); // linux
                                socket.try_read(&mut buf)
                            })
                            .await
                            {
                                let response_version = buf[0];
                                if response_version == 5 {
                                    _retu = true;
                                    break;
                                }
                            }
                        };
                    }
                }
                let _ = tx2.send(_retu);
            }));
            let host = proxy.host.clone();
            handlers.push(tokio::spawn(async move {
                let mut _retu = false;
                for _ in 0..retrys {
                    let addrs = format!("{}:{}", host.as_str(), proxy.port);
                    if let Ok(Ok(socket)) =
                        future::timeout(dur.clone(), TcpStream::connect(addrs.clone())).await
                    {
                        let hello =
                            format!("CONNECT {}:{} HTTP/1.1\r\n\r\n", host.as_str(), proxy.port);
                        let request = hello.as_bytes();
                        if let Ok(_) = future::timeout(Duration::from_millis(900), async {
                            let _ = socket.writable().await;
                            socket.try_write(&request.clone())
                        })
                        .await
                        {
                            let mut buf = [0; 1024];
                            if let Ok(Ok(_)) = future::timeout(Duration::from_millis(900), async {
                                let _ = socket.set_ttl(255);
                                let _ = socket.readable().await;
                                socket.try_read(&mut buf)
                            })
                            .await
                            {
                                let ok = b"HTTP/1.1 200 OK\r\n";
                                if &buf[..ok.len()] == ok {
                                    _retu = true
                                }
                            }
                        };
                    }
                }
                let _ = tx3.send(_retu);
            }));
            let host = proxy.host.clone();
            handlers.push(tokio::spawn(async move {
                let connector = async_tls::TlsConnector::default();
                let mut _retu = false;
                let addrs = format!("{}:{}", host.as_str(), proxy.port);
                if let Ok(Ok(socket)) =
                    future::timeout(dur, async_std::net::TcpStream::connect(addrs.clone())).await
                {
                    let _connector = connector.clone();
                    if let Ok(Ok(mut stream_socket)) =
                        future::timeout(dur, _connector.connect(host.as_str(), socket)).await
                    {
                        let hello =
                            format!("CONNECT {}:{} HTTP/1.1\r\n\r\n", host.as_str(), proxy.port);
                        let request = hello.as_bytes();
                        if let Ok(_) = future::timeout(Duration::from_millis(900), async {
                            stream_socket.write_all(&request.clone())
                        })
                        .await
                        {
                            let mut buf = [0; 1024];
                            if let Ok(_) = future::timeout(Duration::from_millis(900), async {
                                stream_socket.read(&mut buf)
                            })
                            .await
                            {
                                let ok = b"HTTP/1.1 200 OK\r\n";
                                if &buf[..ok.len()] == ok {
                                    _retu = true
                                }
                            }
                        };
                    }
                }
                let _ = tx4.send(_retu);
            }));
            let mut res = None;
            let sleep = tokio::time::sleep(Duration::from_secs(24));
            tokio::pin!(sleep);
            tokio::select! {
                Ok(val) = rx2 => {
                    if val {
                        res = Some((true, Proto::SOCKS5));
                        let _ = handlers.iter().map(|h|{
                            if !h.is_finished() {
                                h.abort();
                            }
                        });
                    } else {
                        res = Some((false, Proto::UNKNOWN));
                    }
                },
                Ok(val) = rx1 => {
                    if val && res == None {
                        res = Some((true, Proto::SOCKS4));
                        let _ = handlers.iter().map(|h|{
                            if !h.is_finished() {
                                h.abort();
                            }
                        });
                    } else {
                        res = Some((false, Proto::UNKNOWN));
                    }
                },
                Ok(val) = rx3 => {
                    if val && res == None {
                        res = Some((true, Proto::HTTP));
                        let _ = handlers.iter().map(|h|{
                            if !h.is_finished() {
                                h.abort();
                            }
                        });
                    } else {
                        res = Some((false, Proto::UNKNOWN));
                    }
                },
                Ok(val) = rx4 => {
                    if val && res == None {
                        res = Some((true, Proto::HTTPS));
                        let _ = handlers.iter().map(|h|{
                            if !h.is_finished() {
                                h.abort();
                            }
                        });
                    } else {
                        res = Some((false, Proto::UNKNOWN));
                    }
                },
                _ = &mut sleep => {
                    println!("timeout!");
                    let _ = handlers.iter().map(|h|{
                        if !h.is_finished() {
                            h.abort();
                        }
                    });
                    res = Some((false, Proto::UNKNOWN));
                }
            }
            match res {
                Some(m) => m,
                None => (false, Proto::UNKNOWN),
            }
        }
    }
}
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
pub async fn readfile(path: String) -> Option<Vec<Proxy>> {
    let pth = Path::new(&path);
    if !pth.is_file() { // for somehownot working proporly
    println!("file \"{:?}\" doesn't exist!", pth.to_str().unwrap());
        return None;
    }
    if let Ok(lines) = read_lines(pth) {
        let mut _proxies = vec![];
        let list = lines
            .filter(|line| line.is_ok())
            .map(|p| p.unwrap())
            .collect::<Vec<String>>();
        _proxies = list
            .into_par_iter()
            .enumerate()
            .filter_map(|(_i, p)| {
                let mut __proxy = p.split(":").map(|s| s.to_string()).collect::<Vec<String>>();
                if __proxy.len() == 2 {
                    __proxy.insert(0, "UNKNOWN".into());
                }
                let _proto = match __proxy[0].to_uppercase().as_str() {
                    "HTTP" | "HTTPS" => Proto::HTTP,
                    "SOCKS4" => Proto::SOCKS4,
                    "SOCKS5" => Proto::SOCKS5,
                    "UNKNOWN" => Proto::UNKNOWN,
                    _ => Proto::UNKNOWN,
                };
                let _port = match __proxy[2].parse::<u16>() {
                    Ok(m) => m,
                    Err(_) => 0,
                };
                let current_proxy = Proxy {
                    proto: _proto.clone(),
                    host: __proxy[1].clone(),
                    port: _port,
                };
                if !__proxy[1].is_empty() && current_proxy.check_host() && _port != 0 {
                    Some(current_proxy)
                } else {
                    None
                }
            })
            .collect();
        return Some(_proxies);
    } else {
        return None;
    }
}
pub async fn concurrent_threads(
    threads: Option<usize>,
    proxies: Vec<Proxy>,
    timeout: u64,
    retrys: usize,
    outfile: Option<String>,
) {
    let max_threads = match std::thread::available_parallelism() {
        Ok(s) => s.get(),
        Err(_) => 5,
    };
    let thread_number = match threads {
        Some(m) => {
            if m > max_threads {
                max_threads
            } else {
                m
            }
        }
        None => max_threads,
    };
    let file = match outfile {
        Some(m) => {
            let directory = env::current_dir().unwrap();
            let file = directory.join(m);
            File::create(file).unwrap()
        }
        None => {
            let directory = env::current_dir().unwrap();
            let file = directory.join("live.txt");
            File::create(file).unwrap()
        }
    };
    // let (tx, rx) = channel::<Proxy>();
    let _ = stream::iter(proxies)
        .for_each_concurrent(thread_number, |mut proxie| {
            let mut txn = file.try_clone().unwrap();
            async move {
                let is_valid = compute_proxy(proxie.clone(), timeout, retrys).await;
                if is_valid.0 {
                    proxie.proto = is_valid.1;
                    println!("{:?} {}", proxie.clone(), "✅");
                    let res = proxie.clone();
                    let _ = txn.write(
                        format!("{:?}:{}:{}\n", res.proto, res.host, res.port)
                            .to_lowercase()
                            .as_bytes(),
                    );
                } else {
                    println!("{:?} {}", proxie.clone(), "❌");
                }
            }
        })
        .await;
}
pub async fn check_proxies(
    threads: Option<usize>,
    proxies: Vec<Proxy>,
    timeout: u64,
    retrys: usize,
) -> Option<Vec<Proxy>> {
    let max_threads = match std::thread::available_parallelism() {
        Ok(s) => s.get(),
        Err(_) => 5,
    };
    let thread_number = match threads {
        Some(m) => {
            if m > max_threads {
                max_threads
            } else {
                m
            }
        }
        None => max_threads,
    };
    let data = Arc::new(Mutex::new(vec![]));
    let _ = stream::iter(proxies)
        .for_each_concurrent(thread_number, |mut proxie| {
            let mut result = data.lock().unwrap();
            async move {
                let is_valid = compute_proxy(proxie.clone(), timeout, retrys).await;
                if is_valid.0 {
                    proxie.proto = is_valid.1;
                    println!("{:?} {}", proxie.clone(), "✅");
                    let res = proxie.clone();
                    result.push(res);
                } else {
                    println!("{:?} {}", proxie.clone(), "❌");
                }
            }
        })
        .await;
    match data.clone().lock() {
        Ok(m) => Some(m.clone()),
        Err(_) => None,
    }
}

#[tokio::test]
async fn test_check_port() {
    // check a working proxy to see returns type.
    let proxies = readfile("./socks5.txt".into(), ).await;
    if proxies.is_some() {
        println!("🔥 start computing! 🔥");
        for proxie in proxies.unwrap() {
            let resy = compute_proxy(proxie.clone(), 1, 2).await;
            println!("{:?}", resy);
        }
    }
    
}
