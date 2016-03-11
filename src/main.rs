extern crate solicit;
use solicit::http::client::CleartextConnector;
use solicit::client::SimpleClient;
use std::str;

// extern crate tendril;
// extern crate html5ever;

// use std::io::{self, Write};
// use std::default::Default;
// use tendril::{ByteTendril, ReadExt};
// use html5ever::driver::ParseOpts;
// use html5ever::tokenizer::Attribute;
// use html5ever::tree_builder::TreeBuilderOpts;
// use html5ever::{parse as parse_ever, one_input, serialize};

//use scraper::{Selector, Html};

//use html5ever::rcdom::{RcDom, Handle, Element, ElementEnum, NodeEnum, Node, Text};

static HOST: &'static str = "www.google.com";
static UA: &'static str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_9_5) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/48.0.2564.116 Safari/537.36";

fn main() {
    let connector = CleartextConnector::new(HOST);
    let mut client = SimpleClient::with_connector(connector).unwrap();
    // let resp = client.get( b"/finance?q=TYO%3A6641&ei=lZ_iVoihAsyZ0ATd4KqQBA", &[] ).unwrap();
    // let resp = client.get( b"/finance/getprices?p=1d&f=d,h,o,l,c,v&i=300&x=INDEXNIKKEI&q=NI225", &[] ).unwrap();
    let resp = client.get( b"/", &[ (b"User-Agent".to_vec(), UA.as_bytes().to_vec()  ) ] ).unwrap();

    println!("{}", str::from_utf8(&resp.body).unwrap());
}
