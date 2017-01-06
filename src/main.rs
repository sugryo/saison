extern crate encoding;
extern crate url;
#[macro_use] extern crate hyper;
extern crate scraper;
#[macro_use] extern crate nickel;
extern crate rustc_serialize;
extern crate regex;
extern crate clap;
extern crate modifier;

use std::io::{Read, Write};
use encoding::{Encoding, EncoderTrap, DecoderTrap};
use encoding::all::WINDOWS_31J;
use url::percent_encoding::{percent_encode, percent_decode, DEFAULT_ENCODE_SET};
use url::Url;
use hyper::client;
use scraper::{Html, Selector};
use nickel::status::StatusCode;
use nickel::{Nickel, HttpRouter, MediaType, Request, Response, MiddlewareResult, QueryString, Action, Continue, Halt, NickelError};
use rustc_serialize::json;
use regex::Regex;
use clap::{Arg, App};
use modifier::Modifier;

#[derive(Debug, RustcDecodable, RustcEncodable)]
struct HBScrapingLocation {
    timetable_time: String,
    route: String,
    track: String,
    destination: String,
    location_stops_url: String,
    bus: String,
    information: String,
    timetable_url: String,
}

#[derive(Debug, RustcDecodable, RustcEncodable)]
struct HBRouteStop {
    stop_name: String,
    arrived_time: String,
}

#[derive(Debug, RustcDecodable, RustcEncodable)]
struct HBRouteInformation {
    route_name: String,
    starting_stop_name: String,
    ending_stop_name: String,
    stops: Vec<HBRouteStop>,
}

#[derive(Debug)]
struct StopId {
    left_stop: i32,
    arrived_stop: i32
}

fn get_stop_id(left_stop_name: &String, arrived_stop_name: &String) -> Result<StopId, String>{
    // Stop name encode from Shift-JIS to UTF-8
    let result_left_stop_vec = WINDOWS_31J.encode(left_stop_name, EncoderTrap::Strict);
    let result_arrived_stop_vec = WINDOWS_31J.encode(arrived_stop_name, EncoderTrap::Strict);
    
    if result_left_stop_vec.is_err() || result_arrived_stop_vec.is_err() { return Err("Stop Encode Error".to_string()) }
    let left_stop_vec = result_left_stop_vec.unwrap();
    let arrived_stop_vec = result_arrived_stop_vec.unwrap();
    
    // Stop name percent_encode
    let left_stop_percent_encoded = percent_encode(&left_stop_vec, DEFAULT_ENCODE_SET);
    let arrived_stop_percent_encoded = percent_encode(&arrived_stop_vec, DEFAULT_ENCODE_SET);

    // Hyper: Get Request http
    let client = client::Client::new();
    let search_stop_url = format!("http://hakobus.jp/search02.php?stopname_f={}&stopname_t={}", left_stop_percent_encoded, arrived_stop_percent_encoded);
    let mut search_stop_res = client.get(&search_stop_url).send().unwrap();
    ////let stop_id_url = "http://localhost:8080/";
    ////let mut stop_id_res = client.get(stop_id_url).send().unwrap();

    // Response encode from Shift-JIS to UTF-8
    let mut shift_jis_body = Vec::new();
    search_stop_res.read_to_end(&mut shift_jis_body).unwrap();

    let result_body = WINDOWS_31J.decode(&shift_jis_body, DecoderTrap::Strict);
    if result_body.is_err() { return Err("Responce Body Encode Error".to_string()) }
    let body = result_body.unwrap();
    
    // Scraper
    let fragment = Html::parse_fragment(&body);

    // html
    let html_selector = Selector::parse("html").unwrap();
    let html = fragment.select(&html_selector).next().unwrap();

    // div#container
    let div_id_container_selector = Selector::parse("div#container").unwrap();
    let div_id_container = html.select(&div_id_container_selector).next().unwrap();

    // div#contents
    let div_id_contents_selector = Selector::parse("div#contents").unwrap();
    let div_id_contents = div_id_container.select(&div_id_contents_selector).next().unwrap();

    // form[name=form1]
    let form_name_form1_selector = Selector::parse("form[name=form1]").unwrap();
    let form_name_form1 = div_id_contents.select(&form_name_form1_selector).next().unwrap();

    // table First
    let table_selector = Selector::parse("table").unwrap();
    let table = form_name_form1.select(&table_selector).nth(1).unwrap();

    // tbody
    let tbody_selector = Selector::parse("tbody").unwrap();
    let tbody = table.select(&tbody_selector).next().unwrap();

    // tr
    let tr_selector = Selector::parse("tr").unwrap();
    let tr = tbody.select(&tr_selector).nth(1).unwrap();

    // Left td and Arrived td
    let td_selector = Selector::parse("td").unwrap();
    let left_td = tr.select(&td_selector).nth(0).unwrap();
    let arrived_td = tr.select(&td_selector).nth(2).unwrap();

    // div[align=center]
    let div_align_center_selector = Selector::parse("div[align=center]").unwrap();
    let left_div_align_center = left_td.select(&div_align_center_selector).next().unwrap();
    let arrived_div_align_center = arrived_td.select(&div_align_center_selector).next().unwrap();

    // select[name=in]
    let select_name_in_selector = Selector::parse("select[name=in]").unwrap();
    let select_name_in = left_div_align_center.select(&select_name_in_selector).next().unwrap();

    // select[name=out]
    let select_name_out_selector = Selector::parse("select[name=out]").unwrap();
    let select_name_out = arrived_div_align_center.select(&select_name_out_selector).next().unwrap();

    // option
    let option_selector = Selector::parse("option").unwrap();
    let got_left_stop = select_name_in.select(&option_selector).next().unwrap();
    let got_arrived_stop = select_name_out.select(&option_selector).next().unwrap();

    // Gotten Website stop name
    let website_left_stop_name = got_left_stop.text().next().unwrap();
    let website_arrived_stop_name = got_arrived_stop.text().next().unwrap();

    if website_left_stop_name != left_stop_name {
        return Err("Left stop name doesn't exist".to_string())
    }
    
    if website_arrived_stop_name != arrived_stop_name {
        return Err("Arrived stop name doesn't exist".to_string())
    }
    Ok(
        StopId {
            left_stop: got_left_stop.value().attr("value").unwrap().parse::<i32>().unwrap(),
            arrived_stop: got_arrived_stop.value().attr("value").unwrap().parse::<i32>().unwrap(),
        }
    )
}

