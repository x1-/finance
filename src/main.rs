extern crate chrono;
extern crate csv;
extern crate env_logger;
extern crate hyper;
extern crate hyper_openssl;
extern crate regex;
extern crate rustc_serialize;

use std::str::FromStr;
use chrono::*;
use regex::Regex;
use regex::Match;

mod api_client;

//static URL: &'static str = "https://www.google.co.jp/";
//static url_base: &'static str = "http://www.google.com/finance/getprices?p={term}&f=d,h,o,l,c,v&i={tick}&x={market}&q={code}";

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

    
//    DateTime::parse_from_str("2014-11-28 21:00:09 +09:00", "%Y-%m-%d %H:%M:%S %z")
    let splitter_reg = Regex::new( r"TIMEZONE_OFFSET=\d+\n" ).unwrap();

    let client = api_client::Ssl::new();
    // let res = client.sync_get( URL );
    // println!( "{}", res );

    let mut file = csv::Reader::from_file("./data/stocks.csv").unwrap();

    for r in file.decode() {
        let r: Record = r.unwrap();
        println!("({}, {}): {}", r.market, r.code, r.name);
        let url = format!(
            "http://www.google.com/finance/getprices?p={term}&f=d,h,o,l,c,v&i={tick}&x={market}&q={code}",
            term = "7d", tick = 86400, market = "TYO", code = r.code );
        let res = &client.sync_get( &url );
        let len = res.len();
        let cols = columns( res );
        // println!( "{:?}", cols );

        let nc = |mt: Match|{
            println!( "{}", mt.end() );
            mt.end()
        };
        let mat = splitter_reg.find( res );
        let start = mat.map_or( len, |x| x.end() );
        // println!( "{}, {}", start, len );
        // let caps = timeRe.captures( res ).unwrap();
        // let t = caps.at(1).unwrap();
        let base_time = | mut rows: Vec<CsvRow>, index: usize | {
            let ref mut row = rows[index];
            let first = row.date.pop();
            if Some( 'a' ).eq( &first ) {
                return "";
            }
            return "";
        };
        let data = splitter_reg.split( res ).last().and_then( transform_csv );
        let ref r = data.unwrap()[0];
        println!( "head: {},{}",
                   r.date,
                   r.close
        );

        // match maybe_csv {
        //     Some( csv ) => {
        //     }
        // }

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

