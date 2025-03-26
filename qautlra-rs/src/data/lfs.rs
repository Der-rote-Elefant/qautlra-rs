use std::fs::{self, File};

use polars::prelude::*;

use crate::util::tradedate::QATradeDate;

pub struct QALfs {
    base_dir: String,
    td: QATradeDate,
}

impl QALfs {
    pub fn new(base_dir: String) -> Self {
        let td = QATradeDate::new();
        QALfs { base_dir, td }
    }

    pub fn get_files(&self, files: Vec<String>) -> Result<DataFrame, PolarsError> {
        let mut dfs: Vec<DataFrame> = Vec::new();

        for file_path in files {
            let lf = LazyFrame::scan_parquet(&file_path, Default::default()).map_err(|_| {
                PolarsError::ComputeError("Failed to create LazyFrame from file".into())
            })?;
            let df = lf
                .collect()
                .map_err(|_| PolarsError::ComputeError("Failed to collect DataFrame".into()))?;
            dfs.push(df);
        }

        if let Some(first_df) = dfs.first().cloned() {
            let mut acc = first_df;
            for df in dfs.into_iter().skip(1) {
                acc = acc.vstack(&df).map_err(|_| {
                    PolarsError::ComputeError("Failed to vertically stack DataFrames".into())
                })?;
            }
            Ok(acc)
        } else {
            Err(PolarsError::NoData(
                "No DataFrames were created from the files".into(),
            ))
        }
    }

    pub fn load_bfq_day(&self, start: &str, end: &str) -> Result<DataFrame, PolarsError> {
        // Generate file paths based on trade dates
        let files: Vec<String> = self
            .td
            .get_trade_range(start, end)
            .iter()
            .map(|tradedate| format!("{}/bfqdata/stock_day_bfq_{}.pq", self.base_dir, tradedate))
            .collect();

        // Collect all DataFrames individually and then concatenate
        self.get_files(files)
    }
    pub fn load_hfq_day(&self, start: &str, end: &str) -> Result<DataFrame, PolarsError> {
        // Generate file paths based on trade dates
        let files: Vec<String> = self
            .td
            .get_trade_range(start, end)
            .iter()
            .map(|tradedate| format!("{}/daydata/stock_day_hfq_{}.pq", self.base_dir, tradedate))
            .collect();

        // Collect all DataFrames individually and then concatenate
        self.get_files(files)
    }
    pub fn load_bfq_min(&self, start: &str, end: &str) -> Result<DataFrame, PolarsError> {
        let files: Vec<String> = self
            .td
            .get_trade_range(start, end)
            .iter()
            .map(|tradedate| format!("{}/mindata/stock_min_{}.pq", self.base_dir, tradedate))
            .collect();
        self.get_files(files)
    }
    pub fn load_hfq_min(&self, start: &str, end: &str) -> Result<DataFrame, PolarsError> {
        // Generate file paths based on trade dates
        let files: Vec<String> = self
            .td
            .get_trade_range(start, end)
            .iter()
            .map(|tradedate| format!("{}/mindata/stock_min_hfq_{}.pq", self.base_dir, tradedate))
            .collect();

        // Collect all DataFrames individually and then concatenate
        self.get_files(files)
    }

    pub fn load_turnover(&self, start: &str, end: &str) -> Result<DataFrame, PolarsError> {
        let files: Vec<String> = self
            .td
            .get_trade_range(start, end)
            .iter()
            .map(|tradedate| format!("{}/turnover/turnover_{}.pq", self.base_dir, tradedate))
            .collect();

        // Collect all DataFrames individually and then concatenate
        self.get_files(files)
    }

    pub fn load_bfq_twap_stock_day(
        &self,
        start: &str,
        end: &str,
    ) -> Result<DataFrame, PolarsError> {
        let files: Vec<String> = self
            .td
            .get_trade_range(start, end)
            .iter()
            .map(|date| {
                format!(
                    "{}/bfqtwapdaydata/twap_stock_day_bfq_{}.pq",
                    self.base_dir, date
                )
            })
            .collect();
        self.get_files(files)
    }

