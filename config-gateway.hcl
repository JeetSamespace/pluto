gateway {
  name        = "frankfurt-gateway"
  region      = "eu-central-1"
  listen_port = 8080
  
  services = [
    {
      name    = "llm"
      address = "195.16.17.2"
      port    = 5677
    },
    {
      name    = "asr"
      address = "195.8.18.2"
      port    = 5677
    },
    {
      name    = "tts"
      address = "195.8.18.2"
      port    = 5678
    }
  ]
  
  latency {
    interval = "5s"
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