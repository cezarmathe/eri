# Example oversimplified vault configuration

listener "tcp" {
  address       = {{ vault.address }}
  tls_cert_file = {{ vault.tls_cert_file }}
  tls_key_file  = {{ vault.tls_key_file }}
}

storage "consul" {
  address = {{ vault.consul_address }}
  path = "vault/"
}

ui = {{ vault.ui }}