    pub fn load_twap_index_day(&self, start: &str, end: &str) -> Result<DataFrame, PolarsError> {
        let files: Vec<String> = self
            .td
            .get_trade_range(start, end)
            .iter()
            .map(|date| {
                format!(
                    "{}/twapindexdaydata/twap_index_day_bfq_{}.pq",
                    self.base_dir, date
                )
            })
            .collect();
        self.get_files(files)
    }

    pub fn load_twap_index_pool_day(
        &self,
        start: &str,
        end: &str,
    ) -> Result<DataFrame, PolarsError> {
        let files: Vec<String> = self
            .td
            .get_trade_range(start, end)
            .iter()
            .map(|date| {
                format!(
                    "{}/twapindexpooldaydata/twap_index_pool_day_bfq_{}.pq",
                    self.base_dir, date
                )
            })
            .collect();
        self.get_files(files)
    }

    pub fn load_future_min(&self, start: &str, end: &str) -> Result<DataFrame, PolarsError> {
        let files: Vec<String> = self
            .td
            .get_trade_range(start, end)
            .iter()
            .map(|date| format!("{}/futuremin/future_min_{}.pq", self.base_dir, date))
            .collect();
        self.get_files(files)
    }

    pub fn load_future_day(&self, start: &str, end: &str) -> Result<DataFrame, PolarsError> {
        let files: Vec<String> = self
            .td
            .get_trade_range(start, end)
            .iter()
            .map(|date| format!("{}/futureday/future_day_{}.pq", self.base_dir, date))
            .collect();
        self.get_files(files)
    }

    pub fn load_stock_semi_day(&self, start: &str, end: &str) -> Result<DataFrame, PolarsError> {
        let files: Vec<String> = self
            .td
            .get_trade_range(start, end)
            .iter()
            .map(|date| {
                format!(
                    "{}/stock/semiday/stock_semiday_hfq_{}.pq",
                    self.base_dir, date
                )
            })
            .collect();
        self.get_files(files)
    }

    pub fn load_stockshare(&self, start: &str, end: &str) -> Result<DataFrame, PolarsError> {
        let files: Vec<String> = self
            .td
            .get_trade_range(start, end)
            .iter()
            .map(|date| format!("{}/stockshare/stockshare_{}.pq", self.base_dir, date))
            .collect();
        self.get_files(files)
    }

    pub fn load_barra(&self, start: &str, end: &str) -> Result<DataFrame, PolarsError> {
        let files: Vec<String> = self
            .td
            .get_trade_range(start, end)
            .iter()
            .map(|date| format!("{}/basic_data/barrav1_{}.pq", self.base_dir, date))
            .collect();
        self.get_files(files)
    }

    pub fn load_financial(&self, start: &str, end: &str) -> Result<DataFrame, PolarsError> {
        let files: Vec<String> = self
            .td
            .get_trade_range(start, end)
            .iter()
            .map(|date| format!("{}/financial/financial_v1_{}.pq", self.base_dir, date))
            .collect();
        self.get_files(files)
    }
    pub fn load_stock_industry(&self, start: &str, end: &str) -> Result<DataFrame, PolarsError> {
        let files: Vec<String> = self
            .td
            .get_trade_range(start, end)
            .iter()
            .map(|date| format!("{}/basic_data/industry_{}.pq", self.base_dir, date))
            .collect();
        self.get_files(files)
    }
}

#[cfg(test)]
mod test {
    use super::QALfs;
    #[test]
    fn load_bfq_day() {
        let base_dir = "/opt/cache/data".to_string();
        let lfs = QALfs::new(base_dir);
        let res = lfs.load_twap_index_pool_day("2024-01-01", "2024-01-22");
        println!("{:#?}", res);
    }
}
