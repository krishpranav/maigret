package chrome

import (
	"io/ioutil"
	"os"

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
		log.WithFields(log.Fields{"user-path": chrome.Path, "error": err}).Debug("Chrome path not set or invalid. Performing search")
	} else {
		log.Debug("Chrome path exists, skipping search and version check")
		return
	}
}
