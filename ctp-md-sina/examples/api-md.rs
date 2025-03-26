use std::ffi::CString;
use std::sync::mpsc::channel;
use std::vec;

use ctp_md_sina::*;
struct Spi {
    sender: std::sync::mpsc::Sender<DepthMarketData>,
}
impl MdSpi for Spi {
    fn on_rtn_depth_market_data(
        &mut self,
        depth_market_data: Option<&CThostFtdcDepthMarketDataField>,
    ) {
        if depth_market_data.is_some() {
            let depth_market_datax: CThostFtdcDepthMarketDataField =
                depth_market_data.unwrap().to_owned();

            self.sender.send(depth_market_datax.to_struct()).unwrap();
        }
    }
}

pub struct MDInstance {
    md_api: MdApi,
    subscribe: Vec<String>,
}

impl MDInstance {
    pub fn new(frontmd_addr: &str, sender: std::sync::mpsc::Sender<DepthMarketData>) -> Self {
        let flow_path = ::std::ffi::CString::new("").unwrap();
        let mut md_api = MdApi::new(flow_path, false, false);

        md_api.register_spi(Box::new(Spi { sender: sender }));
        md_api.register_front(std::ffi::CString::new(frontmd_addr).unwrap());
        md_api.init();
        std::thread::sleep(std::time::Duration::from_secs(1));
        let subscribe = Vec::new();

        Self {
            md_api: md_api,
            subscribe,
        }
    }

    pub fn login(&mut self) {
        match self.md_api.req_user_login(&Default::default(), 1) {
            Ok(()) => {
                std::thread::sleep(std::time::Duration::from_secs(1));
                println!("req_user_login ok")
            }
            Err(err) => println!("req_user_login err: {:?}", err),
        };
    }

    pub fn subscribe(&mut self, subscribe: Vec<String>) {
        let new_subscribe = subscribe
            .iter()
            .filter(|x| !self.subscribe.contains(x))
            .map(|x| x.to_string())
            .collect::<Vec<String>>();

        self.subscribe.extend(new_subscribe.clone());

        let instrument_ids = new_subscribe
            .iter()
            .map(|x| CString::new(x.as_str()).unwrap())
            .collect::<Vec<_>>();

        println!("!!subscribe_market_data instrument_ids: {:?}", instrument_ids);
        match self.md_api.subscribe_market_data(&instrument_ids.clone()) {
            Ok(()) => println!("subscribe_market_data ok"),
            Err(err) => println!("subscribe_market_data err: {:?}", err),
        };
    }

    pub fn unsubscribe(&mut self, subscribe: Vec<String>) {
        let instrument_ids = subscribe
            .iter()
            .map(|x| CString::new(x.as_str()).unwrap())
            .collect::<Vec<_>>();
        match self.md_api.unsubscribe_market_data(&instrument_ids.clone()) {
            Ok(()) => println!("unsubscribe_market_data ok"),
            Err(err) => println!("unsubscribe_market_data err: {:?}", err),
        };
    }

    pub fn close(&mut self) {
        match self.md_api.req_user_logout(&Default::default(), 2) {
            Ok(()) => println!("req_user_logout ok"),
            Err(err) => println!("req_user_logout err: {:?}", err),
        };
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

pub struct MarketGateway {
    recv: std::sync::mpsc::Receiver<DepthMarketData>,
    subscribe: Vec<String>,
}

fn main() {
    const FRONT1: &'static str = "tcp://120.136.160.67:33441"; // 创元期货
    const FRONT2: &'static str = "tcp://180.166.103.21:57213"; //银河期货
    const FRONT3: &'static str = "tcp://116.228.31.198:43213"; //渤海期货
    const FRONT4: &'static str = "tcp://101.231.162.58:41213"; //光大期货

    let (tx, rx) = channel();

    let mut md_api = MDInstance::new(FRONT1, tx.clone());

    md_api.login();
    let subsc = vec![
        "600000".to_string(),
        "000001".to_string(),
        "00700".to_string(),
        "AAPL".to_string(),
    ];
    md_api.subscribe(subsc);

    while let Ok(data) = rx.recv() {
        println!("recv md: {:?}", data);
    }
}
