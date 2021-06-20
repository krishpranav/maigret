package chrome

import (
	"io/ioutil"

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
