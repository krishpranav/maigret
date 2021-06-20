package downloader

import (
	"log"
	"os"
	"strings"
)

func downloadInstagram(url string, logger *log.Logger) {

	_splitURL := strings.Split(url, "/")
	username := _splitURL[len(_splitURL)-1]

	OUT := "./downloads/" + username + "/instagram/"
	os.MkdirAll(OUT, os.ModePerm)

}
