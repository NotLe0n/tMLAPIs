package main

import (
	"encoding/json"
	"errors"
	"fmt"
	"net/http"
	"net/url"
	"strconv"

	"golang.org/x/net/html"
)

type ListModInfo struct {
	DisplayName        string
	Rank               int
	DownloadsTotal     int
	DownloadsToday     int
	DownloadsYesterday int
	TModLoaderVersion  string
	ModName            string
}

type ModInfo struct {
	DisplayName        string
	InternalName       string
	Author             string
	Homepage           string
	Description        string
	Icon               string
	Version            string
	TModLoaderVersion  string
	LastUpdated        string
	ModDependencies    string
	ModSide            string
	DownloadLink       string
	DownloadsTotal     int
	DownloadsYesterday int
}

type ModVersionInfo struct {
	Version           string
	Downloads         int
	TModLoaderVersion string
	PublishDate       string
}

func GetHtml(url string) (*html.Node, error) {
	resp, err := http.Get(url)
	if err != nil {
		return nil, err
	}
	return html.Parse(resp.Body)
}

func GetNodesByTag(doc *html.Node, tag string) ([]*html.Node, error) {
	var Node []*html.Node
	var crawler func(*html.Node)
	crawler = func(node *html.Node) {
		if node.Type == html.ElementNode && node.Data == tag {
			Node = append(Node, node)
		}
		for child := node.FirstChild; child != nil; child = child.NextSibling {
			crawler(child)
		}
	}
	crawler(doc)
	if Node != nil {
		return Node, nil
	}
	return nil, fmt.Errorf("missing %s in the doc", tag)
}

func getNodeContent(node *html.Node) string {
	var ret string
	if node.Type == html.TextNode {
		ret += node.Data
	}
	for child := node.FirstChild; child != nil; child = child.NextSibling {
		ret += getNodeContent(child)
	}
	return ret
}

func getModHistory(modName string) ([]ModVersionInfo, error) {
	doc, err := GetHtml("http://javid.ddns.net/tModLoader/tools/moddownloadhistory.php?modname=" + modName)
	if err != nil {
		return nil, err
	}
	tBody, err := GetNodesByTag(doc, "tbody")
	if err != nil {
		return nil, err
	}
	table, err := GetNodesByTag(tBody[0], "tr")
	if err != nil {
		return nil, err
	}
	var ModHistory []ModVersionInfo = make([]ModVersionInfo, 0)
	for _, v := range table[1:] {
		tds, err := GetNodesByTag(v, "td")
		if err != nil {
			return nil, err
		}
		downloads, err := strconv.Atoi(getNodeContent(tds[1]))
		if err != nil {
			return nil, err
		}
		ModHistory = append(ModHistory, ModVersionInfo{
			Version:           getNodeContent(tds[0]),
			Downloads:         downloads,
			TModLoaderVersion: getNodeContent(tds[2]),
			PublishDate:       getNodeContent(tds[3]),
		})
	}
	return ModHistory, nil
}