fn hello_world<'mw>(_req: &mut Request, res: Response<'mw>) -> MiddlewareResult<'mw> {
    res.send("Hello World")
}

fn get_locations<'mw>(req: &mut Request, mut response: Response<'mw>) -> MiddlewareResult<'mw> {
    // Percent Encoded Path decode to UTF-8
    let left_stop_percent_encoded = req.param("left_stop").unwrap();
    let left_stop_name = percent_decode(&left_stop_percent_encoded.as_bytes()).decode_utf8_lossy().to_string();

    let arrived_stop_percent_encoded = req.param("arrived_stop").unwrap();
    let arrived_stop_name = percent_decode(&arrived_stop_percent_encoded.as_bytes()).decode_utf8_lossy().to_string();
    
    //println!("乗車停留所：{}", left_stop_name);
    //println!("下車停留所：{}", arrived_stop_name);
    
    // Get Stop Id
    let stop_id = get_stop_id(&left_stop_name, &arrived_stop_name).unwrap();
    //let stop_id = get_stop_id(&"新川町".to_string(), &"五稜郭".to_string()).unwrap();
    //println!("{:?}", matz);
    
    // Hyper
    let client = client::Client::new();
    //let url = "http://localhost:8080/result.html";
    //let mut res = client.get(url).send().unwrap();

    let url = format!("http://hakobus.jp/result.php?in={}&out={}", stop_id.left_stop, stop_id.arrived_stop);
    let mut res = client.get(&url).send().unwrap();

    // Responce
    let mut shift_jis_body = Vec::new();
    res.read_to_end(&mut shift_jis_body).unwrap();

    let mut hb_scraping_locations = Vec::new();
    if let Ok(body) = WINDOWS_31J.decode(&shift_jis_body, DecoderTrap::Strict) {
        ////                        println!("{}", body);
        
        // Scraper
        let fragment = Html::parse_fragment(&body);
        
        // html
        let html_selector = Selector::parse("html").unwrap();
        let html = fragment.select(&html_selector).next().unwrap();

        // div#container
        let div_id_container_selector = Selector::parse("div#container").unwrap();
        let div_id_container = html.select(&div_id_container_selector).next().unwrap();

        // div#result
        let div_id_result_selector = Selector::parse("div#result").unwrap();
        let div_id_result = div_id_container.select(&div_id_result_selector).next().unwrap();

        // table
        let table_selector = Selector::parse("table").unwrap();
        let table = div_id_result.select(&table_selector).next().unwrap();

        // tbody
        let tbody_selector = Selector::parse("tbody").unwrap();
        let tbody = table.select(&tbody_selector).next().unwrap();

        // tr
        let tr_selector = Selector::parse("tr").unwrap();
        
        let mut locations = Vec::new();
        for location_row in tbody.select(&tr_selector) {
            let mut location = Vec::new();
            // td
            let td_selector = Selector::parse("td").unwrap();
            
            // 停留所時刻
            if let Some(timetable_time) = location_row.select(&td_selector).nth(1) {
                // div[align=center]
                let div_align_center_selector = Selector::parse("div[align=center]").unwrap();
                let div_align_center_option = timetable_time.select(&div_align_center_selector).next();
                if let Some(div_align_center) = div_align_center_option {
                    if let Some(timetable_text) = div_align_center.text().next() {
                        location.push(Some(timetable_text.to_string()));
                        //hb_scraping_location.set_timetable_time(timetable_text.to_string());
                    }
                }
            }

            // 系統
            if let Some(route) = location_row.select(&td_selector).nth(2) {
                // div[align=center]
                let div_align_center_selector = Selector::parse("div[align=center]").unwrap();
                let div_align_center_option = route.select(&div_align_center_selector).next();
                if let Some(div_align_center) = div_align_center_option {
                    if let Some(route_text) = div_align_center.text().next() {
                        location.push(Some(route_text.to_string()));
                        //hb_scraping_location.set_route(&route_text);
                    }
                }
            }

            // のりば
            if let Some(track) = location_row.select(&td_selector).nth(3) {
                // div[align=center]
                let div_align_center_selector = Selector::parse("div[align=center]").unwrap();
                let div_align_center_option = track.select(&div_align_center_selector).next();
                if let Some(div_align_center) = div_align_center_option {
                    if div_align_center.text().next().is_none() {
                        // a
                        let a_selector = Selector::parse("a").unwrap();
                        let a_option = div_align_center.select(&a_selector).next();
                        if let Some(a) = a_option {
                            let url = Url::parse("http://hakobus.jp").unwrap();
                            let path = a.value().attr("href").unwrap();
                            let url = url.join(path).unwrap();
                            location.push(Some(url.into_string()));
                            //hb_scraping_location.set_track(&url.into_string());
                        } else {
                            location.push(None);
                        }
                    } else {
                        location.push(None);
                    }
                }
            }

            // 行き先
            if let Some(destination) = location_row.select(&td_selector).nth(4) {
                // div[align=center]
                let div_align_center_selector = Selector::parse("div[align=center]").unwrap();
                let div_align_center_option = destination.select(&div_align_center_selector).next();
                if let Some(div_align_center) = div_align_center_option {
                    if let Some(destination_text) = div_align_center.text().next() {
                        location.push(Some(destination_text.to_string()));
                    }
                }
            }

            // 経路
            if let Some(bus_type) = location_row.select(&td_selector).nth(5) {
                // div[align=center]
                let div_align_center_selector = Selector::parse("div[align=center]").unwrap();
                let div_align_center_option = bus_type.select(&div_align_center_selector).next();
                if let Some(div_align_center) = div_align_center_option {
                    if div_align_center.text().next().is_none() {
                        // a
                        let a_selector = Selector::parse("a").unwrap();
                        let a_option = div_align_center.select(&a_selector).next();
                        if let Some(a) = a_option {
                            let url = Url::parse("http://hakobus.jp").unwrap();
                            let path = a.value().attr("href").unwrap();
                            let url = url.join(path).unwrap();
                            location.push(Some(url.into_string()));
                        }
                    }
                }
            }

            // 車両
            if let Some(bus_type) = location_row.select(&td_selector).nth(6) {
                // div[align=center]
                let div_align_center_selector = Selector::parse("div[align=center]").unwrap();
                let div_align_center_option = bus_type.select(&div_align_center_selector).next();
                if let Some(div_align_center) = div_align_center_option {
                    if let Some(bus_type_text) = div_align_center.text().next() {
                        location.push(Some(bus_type_text.to_string()));
                    } else {
                        location.push(None);
                    }
                }
            }
            
            // 運行状況
            if let Some(delay_information) = location_row.select(&td_selector).nth(7) {
                if let Some(delay_information_text) = delay_information.text().next() {
                    if location.contains(&Some(delay_information_text.to_string())) || delay_information_text == "*****"{
                        location.push(None);
                    } else {
                        location.push(Some(delay_information_text.to_string()));
                    }
                } else {
                    location.push(None);
                }
            }

            // 時刻表
            if let Some(timetable) = location_row.select(&td_selector).nth(8) {
                // div[align=center]
                let div_align_center_selector = Selector::parse("div[align=center]").unwrap();
                let div_align_center_option = timetable.select(&div_align_center_selector).next();
                if let Some(div_align_center) = div_align_center_option {
                    if div_align_center.text().next().is_none() {
                        // a
                        let a_selector = Selector::parse("a").unwrap();
                        let a_option = div_align_center.select(&a_selector).next();
                        if let Some(a) = a_option {
                            let url = Url::parse("http://hakobus.jp").unwrap();
                            let path = a.value().attr("href").unwrap();
                            let url = url.join(path).unwrap();
                            location.push(Some(url.into_string()));
                        }
                    }
                }
            }
            if location.is_empty() {
                location.clear();
                continue
            }
            locations.push(location);
        }
        for location in &locations {
            let track = location[2].clone().unwrap_or("不明".to_string());
            let bus = match location[5].clone() {
                None => "標準のバス".to_string(),
                Some(v) => v,
            };
            let information = location[6].clone().unwrap_or("不明".to_string());
            let hb_scraping_location = HBScrapingLocation {
                timetable_time: location[0].clone().unwrap(),
                route: location[1].clone().unwrap(),
                track: track.to_string(),
                destination: location[3].clone().unwrap(),
                location_stops_url: location[4].clone().unwrap(),
                bus: bus,
                information: information,
                timetable_url: location[7].clone().unwrap(),
            };
            hb_scraping_locations.push(hb_scraping_location);
        }
        //println!("{:?}", &hb_scraping_locations);
    }
    let hb_scraping_locations_json_encoded = json::encode(&hb_scraping_locations).unwrap();
    response.set(MediaType::Json);
    response.send(hb_scraping_locations_json_encoded)
}

