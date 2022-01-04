# SSTP (Steve's Super Transfer Program) 

Some time ago I came across [Croc](https://github.com/schollz/croc), a file transfer program written in Go by schollz. It struck me as a really good program that I could actually use in my daily life.
Since I've been trying to learn some Rust and at the same time expand my networking knowledge, I decided to give it a try and create a Rust implementation (-ish, at least get the same functionality out of it) of it. WIP.

## Install
```
cargo install sstp
```

## Usage
**Send a file:**
```
sstp send [path/to/file]
```
Running the previous command will generate a unique code. Use that code from a different device to start the transfer.

**Recieve a file:**
```
sstp send [code]
```

**Start a relay server:**
```
sstp relay
```

## TODOs:
- [ ] Add extra parameters: Relay address(as a domain name) and custom code.
- [ ] Add compression
- [ ] Add e2e encryption (PAKE or other protocol)
- [ ] Automate tests
- [ ] Use multiple ports on the relay...?
- [x] Improve project structure. Change into cli/lib workspace maybe?
- [ ] Fix room cleanup
- [ ] Resume interrupted transfers
- [ ] Support more than 1 file transfers.
- [ ] Use Defeault trait for params?
