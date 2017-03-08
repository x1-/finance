extern crate chrono;
extern crate csv;
extern crate env_logger;
extern crate hyper;
extern crate hyper_openssl;
extern crate regex;
extern crate rustc_serialize;
extern crate slack_hook;
extern crate time;

#[macro_use]
extern crate lazy_static;

use chrono::prelude::*;
use regex::Regex;
use slack_hook::{Slack, Payload, PayloadBuilder};
use time::Duration;

mod api_client;

static INTERVAL: i64 = 86400;
const TIME_FORMAT: &'static str = "%Y-%m-%d %H:%M:%S";
const URL_BASE: &'static str = "http://www.google.com/finance/getprices";

const SLACK_WEBHOOK: &'static str = "";

lazy_static! {
    static ref SPLITTER_REG: Regex = Regex::new( r"TIMEZONE_OFFSET=\d+\n" ).unwrap();
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

    let client = api_client::Ssl::new();
    let slack = Slack::new(SLACK_WEBHOOK).unwrap();

    let mut file = csv::Reader::from_file("./data/stocks.csv").unwrap();

    for r in file.decode() {
        let r: Record = r.unwrap();

        let url = format!(
            "{uri}?f=d,h,o,l,c,v&p={term}&i={tick}&x={market}&q={code}",
            uri = URL_BASE,
            term = "7d", tick = INTERVAL, market = "TYO", code = r.code );

        let res = &client.sync_get( &url );

        let rdata = data_to_struct( res );
        let ratex = rdata.and_then( |data| close_rate( &data ) );

        match ratex {
            Ok((rate, x)) if rate >= 0.1 || rate < -0.1 => {
                let payload = slack_payload( r.code, r.name, x, rate );
                slack.send( &payload );
                println!("payload: {:?}", payload);
            },
            _ => println!( "rate is greater than -10% or less than 10%." )
        }
    }
}

fn close(maybe_row: Option<&CsvRow>) -> Result<f32, String> {
    maybe_row.map( |r|r.close ).ok_or( "cannot get row or row.close".to_string() )
}

fn close_rate(vec: &Vec<CsvRow>) -> Result<(f32, f32), String> {
    let ( before, now ) = ( try!(close(vec.first())), try!(close(vec.last())) );
    if before == 0.0 {
        Err( "close value of first row is zero (0.0)".to_string() )
    } else {
        Ok( ( (now - before) / before, now ) )
    }
}

fn transform_csv(data: &str) -> Result<Vec<CsvRow>, String> {
    let mut rdr = csv::Reader::from_string( data )
                              .has_headers(false);
    rdr.decode().collect::<csv::Result<Vec<CsvRow>>>()
        .map_err( |e|e.to_string() )
}

fn data_to_struct(res: &str) -> Result<Vec<CsvRow>, String> {
    let mut new_data: Vec<CsvRow> = Vec::new();
    let mut base_time: Option<DateTime<Local>> = None;

    let data = SPLITTER_REG.split( res ).last()
        .ok_or( "cannot get data section".to_string() )
        .and_then( transform_csv )?;

    for row in data {

        let ref date = row.date;
        if let Ok(new_date) = calc_time( date, INTERVAL, &mut base_time ) {
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
    return Ok(new_data);
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

fn slack_payload(code: String, name: String, price: f32, rate: f32) -> Payload {
    let emoji = if rate > 0.0 { ":chart_with_upwards_trend:" } else { ":chart_with_downwards_trend:" };

    let message = format!("Now, {code}:{name} price {price} ( change rate is {rate} from last week ).",
                          code = code,
                          name = name,
                          price = price,
                          rate = rate);

    let p = PayloadBuilder::new()
        .text( message )
        .channel("#kabu-notice")
        .username("kabu-bot")
        .icon_emoji(emoji)
        .build()
        .unwrap();
    p
}
