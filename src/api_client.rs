use std::io::Read;
use hyper::Client;
use hyper::net::HttpsConnector;
use hyper_openssl::OpensslClient;

pub struct Ssl {
    client: Client
}
impl Ssl {
    pub fn new() -> Ssl {
        let ssl = OpensslClient::new().unwrap();
        let connector = HttpsConnector::new( ssl );
        Ssl {
            client: Client::with_connector( connector )
        }
    }

    pub fn sync_get( &self, url: &str ) -> String {
        let mut res = self.client.get( url ).send().unwrap();
        let mut body = vec![];
        res.read_to_end( &mut body ).unwrap();
        String::from_utf8_lossy( &body ).into_owned()
    }
}
