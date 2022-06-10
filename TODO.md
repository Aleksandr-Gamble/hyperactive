
#### CORS and application/json headers
Don't add them automatically, add the headers the user wants 

#### Transport-Level Security
you currently have a pool with no TLS. Add TLS, perhaps following this [example](https://docs.rs/tokio-postgres-native-tls/0.1.0-rc.1/tokio_postgres_native_tls/):

```
use native_tls::{Certificate, TlsConnector};
use tokio_postgres_native_tls::MakeTlsConnector;
use std::fs;

let cert = fs::read("database_cert.pem").unwrap();
let cert = Certificate::from_pem(&cert).unwrap();
let connector = TlsConnector::builder()
    .add_root_certificate(cert)
    .build()
    .unwrap();
let connector = MakeTlsConnector::new(connector);

let connect_future = tokio_postgres::connect(
    "host=localhost user=postgres sslmode=require",
    connector,
);
```
