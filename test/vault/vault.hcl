# Example oversimplified vault configuration

listener "tcp" {
    address       = "{{ address }}"
    tls_cert_file = "{{ tls_cert_file }}"
    tls_key_file  = "{{ tls_key_file }}"
}

storage "consul" {
    address = "{{ consul_address }}"
    path = "vault/"
}

ui = {{ ui }}
