# rfid-trf7970-grpc

## Introduction
This repository provides a gRPC interface to interact with the TRF7970 RFID module from TI. It is written in Rust.  

## Building
Make sure rustc version at least 1.47.0 is installed. 

```rustc --version```

### Testing
Simply run cargo test. The testing is done via Mockall

```cargo test```

### Running
This would only work if the RFID is already connected to your computer. You should see the gRPC server listening on port 50051 if all is good

``` cargo run ```
