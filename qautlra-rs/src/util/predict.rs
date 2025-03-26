use regex::Regex;
pub fn adjust_market(code: &str) -> String {
    let re = Regex::new(r"[a-zA-z]+").unwrap();
    let res = re.captures(code);
    match res {
        Some(x) => {
            let market = x.get(0).unwrap().as_str();
            println!("{:#?} => {:#?}", code, market);
            match market {
                "XSHG" => "stock_cn".to_string(),
                "XSHE" => "stock_cn".to_string(),
                _ => "future_cn".to_string(),
            }
        }
        None => "stock_cn".to_string(),
    }
}
