use std::sync::Arc;

use sirara_core::utils::configuration::Config;
use sirara_core::infrastructure::services::http::request_client::RequestHttpClient;
use sirara_core::infrastructure::services::fx_quote::fx_rate_gateway::FxRatesGateway;
use sirara_core::infrastructure::services::fx_quote::fx_rate_impl::ForexRateApiGateway;

#[tokio::test]
async fn test_live_fx_rates_gateway() {
    let cfg = match Config::load() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Config::load() failed: {e:#}");
            panic!("Config::load() failed");
        }
    };

    eprintln!("fx.base_url = {}", cfg.fx.base_url);
    eprintln!("fx.api_key present? {}", cfg.fx.api_key.is_some());

    let http = Arc::new(RequestHttpClient::new());

    let gw = match ForexRateApiGateway::from_config(http, &cfg) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("ForexRateApiGateway::from_config failed: {e:?}");
            panic!("failed to build gateway");
        }
    };

    let resp = match gw.latest("USD", &["USD", "NGN", "KES"]).await {
        Ok(r) => r,
        Err(e) => {
            eprintln!("gw.latest(...) failed: {e:?}");
            panic!("fx api call failed");
        }
    };

    eprintln!("resp = {:?}", resp);

    assert!(resp.success);
    assert_eq!(resp.base, "USD");
    assert!(resp.rates.contains_key("USD"));
}
