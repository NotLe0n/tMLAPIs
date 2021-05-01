package main

import (
	"context"
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"sync"
)

//write an json error to w
func errorJson(w http.ResponseWriter, msg string, code int) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(code)
	json.NewEncoder(w).Encode(`{"error":"` + msg + `"}`)
}

//send a response with a json body constructed from data over w
func returnJsonFromStruct(w http.ResponseWriter, data interface{}, code int) {
	w.Header().Set("Content-Type", "application/json")
	w.WriteHeader(code)
	json.NewEncoder(w).Encode(data)
}

var wg sync.WaitGroup

var serverHandler *http.ServeMux
var server http.Server

func authorApiHandler(w http.ResponseWriter, r *http.Request) {
	log.Println("Got a request on /author_api/")
	if r.Method != http.MethodGet {
		errorJson(w, "Request must be GET", http.StatusBadRequest)
		return
	}
	var steamId string = r.URL.Path[len("/author_api/"):]
	authorStats, err := GetAuthorStats(steamId)
	if err != nil {
		log.Println(err)
		errorJson(w, err.Error(), http.StatusInternalServerError)
		return
	}
	returnJsonFromStruct(w, authorStats, http.StatusOK)
}

func modListHandler(w http.ResponseWriter, r *http.Request) {
	log.Println("Got a request on /modList")
	if r.Method != http.MethodGet {
		errorJson(w, "Request must be GET", http.StatusBadRequest)
		return
	}
	modList, err := GetModList()
	if err != nil {
		log.Panicln(err)
		errorJson(w, err.Error(), http.StatusInternalServerError)
		return
	}
	returnJsonFromStruct(w, modList, http.StatusOK)
}

func main() {
	serverHandler = http.NewServeMux()
	server = http.Server{Addr: ":3000", Handler: serverHandler}

	serverHandler.HandleFunc("/author_api/", authorApiHandler)
	serverHandler.HandleFunc("/modList", modListHandler)

	wg.Add(1)
	go func() {
		defer wg.Done() //tell the waiter group that we are finished at the end
		cmdInterface()
		log.Println("cmd goroutine finished")
	}()

	log.Println("server starting on Port 3000")
	if err := server.ListenAndServe(); err != nil && err != http.ErrServerClosed {
		log.Fatal(err.Error())
	} else if err == http.ErrServerClosed {
		log.Println("Server not listening anymore")
	}

	wg.Wait()
}

func cmdInterface() {
	for loop := true; loop; {
		var inp string
		_, err := fmt.Scanln(&inp)
		if err != nil {
			log.Println(err.Error())
		} else {
			switch inp {
			case "quit":
				log.Println("Attempting to shutdown server")
				err := server.Shutdown(context.Background())
				if err != nil {
					log.Fatal("Error while trying to shutdown server: " + err.Error())
				}
				log.Println("Server was shutdown")
				loop = false
			default:
				fmt.Println("cmd not supported")
			}
		}
	}
}
