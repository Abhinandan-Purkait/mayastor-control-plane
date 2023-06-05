use anyhow::Context;
use clap::ArgMatches;
use once_cell::sync::OnceCell;
use std::{collections::HashMap, time::Duration};

static CONFIG: OnceCell<CsiControllerConfig> = OnceCell::new();

// Global CSI Controller config.
pub(crate) struct CsiControllerConfig {
    /// REST API endpoint URL.
    rest_endpoint: String,
    /// I/O timeout for REST API operations.
    io_timeout: Duration,
    /// Node Plugin selector label.
    node_selector: HashMap<String, String>,
    /// Max Outstanding Create Volume Requests.
    create_volume_limit: usize,
}

impl CsiControllerConfig {
    /// Initialize global instance of the CSI config. Must be called prior to using the config.
    pub(crate) fn initialize(args: &ArgMatches) -> anyhow::Result<()> {
        assert!(
            CONFIG.get().is_none(),
            "CSI Controller config already initialized"
        );

        let rest_endpoint = args
            .get_one::<String>("endpoint")
            .context("rest endpoint must be specified")?;

        let io_timeout = args
            .get_one::<String>("timeout")
            .context("I/O timeout must be specified")?
            .parse::<humantime::Duration>()?;

        let create_volume_limit = *args
            .get_one::<usize>("create-volume-limit")
            .context("create-volume-limit must be specified")?;

        let node_selector = csi_driver::csi_node_selector_parse(
            args.get_many::<String>("node-selector")
                .map(|s| s.map(|s| s.as_str())),
        )?;

        CONFIG.get_or_init(|| Self {
            rest_endpoint: rest_endpoint.into(),
            io_timeout: io_timeout.into(),
            node_selector,
            create_volume_limit,
        });
        Ok(())
    }

    /// Get global instance of CSI controller config.
    pub(crate) fn get_config() -> &'static CsiControllerConfig {
        CONFIG
            .get()
            .expect("CSI Controller config is not initialized")
    }

    /// Get REST API endpoint.
    pub(crate) fn rest_endpoint(&self) -> &str {
        &self.rest_endpoint
    }

    /// The maximum number of concurrent create volume requests.
    pub(crate) fn create_volume_limit(&self) -> usize {
        self.create_volume_limit
    }

    /// Get I/O timeout for REST API operations.
    pub(crate) fn io_timeout(&self) -> Duration {
        self.io_timeout
    }

    /// Get the node selector label segment.
    pub(crate) fn node_selector_segment(&self) -> HashMap<String, String> {
        self.node_selector.clone()
    }
}
