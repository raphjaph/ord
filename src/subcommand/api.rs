use super::*;
use crate::rpc::start_grpc_server;
use anyhow::Ok;
use std::panic;
use tokio::runtime::Runtime;

pub(crate) fn run(options: Options) -> Result {

  //Hook to make sure that the program panics on any async or thread panic
  panic::set_hook(Box::new(|info| {
    panic!("Panic hooked, let's panic")
  }));

  let result = Runtime::new().unwrap().block_on(start_grpc_server(options));

  Ok(())
}
