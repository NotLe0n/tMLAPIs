package main

import (
	"encoding/json"
	"errors"
	"net/http"
	"os"
	"strconv"
)

type steamAcc struct {
	Steamid                  string
	Communityvisibilitystate int
	Profilestate             int
	Personaname              string
	Profileurl               string
	Avatar                   string
	Avatarmedium             string
	Avatarfull               string
	Avatarhash               string
	Lastlogoff               int
	Personastate             int
	Primaryclanid            string
	Timecreated              int
	Personastateflags        int
	Loccountrycode           string
}

var mySecret = os.Getenv("steamAPIKey")

type AuthorModStats struct {
	RankTotal          int
	DisplayName        string
	DownloadsTotal     int
	DownloadsYesterday int
}

type AuthorMaintainedMod struct {
	ModName            string
	DownloadsTotal     int
	DownloadsYesterday int
}

type Author struct {
	SteamName          string
	DownloadsTotal     int
	DownloadsYesterday int
	Mods               []AuthorModStats
	MaintainedMods     []AuthorMaintainedMod
}

func getSteamJson(steamId string) (*steamAcc, error) {
	r, err := http.Get("https://api.steampowered.com/ISteamUser/GetPlayerSummaries/v2/?key=" + mySecret + "&steamids=" + steamId)
	if err != nil {
		return nil, err
	}
	defer r.Body.Close()

	type resp struct {
		Response struct {
			Players []steamAcc
		}
	}

	var res resp
	err = json.NewDecoder(r.Body).Decode(&res)
	if err != nil {
		return nil, err
	}
	if len(res.Response.Players) == 0 {
		return nil, errors.New("please enter a valid steamid64")
	}
	return &res.Response.Players[0], nil
}

func GetAuthorStats(steamId string) (*Author, error) {
	steam, err := getSteamJson(steamId)
	if err != nil {
		return nil, err
	}
	doc, err := GetHtml("http://javid.ddns.net/tModLoader/tools/ranksbysteamid.php?steamid64=" + steamId)
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
	var modStats []AuthorModStats = make([]AuthorModStats, 0)
	for _, v := range table[1:] {
		tds, err := GetNodesByTag(v, "td")
		if err != nil {
			return nil, err
		}
		rankTotal, err := strconv.Atoi(getNodeContent(tds[0]))
		if err != nil {
			return nil, err
		}
		downloadsTotal, err := strconv.Atoi(getNodeContent(tds[2]))
		if err != nil {
			return nil, err
		}
		downloadsYesterday, err := strconv.Atoi(getNodeContent(tds[3]))
		if err != nil {
			return nil, err
		}
		modStats = append(modStats, AuthorModStats{
			RankTotal:          rankTotal,
			DisplayName:        getNodeContent(tds[1]),
			DownloadsTotal:     downloadsTotal,
			DownloadsYesterday: downloadsYesterday,
		})
	}
	table, err = GetNodesByTag(tBody[3], "tr")
	if err != nil {
		return nil, err
	}
	var maintainedMods []AuthorMaintainedMod = make([]AuthorMaintainedMod, 0)
	for _, v := range table[1:] {
		tds, err := GetNodesByTag(v, "td")
		if err != nil {
			return nil, err
		}
		downloadsTotal, err := strconv.Atoi(getNodeContent(tds[1]))
		if err != nil {
			return nil, err
		}
		downloadsYesterday, err := strconv.Atoi(getNodeContent(tds[2]))
		if err != nil {
			return nil, err
		}
		maintainedMods = append(maintainedMods, AuthorMaintainedMod{
			ModName:            getNodeContent(tds[0]),
			DownloadsTotal:     downloadsTotal,
			DownloadsYesterday: downloadsYesterday,
		})
	}
	body, err := GetNodesByTag(doc, "body")
	if err != nil {
		return nil, err
	}
	brs, err := GetNodesByTag(body[0], "br")
	if err != nil {
		return nil, err
	}
	downloadsTotal, err := strconv.Atoi(getNodeContent(brs[0].NextSibling)[len("Total Downloads: "):])
	if err != nil {
		return nil, err
	}
	downloadsYesterday, err := strconv.Atoi(getNodeContent(brs[1].NextSibling)[len("Yesterday Downloads: "):])
	if err != nil {
		return nil, err
	}
	return &Author{
		SteamName:          steam.Personaname,
		DownloadsTotal:     downloadsTotal,
		DownloadsYesterday: downloadsYesterday,
		Mods:               modStats,
		MaintainedMods:     maintainedMods,
	}, nil
}
