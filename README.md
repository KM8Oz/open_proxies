

<h1 id="top" align="center">Open Proxies</h1>
<p id="top" color="#343434" align="center">⭐️ <font color="#F7C815">Leave me a start please</font> ⭐️</p>
<p id="top" align="center">
<font color="#F7C815">it will motivate me to continue maintaining and adding futures</font></p>
<div align="center" style="display:flex;flex-direction:row;gap:5px; width:100%;justify-content:center;">
  <img alt="Github top language" href="https://crates.io/crates/tinkoffpay" src="https://img.shields.io/github/languages/top/KM8Oz/open_proxies?color=56BEB8">

  <img alt="Github language count" href="https://crates.io/crates/tinkoffpay" src="https://img.shields.io/github/languages/count/KM8Oz/open_proxies?color=56BEB8">

  <img alt="Repository size" href="https://crates.io/crates/tinkoffpay" src="https://img.shields.io/github/repo-size/KM8Oz/open_proxies?color=56BEB8">

  <img alt="License" href="https://crates.io/crates/open_proxies" src="https://img.shields.io/github/license/KM8Oz/open_proxies?color=56BEB8">
  <img alt="Crates.io" href="https://crates.io/crates/open_proxies" src="https://img.shields.io/crates/v/open_proxies?color=56BEB8&label=open_proxies">
  <!-- <img alt="Github issues" src="https://img.shields.io/github/issues/KM8Oz/open_proxies?color=56BEB8" /> -->

  <!-- <img alt="Github forks" src="https://img.shields.io/github/forks/KM8Oz/open_proxies?color=56BEB8" /> -->

  <!-- <img alt="Github stars" src="https://img.shields.io/github/stars/KM8Oz/open_proxies?color=56BEB8" /> -->
</div>

<!-- Status -->

<!-- <h4 align="center"> 
	🚧  open proxies 🚀 Under developement...  🚧
</h4> 

<hr> -->

<p align="center" >
  <a href="#-about">About</a> &#xa0; | &#xa0; 
  <!-- <a href="#sparkles-features">Features</a> &#xa0; | &#xa0; -->
  <a href="#-technologies">Technologies</a> &#xa0; | &#xa0;
  <a href="#-requirements">Requirements</a> &#xa0; | &#xa0;
  <a href="#-starting">Starting</a> &#xa0; | &#xa0;
  <a href="#-lib_usage">lib Usage</a> &#xa0; | &#xa0;
  <a href="#-exec_Usage">exec Usage</a> &#xa0; | &#xa0;
  <a href="#-license">License</a> &#xa0; | &#xa0;
  <a href="https://github.com/KM8Oz" target="_blank">Author</a>
</p>

<br>

## 🎯 About ##

Simple and fast proxy checker that include protocol validation; 

## 🚀 Technologies ##

The following tools were used in this project:

- [rust](https://www.rust-lang.org/)
- [crago](https://crates.io/)

## ✅ Requirements ##

Before starting :checkered_flag:, you need to have [Git](https://git-scm.com) and [Rust](https://www.rust-lang.org/) installed.

## 🏁 Starting ##

```bash
# install using cargo:
$ cargo install open_proxies
# install manualy:
$ curl https://github.com/KM8Oz/open_proxies/archive/refs/tags/[binary]
```

## ✅ lib_Usage ##

```rust
   use open_proxies::{compute_proxy, readfile}
   #[tokio::main]
   async fn main(){
    let proxies = readfile("./socks5.txt".into(), ).await;
    if proxies.is_some() {
        println!("🔥 start computing! 🔥");
        for proxie in proxies.unwrap() {
            let is_valid = compute_proxy(proxie.clone(), 1, 2).await;
            println!("{:?}", is_valid);
        }
    }
   }
```

## ✅ exec_Usage ##

```
Usage: open_proxies [OPTIONS] --input <FILENAME>

Options:
  -i, --input <FILENAME>  TXT file path where proxies ready to be parsed
  -o, --out <FILENAME>    file path where live proxies will be saved [default: live.txt]
  -t, --timeout <NUMBER>  single proxy compute iteration timeout in seconds [default: 2]
  -n, --threads <NUMBER>  threads number used for proxies computing [default: 10]
  -r, --retrys <NUMBER>   how many time a single proxy will be tested (>=1) [default: 2]
  -h, --help              Print help information
  -V, --version           Print version information

USAGE:
  -a <example1>      open_proxies -i ./socks.txt -o ./live.txt -t 2 -r 2 -n 10
  -b <example2>      open_proxies -i ./socks.txt -o ./live.txt
```
&#xa0;
## 📝 License ##

This project is under license from MIT. For more details, see the [LICENSE](LICENCE.md) file.


Made with :heart: by <a href="https://github.com/KM8Oz" target="_blank">@KM8Oz</a>


&#xa0;
<a href="#top">Back to top</a>

