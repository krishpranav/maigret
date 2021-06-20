package chrome

import (
	"context"
	"io/ioutil"
	"net/url"
	"os"
	"os/exec"
	"regexp"
	"strconv"
	"time"

	"github.com/pkg/errors"

	gover "github.com/mcuadros/go-version"
	log "github.com/sirupsen/logrus"
)

type Chrome struct {
	Resolution       string
	ChromeTimeout    int
	ChromeTimeBudget int
	Path             string
	UserAgent        string
	Argvs            []string
	ScreenshotPath   string
}

func (chrome *Chrome) setLoggerStatus(status bool) {
	if !status {
		log.SetOutput(ioutil.Discard)
	}
}

func (chrome *Chrome) Setup() {
	chrome.chromeLocator()
}

func (chrome *Chrome) chromeLocator() {
	if _, err := os.Stat(chrome.Path); os.IsNotExist(err) {
		log.WithFields(log.Fields{"user-path": chrome.Path, "error": err}).
			Debug("Chrome path not set or invalid. Performing search")
	} else {

		log.Debug("Chrome path exists, skipping search and version check")
		return
	}

	paths := []string{
		"/usr/bin/chromium",
		"/usr/bin/chromium-browser",
		"/usr/bin/google-chrome-stable",
		"/usr/bin/google-chrome",
		"/usr/bin/chromium-browser",
		"/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
		"/Applications/Google Chrome Canary.app/Contents/MacOS/Google Chrome Canary",
		"/Applications/Chromium.app/Contents/MacOS/Chromium",
		"C:/Program Files (x86)/Google/Chrome/Application/chrome.exe",
	}

	for _, path := range paths {

		if _, err := os.Stat(path); os.IsNotExist(err) {
			continue
		}

		log.WithField("chrome-path", path).Debug("Google Chrome path")
		chrome.Path = path

		if chrome.checkVersion("60") {
			break
		}
	}

	if chrome.Path == "" {
		log.Fatal("Unable to locate a valid installation of Chrome to use. gowitness needs at least Chrome/" +
			"Chrome Canary v60+. Either install Google Chrome or try specifying a valid location with " +
			"the --chrome-path flag")
	}
}

func (chrome *Chrome) checkVersion(lowestVersion string) bool {

	out, err := exec.Command(chrome.Path, "-version").Output()
	if err != nil {
		log.WithFields(log.Fields{"chrome-path": chrome.Path, "err": err}).
			Error("An error occurred while trying to get the Chrome version")
		return false
	}

	version := string(out)

	re := regexp.MustCompile(`\d+(\.\d+)+`)
	match := re.FindStringSubmatch(version)
	if len(match) <= 0 {
		log.WithField("chrome-path", chrome.Path).Debug("Unable to determine Chrome version.")

		return false
	}

	version = match[0]

	if gover.Compare(version, lowestVersion, "<") {
		log.WithFields(log.Fields{"chrome-path": chrome.Path, "chromeversion": version}).
			Warn("Chrome version is older than v" + lowestVersion)

		return false
	}

	log.WithFields(log.Fields{"chrome-path": chrome.Path, "chromeversion": version}).Debug("Chrome version")
	return true
}

func (chrome *Chrome) SetScreenshotPath(p string) error {

	if _, err := os.Stat(p); os.IsNotExist(err) {
		return errors.New("Destination path does not exist")
	}

	log.WithField("screenshot-path", p).Debug("Screenshot path")
	chrome.ScreenshotPath = p

	return nil
}

func (chrome *Chrome) ScreenshotURL(targetURL *url.URL, destination string) {

	log.WithFields(log.Fields{"url": targetURL, "full-destination": destination}).
		Debug("Full path to screenshot save using Chrome")

	var chromeArguments = []string{
		"--headless", "--disable-gpu", "--hide-scrollbars",
		"--disable-crash-reporter", "--no-sandbox",
		// "--disable-software-rasterizer", "--disable-dev-shm-usage",
		"--user-agent=" + chrome.UserAgent,
		"--window-size=" + chrome.Resolution, "--screenshot=" + destination,
		"--virtual-time-budget=" + strconv.Itoa(chrome.ChromeTimeBudget*6000),
	}

	if len(chrome.Argvs) > 0 {
		for _, a := range chrome.Argvs {
			chromeArguments = append(chromeArguments, a)
		}
	}

	log.Info(chromeArguments)

	if os.Geteuid() == 0 {
		log.WithField("euid", os.Geteuid()).Debug("Running as root, adding --no-sandbox")
		chromeArguments = append(chromeArguments, "--no-sandbox")
	}

	if targetURL.Scheme == "https" {

		originalPath := targetURL.Path
		proxy := forwardingProxy{targetURL: targetURL}

		time.Sleep(500 * time.Millisecond)

		if err := proxy.start(); err != nil {

			log.WithField("error", err).Warning("Failed to start proxy for HTTPS request")
			return
		}

		proxyURL, _ := url.Parse("http://localhost:" + strconv.Itoa(proxy.port) + "/")
		proxyURL.Path = originalPath

		chromeArguments = append(chromeArguments, "--allow-insecure-localhost")

		chromeArguments = append(chromeArguments, proxyURL.String())

		defer proxy.stop()

	} else {

		chromeArguments = append(chromeArguments, targetURL.String())
	}

	log.WithFields(log.Fields{"arguments": chromeArguments}).Debug("Google Chrome arguments")

	ctx, cancel := context.WithTimeout(context.Background(), time.Duration(chrome.ChromeTimeout)*time.Second)
	defer cancel()

	cmd := exec.CommandContext(ctx, chrome.Path, chromeArguments...)

	log.WithFields(log.Fields{"url": targetURL, "destination": destination}).Info("Taking screenshot")

	startTime := time.Now()
	if err := cmd.Start(); err != nil {
		log.Fatal(err)
	}

	if err := cmd.Wait(); err != nil {

		if ctx.Err() == context.DeadlineExceeded {
			log.WithFields(log.Fields{"url": targetURL, "destination": destination, "err": err}).
				Error("Timeout reached while waiting for screenshot to finish")
			return
		}

		log.WithFields(log.Fields{"url": targetURL, "destination": destination, "err": err}).
			Error("Screenshot failed")

		return
	}

	log.WithFields(log.Fields{
		"url": targetURL, "destination": destination, "duration": time.Since(startTime),
	}).Info("Screenshot taken")
}
