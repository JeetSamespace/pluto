

gateway {
  name        = "frankfurt-gateway"
  region      = "eu-central-1"
  listen_port = 8080
  
  services = [
    {
      id = "asr"
      name    = "asr"
      address = "127.0.0.1"
      port    = 5500
      health_check = {
        type = "http"
        url = "http://127.0.0.1:5500/"
        interval = "10s"
        timeout = "2s"
      }
    },
    {
      id = "tts"
      name    = "tts"
      address = "127.0.0.1"
      port    = 5501
      health_check = {
        type = "tcp"
        interval = "10s"
        timeout = "2s"
      }
    },
    {
      id = "llm"
      name    = "llm"
      address = "127.0.0.1"
      port    = 5502
      health_check = {
        type = "tcp"
        interval = "10s"
        timeout = "2s"
      }
    },
  ]

  transport {
    type = "nats"
    nats  {
      url = "nats://localhost:4222"
    }
  }
  
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