fn get_location<'mw>(request: &mut Request, mut response: Response<'mw>) -> MiddlewareResult<'mw> {
    let hakobus_location_url = request.query().get("url").unwrap();
    ////let hakobus_location_url = "http://localhost:8080/location.html"

    // Hyper: Get Request http
    let client = client::Client::new();
    let mut hakobus_location_res = client.get(hakobus_location_url).send().unwrap();
    ////let mut hakobus_location_res = client.get(hakobus_location_url).send().unwrap();

    // Response encode from Shift-JIS to UTF-8
    let mut shift_jis_body = Vec::new();
    hakobus_location_res.read_to_end(&mut shift_jis_body).unwrap();

    let result_body = WINDOWS_31J.decode(&shift_jis_body, DecoderTrap::Strict);
    let body = result_body.unwrap();

    // Scraper
    let fragment = Html::parse_fragment(&body);

    // html
    let html_selector = Selector::parse("html").unwrap();
    let html = fragment.select(&html_selector).next().unwrap();

    // div#container
    let div_id_container_selector = Selector::parse("div#container").unwrap();
    let div_id_container = html.select(&div_id_container_selector).next().unwrap();

    // div#contents
    let div_id_contents_selector = Selector::parse("div#contents").unwrap();
    let div_id_contents = div_id_container.select(&div_id_contents_selector).next().unwrap();

    // div#route
    let div_id_route_selector = Selector::parse("div#route").unwrap();
    let div_id_route = div_id_contents.select(&div_id_route_selector).next().unwrap();

    // h3
    let h3_selector = Selector::parse("h3").unwrap();
    let h3 = div_id_route.select(&h3_selector).next().unwrap();
    let h3_text = h3.text().next().unwrap();

    // Regex
    let re = Regex::new(r"^系統名：(.*)　(.*)→(.*)").unwrap();
    let cap = re.captures(h3_text).unwrap();
    
    // Extract the route information
    let route_name = cap.at(1).unwrap().trim();
    let starting_station_name = cap.at(2).unwrap().trim();
    let ending_station_name = cap.at(3).unwrap().trim();

    // font
    let font_selector = Selector::parse("font").unwrap();
    let mut route_stops = Vec::new();
    for route_information in div_id_route.select(&font_selector) {
        // p
        let p_selector = Selector::parse("p").unwrap();
        let p = route_information.select(&p_selector).next().unwrap();
        let p_text = p.text().next().unwrap();

        // Regex: Split the stop and time in p_text
        let re = Regex::new(r"(\s*(?P<stopname>.*)\s*)　（(?P<arrivedname>.*)）").unwrap();
        let cap = re.captures(p_text).unwrap();

        // Extract the stop and arrived time
        let stop_name = cap.name("stopname").unwrap().trim();
        let arrived_time = cap.name("arrivedname").unwrap().trim();
        println!("０番目：{}", cap.at(0).unwrap_or(""));
        println!("１番目：{}", cap.at(1).unwrap_or(""));
        println!("２番目：{}", cap.at(2).unwrap_or(""));
        
        println!("{}", p_text);
        route_stops.push(HBRouteStop {
            stop_name: stop_name.to_string(),
            arrived_time: arrived_time.to_string(),
        });
    }
    println!("{:?}", route_stops);
    let route_information = HBRouteInformation {
        route_name: route_name.to_string(),
        starting_stop_name: starting_station_name.to_string(),
        ending_stop_name: ending_station_name.to_string(),
        stops: route_stops,
    };
    let route_information_json_encoded = json::encode(&route_information).unwrap();
    
    response.set(MediaType::Json);
    //response.send(json::encode(&"http://hakobus.jp").unwrap())
    response.send(route_information_json_encoded)
}

