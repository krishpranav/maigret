package downloader

import (
	"log"
	"net/http"
	"os"
	"strings"
	"sync"
)

func downloadInstagram(url string, logger *log.Logger) {

	_splitURL := strings.Split(url, "/")
	username := _splitURL[len(_splitURL)-1]

	OUT := "./downloads/" + username + "/instagram/"
	os.MkdirAll(OUT, os.ModePerm)

	var targetURIs []string
	var wg sync.WaitGroup

	r, err := http.Get(url + "?__a=1")
	if err != nil {
		log.Fatal(err)
	}
}
