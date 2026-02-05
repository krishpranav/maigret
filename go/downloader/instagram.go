package downloader

import (
	"io"
	"io/ioutil"
	"log"
	"net/http"
	"os"
	"strconv"
	"strings"
	"sync"

	"github.com/tidwall/gjson"
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
	bdB, _ := ioutil.ReadAll(r.Body)
	r.Body.Close()

	targetURIs = append(targetURIs, gjson.GetBytes(bdB, "graphql.user.profile_pic_url_hd").String())

	addURIFromNode := func(node gjson.Result) {
		var targetURI string
		if node.Get("is_video").Bool() {
			targetURI = node.Get("video_url").String()
		} else {
			targetURI = node.Get("display_url").String()
		}
		targetURIs = append(targetURIs, targetURI)
	}

	for _, edge := range gjson.GetBytes(bdB, "graphql.user.edge_owner_to_timeline_media.edges").Array() {
		node := edge.Get("node")
		addURIFromNode(node)
		for i, subEdge := range node.Get("edge_sidecar_to_children.edges").Array() {
			if i != 0 {
				subNode := subEdge.Get("node")
				addURIFromNode(subNode)
			}
		}
	}

	wg.Add(len(targetURIs))
	for i, uri := range targetURIs {
		go func(i int, uri string) {
			defer wg.Done()
			_splitURL := strings.Split(strings.Split(uri, "?")[0], ".")

			file, err := os.Create(OUT + strconv.Itoa(i) + "." + _splitURL[len(_splitURL)-1])
			if err != nil {
				log.Fatal(err)
			}

			r, err := http.Get(uri)
			if err != nil {
				log.Fatal(err)
			}

			_, err = io.Copy(file, r.Body)
			if err != nil {
				log.Fatal(err)
			}

			r.Body.Close()
			file.Close()
		}(i, uri)
	}
	wg.Wait()
}