#[derive(Debug, RustcEncodable)]
struct Error {
    code: i32,
    message: String,
}

#[derive(Debug, RustcEncodable)]
struct Errors{
    errors: Vec<Error>
}

fn not_found<'mw>(err: &mut NickelError, _req: &mut Request) -> Action {
    if let Some(ref mut res) = err.stream {
        if res.status() == StatusCode::NotFound {
            let error = Error {
                code: 1,
                message: "要求されたルーティングは存在しません。".to_string(),
            };
            let errors = Errors {
                errors: vec![error],
            };
            res.write_all(json::encode(&errors).unwrap().as_bytes());
            return Halt(())
        }
    }

    Continue(())
}

fn bad_request<'mw>(err: &mut NickelError, _req: &mut Request) -> Action {
    if let Some(ref mut res) = err.stream {
        if res.status() == StatusCode::BadRequest {
            let error = Error {
                code: 2,
                message: "要求の形式が正しくありません。".to_string(),
            };
            let errors = Errors {
                errors: vec![error],
            };
            res.write_all(json::encode(&errors).unwrap().as_bytes());
            return Halt(())
        }
    }

    Continue(())
}

fn enable_mediatype_json<'mw>(_req: &mut Request, mut res: Response<'mw>) -> MiddlewareResult<'mw> {
    res.set(MediaType::Json);
    res.next_middleware()
}