func getModInfo(modName string) (*ModInfo, error) {
	var result ModInfo

	type JavidModInfoResponse struct {
		DisplayName      string
		Name             string
		Version          string
		Author           string
		Download         string
		Downloads        int
		Hot              int
		UpdateTimeStamp  string
		Modloaderversion string
		Modreferences    string
		Modside          string
	}

	var javidModInfoResponse JavidModInfoResponse
	resp, err := http.Get("http://javid.ddns.net/tModLoader/tools/modinfo.php?modname=" + modName)
	if err != nil {
		return nil, err
	}
	err = json.NewDecoder(resp.Body).Decode(&javidModInfoResponse)
	if err != nil {
		return nil, errors.New("invalid modname")
	}

	result.DisplayName = javidModInfoResponse.DisplayName
	result.InternalName = javidModInfoResponse.Name
	result.Version = javidModInfoResponse.Version
	result.Author = javidModInfoResponse.Author
	result.DownloadLink = javidModInfoResponse.Download
	result.DownloadsTotal = javidModInfoResponse.Downloads
	result.DownloadsYesterday = javidModInfoResponse.Hot
	result.LastUpdated = javidModInfoResponse.UpdateTimeStamp
	result.TModLoaderVersion = javidModInfoResponse.Modloaderversion
	result.ModDependencies = javidModInfoResponse.Modreferences
	result.ModSide = javidModInfoResponse.Modside

	type DescriptionResponse struct {
		Homepage    string
		Description string
	}

	resp, err = http.PostForm("http://javid.ddns.net/tModLoader/moddescription.php", url.Values{
		"modname": {modName},
	})
	if err != nil {
		return nil, err
	}
	var descriptionResponse DescriptionResponse
	err = json.NewDecoder(resp.Body).Decode(&descriptionResponse)
	if err != nil {
		return nil, err
	}

	result.Homepage = descriptionResponse.Homepage
	result.Description = descriptionResponse.Description

	resp, err = http.Get("https://mirror.sgkoi.dev/direct/" + modName + ".png")
	if err != nil {
		return nil, err
	}
	if resp.StatusCode == http.StatusNotFound {
		result.Icon = ""
	} else {
		result.Icon = "https://mirror.sgkoi.dev/direct/" + modName + ".png"
	}

	return &result, nil
}

func GetModList() ([]ListModInfo, error) {
	doc, err := GetHtml("http://javid.ddns.net/tModLoader/modmigrationprogress.php")
	if err != nil {
		return nil, err
	}
	tBody, err := GetNodesByTag(doc, "tbody")
	if err != nil {
		return nil, err
	}
	table, err := GetNodesByTag(tBody[0], "tr")
	if err != nil {
		return nil, err
	}
	var modList []ListModInfo = make([]ListModInfo, 0)
	for _, v := range table[1:] {
		tds, err := GetNodesByTag(v, "td")
		if err != nil {
			return nil, err
		}
		downloadsToday, err := strconv.Atoi(getNodeContent(tds[1]))
		if err != nil {
			return nil, err
		}
		downloadsYesterday, err := strconv.Atoi(getNodeContent(tds[2]))
		if err != nil {
			return nil, err
		}
		modList = append(modList, ListModInfo{
			DisplayName:        getNodeContent(tds[0]),
			DownloadsToday:     downloadsToday,
			DownloadsYesterday: downloadsYesterday,
			TModLoaderVersion:  getNodeContent(tds[3]),
			ModName:            getNodeContent(tds[4]),
		})
	}
	NameDownloadMap, err := GetDownloadsTotalMap()
	if err != nil {
		return nil, err
	}
	for i, v := range modList {
		modList[i].DownloadsTotal = NameDownloadMap[v.DisplayName].DownloadsTotal
		modList[i].Rank = NameDownloadMap[v.DisplayName].Rank
	}
	return modList, nil
}

type RankDownloadTotalInfo struct {
	Rank           int
	DownloadsTotal int
}

func GetDownloadsTotalMap() (map[string]RankDownloadTotalInfo, error) {
	doc, err := GetHtml("http://javid.ddns.net/tModLoader/modmigrationprogressalltime.php")
	if err != nil {
		return nil, err
	}
	tBody, err := GetNodesByTag(doc, "tbody")
	if err != nil {
		return nil, err
	}
	table, err := GetNodesByTag(tBody[0], "tr")
	if err != nil {
		return nil, err
	}
	var NameDownloadMap map[string]RankDownloadTotalInfo = make(map[string]RankDownloadTotalInfo)
	for _, v := range table[1:] {
		tds, err := GetNodesByTag(v, "td")
		if err != nil {
			return nil, err
		}
		rank, err := strconv.Atoi(getNodeContent(tds[0]))
		if err != nil {
			return nil, err
		}
		downloads, err := strconv.Atoi(getNodeContent(tds[2]))
		if err != nil {
			return nil, err
		}
		NameDownloadMap[getNodeContent(tds[1])] = RankDownloadTotalInfo{Rank: rank, DownloadsTotal: downloads}
	}
	return NameDownloadMap, nil
}
