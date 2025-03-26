use std::sync::mpsc::Sender;
use std::ffi::CStr;

use ctp_common::{DepthMarketData, MdSpi, RspInfo, RspUserLogin, UserLogout};

/// CTP Market Data SPI implementation to handle callbacks from the CTP API
pub struct CTPMDSPI {
    /// Channel to send market data to the server
    sender: Sender<DepthMarketData>,
    /// Whether we're logged in
    logged_in: bool,
}

impl CTPMDSPI {
    pub fn new(sender: Sender<DepthMarketData>) -> Self {
        Self {
            sender,
            logged_in: false,
        }
    }
}

impl MdSpi for CTPMDSPI {
    fn on_front_connected(&mut self) {
        println!("CTP MD API connected to front");
    }

    fn on_front_disconnected(&mut self, reason: i32) {
        println!("CTP MD API disconnected from front, reason: {}", reason);
        self.logged_in = false;
    }

    fn on_rsp_user_login(&mut self, rsp_user_login: Option<&RspUserLogin>, rsp_info: Option<&RspInfo>, request_id: i32, is_last: bool) {
        if let Some(rsp_info) = rsp_info {
            if rsp_info.ErrorID != 0 {
                let error_msg = unsafe { CStr::from_ptr(rsp_info.ErrorMsg.as_ptr()) };
                println!("Login failed: {:?} {}", error_msg, rsp_info.ErrorID);
                return;
            }
        }

        if let Some(login_info) = rsp_user_login {
            println!(
                "Login successful. Trading day: {}, Login time: {}, Broker ID: {}, User ID: {}",
                login_info.TradingDay, login_info.LoginTime, login_info.BrokerID, login_info.UserID
            );
            self.logged_in = true;
        }
    }

    fn on_rsp_user_logout(&mut self, rsp_user_logout: Option<&UserLogout>, rsp_info: Option<&RspInfo>, request_id: i32, is_last: bool) {
        if let Some(rsp_info) = rsp_info {
            if rsp_info.ErrorID != 0 {
                let error_msg = unsafe { CStr::from_ptr(rsp_info.ErrorMsg.as_ptr()) };
                println!("Logout response with error: {:?} {}", error_msg, rsp_info.ErrorID);
                return;
            }
        }

        if let Some(logout_info) = rsp_user_logout {
            println!(
                "Logout successful. Broker ID: {}, User ID: {}",
                logout_info.BrokerID, logout_info.UserID
            );
        }

        self.logged_in = false;
    }

    fn on_rsp_error(&mut self, rsp_info: Option<&RspInfo>, request_id: i32, is_last: bool) {
        if let Some(info) = rsp_info {
            let error_msg = unsafe { CStr::from_ptr(info.ErrorMsg.as_ptr()) };
            println!(
                "Error response: Error ID: {}, Error message: {:?}",
                info.ErrorID, error_msg
            );
        }
    }

    fn on_rsp_sub_market_data(&mut self, specific_instrument: Option<&ctp_common::SpecificInstrument>, rsp_info: Option<&RspInfo>, request_id: i32, is_last: bool) {
        if let Some(specific_instrument) = specific_instrument {
            println!(
                "Subscribed to market data for instrument: {}",
                specific_instrument.InstrumentID
            );
        }

        if let Some(rsp_info) = rsp_info {
            if rsp_info.ErrorID != 0 {
                let error_msg = unsafe { CStr::from_ptr(rsp_info.ErrorMsg.as_ptr()) };
                println!(
                    "Subscribe market data error: ID: {}, Msg: {:?}",
                    rsp_info.ErrorID, error_msg
                );
            }
        }
    }

    fn on_rsp_unsub_market_data(&mut self, specific_instrument: Option<&ctp_common::SpecificInstrument>, rsp_info: Option<&RspInfo>, request_id: i32, is_last: bool) {
        if let Some(specific_instrument) = specific_instrument {
            println!(
                "Unsubscribed from market data for instrument: {}",
                specific_instrument.InstrumentID
            );
        }

        if let Some(rsp_info) = rsp_info {
            if rsp_info.ErrorID != 0 {
                let error_msg = unsafe { CStr::from_ptr(rsp_info.ErrorMsg.as_ptr()) };
                println!(
                    "Unsubscribe market data error: ID: {}, Msg: {:?}",
                    rsp_info.ErrorID, error_msg
                );
            }
        }
    }

    fn on_rtn_depth_market_data(&mut self, depth_market_data: Option<&DepthMarketData>) {
        if let Some(market_data) = depth_market_data {
            // Clone the market data and send it to the server
            let cloned_data = DepthMarketData {
                TradingDay: market_data.TradingDay.clone(),
                InstrumentID: market_data.InstrumentID.clone(),
                ExchangeID: market_data.ExchangeID.clone(),
                ExchangeInstID: market_data.ExchangeInstID.clone(),
                LastPrice: market_data.LastPrice,
                PreSettlementPrice: market_data.PreSettlementPrice,
                PreClosePrice: market_data.PreClosePrice,
                PreOpenInterest: market_data.PreOpenInterest,
                OpenPrice: market_data.OpenPrice,
                HighestPrice: market_data.HighestPrice,
                LowestPrice: market_data.LowestPrice,
                Volume: market_data.Volume,
                Turnover: market_data.Turnover,
                OpenInterest: market_data.OpenInterest,
                ClosePrice: market_data.ClosePrice,
                SettlementPrice: market_data.SettlementPrice,
                UpperLimitPrice: market_data.UpperLimitPrice,
                LowerLimitPrice: market_data.LowerLimitPrice,
                PreDelta: market_data.PreDelta,
                CurrDelta: market_data.CurrDelta,
                UpdateTime: market_data.UpdateTime.clone(),
                UpdateMillisec: market_data.UpdateMillisec,
                BidPrice1: market_data.BidPrice1,
                BidVolume1: market_data.BidVolume1,
                AskPrice1: market_data.AskPrice1,
                AskVolume1: market_data.AskVolume1,
                BidPrice2: market_data.BidPrice2,
                BidVolume2: market_data.BidVolume2,
                AskPrice2: market_data.AskPrice2,
                AskVolume2: market_data.AskVolume2,
                BidPrice3: market_data.BidPrice3,
                BidVolume3: market_data.BidVolume3,
                AskPrice3: market_data.AskPrice3,
                AskVolume3: market_data.AskVolume3,
                BidPrice4: market_data.BidPrice4,
                BidVolume4: market_data.BidVolume4,
                AskPrice4: market_data.AskPrice4,
                AskVolume4: market_data.AskVolume4,
                BidPrice5: market_data.BidPrice5,
                BidVolume5: market_data.BidVolume5,
                AskPrice5: market_data.AskPrice5,
                AskVolume5: market_data.AskVolume5,
                AveragePrice: market_data.AveragePrice,
                ActionDay: market_data.ActionDay.clone(),
            };

            // Send market data to the server
            if let Err(e) = self.sender.send(cloned_data) {
                println!("Failed to send market data: {}", e);
            }
        }
    }
} 