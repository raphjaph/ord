use self::data::{
  ord_api_server::{OrdApi, OrdApiServer},
  InscribeRequest, InscribeResponse,
};
use crate::Options;
use crate::{
  subcommand::wallet::inscribe::{Inscribe},
  FeeRate,
};
use bitcoin::util::address::Address;
use std::{env, fs::File, io::Write, path::PathBuf, str::FromStr};
use tonic::{transport::Server, Request, Response, Status};
pub mod data {
  tonic::include_proto!("ordinals");
}

pub struct Ord {
  options: Options,
}

/// Starts the gRPC server.
pub(crate) async fn start_grpc_server(options: Options) -> Result<(), Box<dyn std::error::Error>> {


  let ord = Ord { options };
  let port = 50051;
  let socket = format!("0.0.0.0:{port}");
  println!("Starting gRPC API on port {}",port);

  let addr = socket.parse().unwrap();
  Server::builder()
    .add_service(OrdApiServer::new(ord))
    .serve(addr)
    .await?;

  Ok(())
}

#[tonic::async_trait]
impl OrdApi for Ord {
  async fn inscribe(
    &self,
    request: Request<InscribeRequest>,
  ) -> Result<Response<InscribeResponse>, Status> {
    //TODO Proper logging

    let request = request.into_inner();
    println!( "Inscribing request order id: {}", request.order_id);

    //store the inscription on disk
    let _ = match store_inscription(request.clone()) {
      Ok(_) => Ok(()),
      Err(e) => {
        println!("Error: {}", e);
        Err(Status::internal(e.to_string()))
      }
    };

    //Fee rate
    let fee_rate = FeeRate::try_from(request.fee_rate).unwrap();

    //Create Inscribe struct
    let file = format!(
      "{}/{}.{}",
      env::var("ORD_PATH").unwrap_or("/ord".to_string()),
      request.order_id,
      request.inscription_file_extension
    );

    //If destination address is not empty, create and Address type of it
    let inscribe;
    if request.destination_address.is_empty() {
      inscribe = Inscribe {
        fee_rate,
        dry_run: request.dry_run,
        file: PathBuf::from(&file),
        destination: None,
        satpoint: None,
        commit_fee_rate: None,
        no_backup: false,
        no_limit: false,
      };
    } else {
      inscribe = Inscribe {
        fee_rate,
        dry_run: request.dry_run,
        file: PathBuf::from(&file),
        destination: Some(Address::from_str(&request.destination_address.clone()).unwrap()),
        satpoint: None,
        commit_fee_rate: None,
        no_backup: false,
        no_limit: false,
      };
    }

    let output = match inscribe.run_output(self.options.clone()) {
      Ok(output) => output.unwrap(),
      Err(e) => {
        println!("Error: {}", e);
        return Err(Status::internal(e.to_string()));
      }
    };

    let response = InscribeResponse {
      commit_txid: output.commit.to_string(),
      reveal_txid: output.reveal.to_string(),
      inscription_id: output.inscription.to_string(),
      dry_run: request.dry_run,
      network_fee: output.fees,
    };

    //Remove the file from disk
    match std::fs::remove_file(&file) {
      Ok(_) => println!("File removed for order id: {}", request.order_id),
      Err(e) => println!("Error: {}", e),
    }

    Ok(Response::new(response))
  }
}

//function that gets and InscribeRequest and stores on disk as a file
fn store_inscription(inscription: InscribeRequest) -> Result<(), Box<dyn std::error::Error>> {
  //get the default path for storing files from the env var
  let path = env::var("ORD_PATH").unwrap_or("/ord".to_string());

  //create the file

  let path = format!(
    "{}/{}.{}",
    path, inscription.order_id, inscription.inscription_file_extension
  );

  //Create the path if it doesn't exist
  let pathbuf = PathBuf::from(&path);
  if !pathbuf.exists() {
    std::fs::create_dir_all(pathbuf.parent().unwrap())?;
  }

  let mut file = match File::create(&path) {
    Err(why) => panic!("couldn't create {}: {}", path, why),
    Ok(file) => file,
  };

  //Convert vec<U8, Global> to u8 array
  let mut bytes: Vec<u8> = Vec::new();

  for byte in inscription.inscription_blob {
    bytes.push(byte);
  }

  file.write_all(&bytes)?;
  Ok(())
}