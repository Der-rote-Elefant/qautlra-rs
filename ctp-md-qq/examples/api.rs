use std::ffi::CString;

use ctp_md_qq::*;

struct Spi;
impl MdSpi for Spi {}

fn main() {
    let flow_path = ::std::ffi::CString::new("").unwrap();
    let mut md_api = MdApi::new(flow_path, false, false);
    md_api.register_spi(Box::new(Spi));
    md_api.register_front(std::ffi::CString::new("tcp://120.136.160.67:33441").unwrap());
    md_api.init();
    std::thread::sleep(std::time::Duration::from_secs(1));
    match md_api.req_user_login(&Default::default(), 1) {
        Ok(()) => println!("req_user_login ok"),
        Err(err) => println!("req_user_login err: {:?}", err),
    };

    std::thread::sleep(std::time::Duration::from_secs(1));
    let instrument_ids = vec![CString::new("au2212").unwrap()];
    match md_api.subscribe_market_data(&instrument_ids.clone()) {
        Ok(()) => println!("subscribe_market_data ok"),
        Err(err) => println!("subscribe_market_data err: {:?}", err),
    };
    // std::thread::sleep(std::time::Duration::from_secs(1));
    // match md_api.subscribe_for_quote_rsp(&instrument_ids) {
    //     Ok(()) => println!("subscribe_for_quote_rsp ok"),
    //     Err(err) => println!("subscribe_for_quote_rsp err: {:?}", err),
    // };
    std::thread::sleep(std::time::Duration::from_secs(5000));
    /*
    match md_api.req_user_logout(&Default::default(), 2) {
        Ok(()) => println!("req_user_logout ok"),
        Err(err) => println!("req_user_logout err: {:?}", err),
    };
    std::thread::sleep(std::time::Duration::from_secs(1));
    */
}
