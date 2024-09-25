orbit {
  listen_port = 9090
  max_connections = 1000

  gateways = [
    {
      host = "203.0.113.1"
      port    = 8080
    },
    {
      host = "198.51.100.2"
      port    = 8080
    }
  ]

  transport {
    type = "nats"
    nats  {
      url = "nats://localhost:4222"
    }
  }
  
  heartbeat {
    interval = "15s"
    timeout  = "3s"
    retries  = 3
  }

  load_balancing {
    method = "round_robin" // round_robin, least_connections, random, ip_hash
  }

  security {
    ssl_enabled = true
    cert_file   = "/path/to/cert.pem"
    key_file    = "/path/to/key.pem"
  }

  logging {
    level = "info" // debug, info, warn, error
    file  = "/var/log/orbit.log"
  }

  metrics {
    enabled = true
    endpoint = "/metrics"
  }
}