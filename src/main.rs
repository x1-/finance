extern crate csv;
extern crate env_logger;
extern crate hyper;
extern crate hyper_openssl;
extern crate rustc_serialize;


mod api_client;

//static URL: &'static str = "https://www.google.co.jp/";
//static url_base: &'static str = "http://www.google.com/finance/getprices?p={term}&f=d,h,o,l,c,v&i={tick}&x={market}&q={code}";

#[derive(RustcDecodable)]
struct Record {
    code: String,
    name: String,
    market: String
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

    let client = api_client::Ssl::new();
    // let res = client.sync_get( URL );
    // println!( "{}", res );

    let mut file = csv::Reader::from_file("./data/stocks.csv").unwrap();
    for r in file.decode().take(10) {
        let r: Record = r.unwrap();
        println!("({}, {}): {}", r.market, r.code, r.name);
        let url = format!(
            "http://www.google.com/finance/getprices?p={term}&f=d,h,o,l,c,v&i={tick}&x={market}&q={code}",
            term = "1d", tick = 60, market = "TYO", code = r.code );
        let res = &client.sync_get( &url );
        println!( "{}", res );
    }
}
