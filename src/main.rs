extern crate chrono;
extern crate csv;
extern crate env_logger;
extern crate hyper;
extern crate hyper_openssl;
extern crate regex;
extern crate rustc_serialize;
extern crate time;

#[macro_use]
extern crate lazy_static;

use std::str::FromStr;
use chrono::prelude::*;
use regex::Regex;
use regex::Match;
use time::Duration;

mod api_client;

//static URL: &'static str = "https://www.google.co.jp/";
//static url_base: &'static str = "http://www.google.com/finance/getprices?p={term}&f=d,h,o,l,c,v&i={tick}&x={market}&q={code}";

static INTERVAL: i64 = 86400;
const TIME_FORMAT: &'static str = "%Y-%m-%d %H:%M:%S";

lazy_static! {
    static ref splitter_reg: Regex = Regex::new( r"TIMEZONE_OFFSET=\d+\n" ).unwrap();
}

#[derive(RustcDecodable)]
struct Record {
    code: String,
    name: String,
    market: String
}

#[derive(RustcDecodable)]
struct CsvRow {
    date  : String,
    close : f32,
    high  : f32,
    low   : f32,
    open  : f32,
    volume: u64
}

fn main() {

    // let url = match env::args().nth(1) {
    //     Some(url) => url,
    //     None => {
    //         println!("Usage: client <url>");
    //         return;
    //     }
    // };

    // let client = match env::var("HTTP_PROXY") {
    //     Ok(mut proxy) => {
    //         let mut port = 80;
    //         if let Some(colon) = proxy.rfind(':') {
    //             port = proxy[colon + 1..].parse().unwrap_or_else(|e| {
    //                 panic!("HTTP_PROXY is malformed: {:?}, port parse error: {}", proxy, e);
    //             });
    //             proxy.truncate(colon);
    //         }
    //         Client::with_http_proxy(proxy, port)
    //     },
    //     _ => Client::new()
    // };

    // let mut res = client.get(URL)
    //     .header(Connection::close())
    //     .send().unwrap();

    // println!("Response: {}", res.status);
    // println!("Headers:\n{}", res.headers);
    // io::copy(&mut res, &mut io::stdout()).unwrap();

    let tmp2 = DateTime::parse_from_str("2017-01-01 09:00:00 +09:00", "%Y-%m-%d %H:%M:%S %z");
    println!( "{}", tmp2.ok().unwrap().format("%s").to_string() );

    // let tmp = DateTime::parse_from_str("1486015200", "%s");
    let tmp = time::strptime("1483228800", "%s");;
    println!( "{:?}", tmp.ok() );

    let tmp3 = Local.timestamp( "1486015200".parse::<i64>().unwrap(), 0 );
    println!( "{:?}", tmp3 );
    println!( "{}", tmp3.format("%Y-%m-%d %H:%M:%S").to_string() );

    let tmp4 = tmp3 + Duration::seconds( 86400 );
    println!( "{:?}", tmp4 );
    println!( "{}", tmp4.format("%Y-%m-%d %H:%M:%S").to_string() );



    let client = api_client::Ssl::new();

    let mut file = csv::Reader::from_file("./data/stocks.csv").unwrap();

    for r in file.decode() {
        let r: Record = r.unwrap();

        let url = format!(
            "http://www.google.com/finance/getprices?p={term}&f=d,h,o,l,c,v&i={tick}&x={market}&q={code}",
            term = "7d", tick = INTERVAL, market = "TYO", code = r.code );
        let res = &client.sync_get( &url );

        let data = splitter_reg.split( res ).last().and_then( transform_csv );
        if data.is_none() {
            break;
        };

        let mut new_data = Vec::new();
        let mut base_time: Option<DateTime<Local>> = None;

        for row in data.unwrap() {

            let ref date = row.date;
            if let Ok(new_date) = calc_time( date, INTERVAL, &mut base_time ) {
                println!("date: {}", new_date);
                let new_row = CsvRow {
                    date  : new_date,
                    close : row.close,
                    high  : row.high,
                    low   : row.low,
                    open  : row.open,
                    volume: row.volume
                };
                new_data.push( new_row );
            }
        };

        let rate = close_rate( &new_data );
        if let Ok(x) = rate {
            println!("rate: {}", x);
        }

// EXCHANGE%3DTYO
// MARKET_OPEN_MINUTE=540
// MARKET_CLOSE_MINUTE=900
// INTERVAL=86400
// COLUMNS=DATE,CLOSE,HIGH,LOW,OPEN,VOLUME
// DATA_SESSIONS=[MORNING,540,690],[AFTERNOON,750,900]
// DATA=
// TIMEZONE_OFFSET=540
// a1484632800,1104,1124,1103,1121,34500

        // let lines = res.split("\n");
        // for row in lines {
        //     println!( "{}", row );
        //     break;
        // }

        // println!( "{}", res );
    }
}

fn close(maybeRow: Option<&CsvRow>) -> Result<f32, String> {
    maybeRow.map( |r|r.close ).ok_or( "cannot get row or row.close".to_string() )
}

fn close_rate(vec: &Vec<CsvRow>) -> Result<f32, String> {
    let ( before, now ) = ( try!(close(vec.first())), try!(close(vec.last())) );
    if before == 0.0 {
        Err( "close value of first row is zero (0.0)".to_string() )
    } else {
        println!("close_rate: {}, {}", now, before);
        Ok( (now - before) / before )
    }
}

fn transform_csv(data: &str) -> Result<Vec<CsvRow>, String> {
    let mut rdr = csv::Reader::from_string( data )
                              .has_headers(false);
    rdr.decode().collect::<csv::Result<Vec<CsvRow>>>()
        .map_err( |e|e.to_string() )
}

fn data_to_struct(res: &str) -> Result<Vec<CsvRow>, String>{
    let mut new_data: Vec<CsvRow> = Vec::new();
    let mut base_time: Option<DateTime<Local>> = None;

    let data = splitter_reg.split( res ).last()
        .ok_or( "cannot get data section".to_string() )
        .and_then( transform_csv )?;

    for row in data {

        let ref date = row.date;
        if let Ok(new_date) = calc_time( date, INTERVAL, &mut base_time ) {
            println!("date: {}", new_date);
            let new_row = CsvRow {
                date  : new_date,
                close : row.close,
                high  : row.high,
                low   : row.low,
                open  : row.open,
                volume: row.volume
            };
            new_data.push( new_row );
        }
    };
    return new_data
}

fn local_time(s: &str) -> Result<DateTime<Local>, String> {
    match s.parse::<i64>().map( |t| Local.timestamp( t, 0 ) ) {
        Ok(t) => Ok( t ),
        Err(e) => Err( format!("cannot parse to i64: {}, because of {}", s, e) )
    }
}

fn calc_time(raw_value: &String, interval: i64, base_time: &mut Option<DateTime<Local>>) -> Result<String, String> {
    if raw_value.starts_with( "a" ) {
        let (_, chars) = raw_value.split_at(1);
        let time = local_time( chars );
        if let Ok(t) = time {
            *base_time = Some( t );
        }
        return time.map( |t| {
            t.format(TIME_FORMAT).to_string()
        });
    }

    let parsed = raw_value.parse::<i64>()
        .map_err( |e| format!("cannot parse to i64: {}, because of {}", raw_value, e) )?;

    let time = base_time.map( |t| {
        t + Duration::seconds( parsed * interval )
    } ).ok_or( "base_time is maybe None".to_string() )?;

    Ok( time.format(TIME_FORMAT).to_string() )
}
