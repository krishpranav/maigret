package main

const (
	userAgent       string = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/90.0.4430.93 Safari/537.36"
	screenShotRes   string = "1024x768"
	torProxyAddress string = "socks5://127.0.0.1:9050"
)

var (
	maxGoroutines int = 32
	guard         chan int
)
