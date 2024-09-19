gateway {
  name        = "frankfurt-gateway"
  region      = "eu-central-1"
  listen_port = 8080
  
  services = [
    {
      name    = "localserver"
      address = "127.0.0.1"
      port    = 8081
    },
    {
      name    = "mercury"
      address = "127.0.0.1"
      port    = 8080
    },
    {
      name  = "qdrant"
      address = "127.0.0.1"
      port = 6333
    },
    {
      name  = "cloudflare"
      address = "1.1.1.1"
      port = 443
    }
  ]
  
  latency {
    interval = "15s"
    timeout  = "1s"
  }
  
  heartbeat {
    interval = "10s"
    retries  = 3
    timeout  = "2s"
  }
  
  failover {
    retries  = 5
    interval = "2s"
  }
}