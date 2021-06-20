package downloader

import (
	"log"
	"strings"
)

func downloadInstagram(url string, logger *log.Logger) {

	_splitURL := strings.Split(url, "/")
	username := _splitURL[len(_splitURL)-1]
}
