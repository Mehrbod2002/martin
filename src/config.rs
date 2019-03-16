use num_cpus;
use serde_yaml;
use std::error::Error;
use std::fs::File;
use std::io;
use std::io::prelude::*;

use super::cli::Args;
use super::db::PostgresPool;
use super::function_source::{get_function_sources, FunctionSources};
use super::table_source::{get_table_sources, TableSources};

#[derive(Clone, Debug, Serialize)]
pub struct Config {
  pub watch: bool,
  pub pool_size: u32,
  pub keep_alive: usize,
  pub worker_processes: usize,
  pub listen_addresses: String,
  pub connection_string: String,
  pub table_sources: Option<TableSources>,
  pub function_sources: Option<FunctionSources>,
}

#[derive(Deserialize)]
struct ConfigBuilder {
  pub watch: Option<bool>,
  pub pool_size: Option<u32>,
  pub keep_alive: Option<usize>,
  pub worker_processes: Option<usize>,
  pub listen_addresses: Option<String>,
  pub connection_string: String,
  pub table_sources: Option<TableSources>,
  pub function_sources: Option<FunctionSources>,
}

impl ConfigBuilder {
  pub fn finalize(self) -> Config {
    Config {
      watch: self.watch.unwrap_or(false),
      pool_size: self.pool_size.unwrap_or(20),
      keep_alive: self.keep_alive.unwrap_or(75),
      worker_processes: self.worker_processes.unwrap_or_else(num_cpus::get),
      listen_addresses: self
        .listen_addresses
        .unwrap_or_else(|| "0.0.0.0:3000".to_owned()),
      connection_string: self.connection_string,
      table_sources: self.table_sources,
      function_sources: self.function_sources,
    }
  }
}

pub fn read_config(file_name: &str) -> io::Result<Config> {
  let mut file = File::open(file_name)?;
  let mut contents = String::new();
  file.read_to_string(&mut contents)?;

  let config_builder: ConfigBuilder = serde_yaml::from_str(contents.as_str())
    .map_err(|err| io::Error::new(io::ErrorKind::Other, err.description()))?;

  Ok(config_builder.finalize())
}

pub fn generate_config(
  args: Args,
  connection_string: String,
  pool: &PostgresPool,
) -> io::Result<Config> {
  let conn = pool
    .get()
    .map_err(|err| io::Error::new(io::ErrorKind::Other, err.description()))?;

  let table_sources = get_table_sources(&conn)?;
  let function_sources = get_function_sources(&conn)?;

  let config = ConfigBuilder {
    watch: Some(args.flag_watch),
    keep_alive: args.flag_keep_alive,
    listen_addresses: args.flag_listen_addresses,
    connection_string: connection_string,
    pool_size: args.flag_pool_size,
    worker_processes: args.flag_workers,
    table_sources: Some(table_sources),
    function_sources: Some(function_sources),
  };

  let config = config.finalize();
  Ok(config)
}
