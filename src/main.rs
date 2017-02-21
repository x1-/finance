extern crate chrono;
extern crate csv;
extern crate env_logger;
extern crate hyper;
extern crate hyper_openssl;
extern crate regex;
extern crate rustc_serialize;
extern crate time;

use std::str::FromStr;
use chrono::prelude::*;
use regex::Regex;
use regex::Match;
use time::Duration;

mod api_client;

//static URL: &'static str = "https://www.google.co.jp/";
//static url_base: &'static str = "http://www.google.com/finance/getprices?p={term}&f=d,h,o,l,c,v&i={tick}&x={market}&q={code}";

static INTERVAL: i64 = 86400;

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


    let splitter_reg = Regex::new( r"TIMEZONE_OFFSET=\d+\n" ).unwrap();

    let client = api_client::Ssl::new();

    let mut file = csv::Reader::from_file("./data/stocks.csv").unwrap();

    for r in file.decode() {
        let r: Record = r.unwrap();
        println!("({}, {}): {}", r.market, r.code, r.name);

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

        // let idata: Vec<_> = (0..).zip(data.iter().flat_map(|x|x.iter())).collect();
        for row in data.unwrap() {

            let ref date = row.date;
            let new_date: String = match calc_time( date, INTERVAL, base_time ) {
                Ok( t ) => {
                    if date.starts_with( "a" ) {
                        base_time = Some( t );
                    };
                    t.format("%Y-%m-%d %H:%M:%S").to_string()
                },
                Err( e ) => {
                    println!( "cannot get datetime: {}", e );
                    "".to_string()
                }
            };

            println!( "new_date: {}", new_date );
            let new_row = CsvRow {
                date  : new_date,
                close : row.close,
                high  : row.high,
                low   : row.low,
                open  : row.open,
                volume: row.volume
            };
            new_data.push( new_row );
        };

        let diff = if new_data.first().is_some() && new_data.last().is_some() {
            let before = new_data.first().unwrap().close;
            let now = new_data.last().unwrap().close;
            ( now - before ) / before
        };

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

fn columns(s: &str) -> Vec<&str> {
    let re = Regex::new( r"COLUMNS=([a-zA-Z,]+)" ).unwrap();
    let caps = re.captures( s ).unwrap();
    let cols = caps.get(1).map_or( "", |x| x.as_str());
    return cols.split( "," ).collect::<Vec<&str>>();
}

fn transform_csv(data: &str) -> Option<Vec<CsvRow>> {
    let mut rdr = csv::Reader::from_string( data )
                              .has_headers(false);
    rdr.decode().collect::<csv::Result<Vec<CsvRow>>>().ok()
}

fn local_time(s: &str) -> Result<DateTime<Local>, String> {
    match s.parse::<i64>().map( |t| Local.timestamp( t, 0 ) ) {
        Ok(t) => Ok( t ),
        Err(e) => Err( format!("cannot parse to i64: {}, because of {}", s, e) )
    }
}

fn calc_time(raw_value: &String, interval: i64, base_time: Option<DateTime<Local>>) -> Result<DateTime<Local>, String> {
    if raw_value.starts_with( "a" ) {
        let (_, chars) = raw_value.split_at(1);
        return local_time( chars );
    }
    raw_value.parse::<i64>()
        .map_err( |e| format!("cannot parse to i64: {}, because of {}", raw_value, e) )
        .and_then( |x| {
            match base_time {
                Some(t) => Ok( t + Duration::seconds( x * interval ) ),
                None    => Err( "base_time is maybe None".to_string() )
            }
        })
}
