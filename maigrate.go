package main

import (
	"image/color"
	"log"
	"sync"

	color "github.com/fatih/color"
)

const (
	userAgent       string = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/90.0.4430.93 Safari/537.36"
	screenShotRes   string = "1024x768"
	torProxyAddress string = "socks5://127.0.0.1:9050"
)

var (
	maxGoroutines int = 32
	guard         chan int
)

type Result struct {
	Username string
	Exist    bool
	Proxied  bool
	Site     string
	URL      string
	URLProbe string
	Link     string
	Err      bool
	ErrMsg   string
}

var (
	waitGroup      = &sync.WaitGroup{}
	logger         = log.New(color.Output, "", 0)
	siteData       = map[string]SiteData{}
	dataFileName   = "data.json"
	specifiedSites string
	options        struct {
		noColor         bool
		verbose         bool
		updateBeforeRun bool
		runTest         bool
		useCustomData   bool
		withTor         bool
		withScreenshot  bool
		specifySite     bool
		download        bool
	}
)

type SiteData struct {
	ErrorType      string `json:"errorType"`
	ErrorMsg       string `json:"errorMsg"`
	URL            string `json:"url"`
	URLMain        string `json:"urlMain"`
	URLProbe       string `json:"urlProbe"`
	URLError       string `json:"errorUrl"`
	UsedUsername   string `json:"username_claimed"`
	UnusedUsername string `json:"username_unclaimed"`
	RegexCheck     string `json:"regexCheck"`
}

type RequestError interface {
	Error() string
}

type counter struct {
	n int32
}
