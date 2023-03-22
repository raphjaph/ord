use super::*;
use anyhow::Ok;
use tokio::runtime::Runtime;
use crate::rpc::start_grpc_server;
pub(crate) fn run(options: Options) -> Result {

  let result = Runtime::new().unwrap().block_on(start_grpc_server(options));

  Ok(())

}

