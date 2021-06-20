package main

import (
	"encoding/json"
	"fmt"
	"image/color"
	"io/ioutil"
	"log"
	"net/http"
	"net/url"
	"os"
	"path/filepath"
	"strings"
	"sync"
	"sync/atomic"
	"time"

	"github.com/dlclark/regexp2"
	color "github.com/fatih/color"
	"github.com/krishpranav/maigrate/downloader"
	"golang.org/x/net/proxy"
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

func (c *counter) Add() {
	atomic.AddInt32(&c.n, 1)
}

func (c *counter) Get() int {
	return int(atomic.LoadInt32(&c.n))
}

func parseArguments() []string {
	args := os.Args[1:]
	var argIndex int

	if help, _ := HasElement(args, "-h", "--help"); help && !options.runTest {
		fmt.Print(
			`maigrate - User Osint Across Social Networks.
usage: maigrate USERNAME [USERNAMES...] flags options
perform test: maigrate --test
positional arguments:
        USERNAMES             one or more usernames to investigate
flags:
        -h, --help            show this help message and exit
        --no-color            disable colored stdout output
        --update              update database before run from Sherlock repository
        -t, --tor             use tor proxy
        -s, --screenshot      take a screenshot of each matched urls
        -v, --verbose         verbose output
        -d, --download        download the contents of site if available
options:
        --database DATABASE   use custom database
        --site SITE           specific site to investigate
`,
		)
		os.Exit(0)
	}

	if len(args) < 1 {
		fmt.Println("WARNING: You executed maigrate without arguments. Use `-h` flag if you need help.")
		fmt.Printf("Input username to investigate:")
		var _usernames string
		fmt.Scanln(&_usernames)
		return strings.Split(_usernames, " ")
	}

	options.noColor, argIndex = HasElement(args, "--no-color")
	if options.noColor {
		logger = log.New(os.Stdout, "", 0)
		args = append(args[:argIndex], args[argIndex+1:]...)
	}

	options.withTor, argIndex = HasElement(args, "-t", "--tor")
	if options.withTor {
		args = append(args[:argIndex], args[argIndex+1:]...)
	}

	options.withScreenshot, argIndex = HasElement(args, "-s", "--screenshot")
	if options.withScreenshot {
		args = append(args[:argIndex], args[argIndex+1:]...)
		maxGoroutines = 8
	} else {
		maxGoroutines = 32
	}

	options.runTest, argIndex = HasElement(args, "--test")
	if options.runTest {
		args = append(args[:argIndex], args[argIndex+1:]...)
	}

	options.verbose, argIndex = HasElement(args, "-v", "--verbose")
	if options.verbose {
		args = append(args[:argIndex], args[argIndex+1:]...)
	}

	options.updateBeforeRun, argIndex = HasElement(args, "--update")
	if options.updateBeforeRun {
		args = append(args[:argIndex], args[argIndex+1:]...)
	}

	options.useCustomData, argIndex = HasElement(args, "--database")
	if options.useCustomData {
		dataFileName = args[argIndex+1]
		args = append(args[:argIndex], args[argIndex+2:]...)
	}

	options.specifySite, argIndex = HasElement(args, "--site")
	if options.specifySite {
		specifiedSites = strings.ToLower(args[argIndex+1])
		// Use verbose output
		options.verbose = true
		args = append(args[:argIndex], args[argIndex+2:]...)
	}

	options.download, argIndex = HasElement(args, "-d", "--download")
	if options.download {
		if len(args) <= 1 {
			fmt.Println("List of sites that can download userdata")
			for key := range downloader.Impls {
				fmt.Fprintf(color.Output, "[%s] %s\n", color.HiGreenString("+"), color.HiWhiteString(key))
			}
			os.Exit(0)
		}
		args = append(args[:argIndex], args[argIndex+1:]...)
	}

	return args
}

func main() {
	usernames := parseArguments()

	initializeSiteData(options.updateBeforeRun)

	guard = make(chan int, maxGoroutines)

	if options.runTest {
		test()
		os.Exit(0)
	}

	if options.specifySite {
		for _, username := range usernames {
			_siteData := map[string]SiteData{}

			for siteName, v := range siteData {
				_siteData[strings.ToLower(siteName)] = v
			}

			if options.noColor {
				fmt.Printf("\nInvestigating %s on:\n", username)
			} else {
				fmt.Fprintf(color.Output, "Investigating %s on:\n", color.HiGreenString(username))
			}
			site := specifiedSites

			if val, ok := _siteData[site]; ok {
				res := Maigrate(username, site, val)
				WriteResult(res)
			} else {
				log.Printf("[!] %s is not a valid site.", site)
			}
		}
	} else {
		for _, username := range usernames {
			if options.noColor {
				fmt.Printf("\nInvestigating %s on:\n", username)
			} else {
				fmt.Fprintf(color.Output, "Investigating %s on:\n", color.HiGreenString(username))
			}
			waitGroup.Add(len(siteData))
			for site := range siteData {
				guard <- 1
				go func(site string) {
					defer waitGroup.Done()
					res := Maigrate(username, site, siteData[site])
					WriteResult(res)
					<-guard
				}(site)
			}
			waitGroup.Wait()
		}
	}
}

func initializeSiteData(forceUpdate bool) {
	jsonFile, err := os.Open(dataFileName)
	if err != nil || forceUpdate {
		if err != nil {
			if options.noColor {
				fmt.Printf(
					"[!] Cannot open database \"%s\"\n",
					dataFileName,
				)
			} else {
				fmt.Fprintf(
					color.Output,
					"[%s] Cannot open database \"%s\"\n",
					color.HiRedString("!"), (dataFileName),
				)
			}
		}
		if options.noColor {
			fmt.Printf(
				"%s Update database: %s",
				("[!]"),
				("Downloading..."),
			)
		} else {
			fmt.Fprintf(
				color.Output,
				"[%s] Update database: %s",
				color.HiBlueString("!"),
				color.HiYellowString("Downloading..."),
			)
		}

		if forceUpdate {
			jsonFile.Close()
		}

		r, err := Request("https://raw.githubusercontent.com/sherlock-project/sherlock/master/sherlock/resources/data.json")

		if err != nil || r.StatusCode != 200 {
			if options.noColor {
				fmt.Printf(" [%s]\n", ("Failed"))
			} else {
				fmt.Fprintf(color.Output, " [%s]\n", color.HiRedString("Failed"))
			}
			if err != nil {
				panic("Failed to update database.\n" + err.Error())
			} else {
				panic("Failed to update database: " + r.Status)
			}
		} else {
			defer r.Body.Close()
		}
		if _, err := os.Stat(dataFileName); !os.IsNotExist(err) {
			if err = os.Remove(dataFileName); err != nil {
				panic(err)
			}
		}
		_updateFile, _ := os.OpenFile(dataFileName, os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0600)
		if _, err := _updateFile.WriteString(ReadResponseBody(r)); err != nil {
			if options.noColor {
				fmt.Printf("Failed to update data.\n")
			} else {
				fmt.Fprint(color.Output, color.RedString("Failed to update data.\n"))
			}
			panic(err)
		}

		_updateFile.Close()
		jsonFile, _ = os.Open(dataFileName)

		if options.noColor {
			fmt.Println(" [Done]")
		} else {
			fmt.Fprintf(color.Output, " [%s]\n", color.GreenString("Done"))
		}
	}

	defer jsonFile.Close()

	byteValue, err := ioutil.ReadAll(jsonFile)
	if err != nil {
		panic("Error while read " + dataFileName)
	} else {
		json.Unmarshal([]byte(byteValue), &siteData)
	}
}

func Request(target string) (*http.Response, RequestError) {
	request, err := http.NewRequest("GET", target, nil)
	if err != nil {
		return nil, err
	}
	request.Header.Set("User-Agent", userAgent)

	client := &http.Client{
		Timeout: 60 * time.Second,
	}

	if options.withTor {
		tbProxyURL, err := url.Parse(torProxyAddress)
		if err != nil {
			return nil, err
		}
		tbDialer, err := proxy.FromURL(tbProxyURL, proxy.Direct)
		if err != nil {
			return nil, err
		}
		tbTransport := &http.Transport{
			Dial: tbDialer.Dial,
		}
		client.Transport = tbTransport
	}

	return client.Do(request)
}

func ReadResponseBody(response *http.Response) string {
	bodyBytes, err := ioutil.ReadAll(response.Body)
	if err != nil {
		panic(err)
	}
	return string(bodyBytes)
}

func HasElement(array []string, targets ...string) (bool, int) {
	for index, item := range array {
		for _, target := range targets {
			if item == target {
				return true, index
			}
		}
	}
	return false, -1
}

func Maigrate(username string, site string, data SiteData) Result {
	var u, urlProbe string
	var result Result

	u = strings.Replace(data.URL, "{}", username, 1)

	if data.URLProbe != "" {
		urlProbe = strings.Replace(data.URLProbe, "{}", username, 1)
	} else {
		urlProbe = u
	}

	if data.RegexCheck != "" {
		re := regexp2.MustCompile(data.RegexCheck, 0)
		if match, _ := re.MatchString(username); !match {
			return Result{
				Username: username,
				URL:      data.URL,
				Proxied:  options.withTor,
				Site:     site,
				Exist:    false,
				Err:      false,
			}
		}
	}

	r, err := Request(urlProbe)

	if err != nil {
		if r != nil {
			r.Body.Close()
		}
		return Result{
			Username: username,
			URL:      data.URL,
			URLProbe: data.URLProbe,
			Proxied:  options.withTor,
			Exist:    false,
			Site:     site,
			Err:      true,
			ErrMsg:   err.Error(),
		}
	}

	// check error types
	switch data.ErrorType {
	case "status_code":
		if r.StatusCode == http.StatusOK {
			result = Result{
				Username: username,
				URL:      data.URL,
				URLProbe: data.URLProbe,
				Proxied:  options.withTor,
				Exist:    true,
				Link:     u,
				Site:     site,
			}
		} else {
			result = Result{
				Username: username,
				URL:      data.URL,
				Proxied:  options.withTor,
				Site:     site,
				Exist:    false,
				Err:      false,
			}
		}
	case "message":
		if !strings.Contains(ReadResponseBody(r), data.ErrorMsg) {
			result = Result{
				Username: username,
				URL:      data.URL,
				URLProbe: data.URLProbe,
				Proxied:  options.withTor,
				Exist:    true,
				Link:     u,
				Site:     site,
			}
		} else {
			result = Result{
				Username: username,
				URL:      data.URL,
				Proxied:  options.withTor,
				Site:     site,
				Exist:    false,
				Err:      false,
			}
		}
	case "response_url":
		if (r.StatusCode <= 300 || r.StatusCode < 200) && r.Request.URL.String() == u {
			result = Result{
				Username: username,
				URL:      data.URL,
				URLProbe: data.URLProbe,
				Proxied:  options.withTor,
				Exist:    true,
				Link:     u,
				Site:     site,
			}
		} else {
			result = Result{
				Username: username,
				URL:      data.URL,
				Proxied:  options.withTor,
				Site:     site,
				Exist:    false,
				Err:      false,
			}
		}
	default:
		result = Result{
			Username: username,
			Proxied:  options.withTor,
			Exist:    false,
			Err:      true,
			ErrMsg:   "Unsupported error type `" + data.ErrorType + "`",
			Site:     site,
		}
	}

	if result.Exist && options.withScreenshot {
		urlParts, _ := url.Parse(urlProbe)
		folderPath := filepath.Join("screenshots", username)
		outputPath := filepath.Join(folderPath, urlParts.Host+".png")
		if err := os.MkdirAll(folderPath, 0755); err != nil {
			log.Fatal(err)
		}
		if err := getScreenshot(screenShotRes, urlProbe, outputPath); err != nil {
			log.Fatal(err)
		}
	}

	if result.Exist && options.download {
		// Check if the downloader for this site exists
		if downloadFunc, ok := downloader.Impls[strings.ToLower(site)]; ok {
			downloadFunc.(func(string, *log.Logger))(urlProbe, logger)
		}
	}

	r.Body.Close()

	return result
}

func WriteResult(result Result) {
	if options.noColor {
		if result.Exist {
			logger.Printf("[%s] %s: %s\n", ("+"), result.Site, result.Link)
		} else {
			if options.verbose {
				if result.Err {
					logger.Printf("[%s] %s: %s: %s", ("!"), result.Site, ("ERROR"), (result.ErrMsg))
				} else {
					logger.Printf("[%s] %s: %s", ("-"), result.Site, ("Not Found!"))
				}
			}
		}
	} else {
		if result.Exist {
			logger.Printf("[%s] %s: %s\n", color.HiGreenString("+"), color.HiWhiteString(result.Site), result.Link)
		} else {
			if options.verbose {
				if result.Err {
					logger.Printf("[%s] %s: %s: %s", color.HiRedString("!"), result.Site, color.HiMagentaString("ERROR"), color.HiRedString(result.ErrMsg))
				} else {
					logger.Printf("[%s] %s: %s", color.HiRedString("-"), result.Site, color.HiYellowString("Not Found!"))
				}
			}
		}
	}
}