impl<'a, D> Modifier<nickel::Response<'a, D>> for XContentTypeOptions {
    fn modify(self, res: &mut nickel::Response<'a, D>) {
        res.headers_mut().set(self)
    }
}

header! { (XContentTypeOptions, "X-Content-Type-Options") => [String] }
fn enable_header_xcontenttypeoptions<'mw>(_req: &mut Request, mut res: Response<'mw>) -> MiddlewareResult<'mw> {
    //let mut headers = Headers::new();
    //headers.set(XContentTypeOptions("nosniff".to_owned()));
    //headers.set_raw("X-Content-Type-Options", vec![b"nosniff".to_vec()]);
    res.set(XContentTypeOptions("nosniff".to_owned()));
    res.next_middleware()
}

fn main() {
    // Clap
    let matches = App::new("Saison")
        .version("0.0.1")
        .author("Ryo Sugimoto <sugryo1109@gmail.com>")
        .arg(Arg::with_name("port")
             .short("p")
             .long("port")
             .value_name("PORT")
             .help("Set a port")
             .takes_value(true))
        .get_matches();
    let port = matches.value_of("port").unwrap_or("6767");
    let listen = format!("localhost:{}", port);

    // Nickel
    let mut server = Nickel::new();
    server.utilize(enable_mediatype_json);
    server.utilize(enable_header_xcontenttypeoptions);
    server.get("/hello_world", hello_world);
    server.get("/locations/:left_stop/to/:arrived_stop", get_locations);
    server.get("/location", get_location);

    // Handle Error
    let bad_request_400: fn(&mut NickelError, &mut Request) -> Action = bad_request;
    server.handle_error(bad_request_400);
    let not_found_404: fn(&mut NickelError, &mut Request) -> Action = not_found;
    server.handle_error(not_found_404);
    server.listen(listen.as_str());
}
